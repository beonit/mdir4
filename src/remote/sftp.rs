use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use super::{
    backend::{
        RemoteDirectoryListing, RemoteEntry, RemoteEntryKind, RemoteName, RemoteReadBackend,
        RemoteReadError,
    },
    location::RemotePath,
    openssh_hosts::SshHostAlias,
};

const SFTP_INIT: u8 = 1;
const SFTP_VERSION: u8 = 2;
const SFTP_CLOSE: u8 = 4;
const SFTP_OPENDIR: u8 = 11;
const SFTP_READDIR: u8 = 12;
const SFTP_HANDLE: u8 = 102;
const SFTP_NAME: u8 = 104;
const SFTP_STATUS: u8 = 101;
const SFTP_STATUS_EOF: u32 = 1;
const MAX_SFTP_PACKET_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpConnectError {
    ClientUnavailable,
    AuthenticationOrHostVerification,
    InvalidRemoteHome,
}

impl SftpConnectError {
    pub fn message(self) -> &'static str {
        match self {
            Self::ClientUnavailable => "OpenSSH SFTP client is unavailable.",
            Self::AuthenticationOrHostVerification => {
                "SSH authentication or host verification failed. Configure it in ~/.ssh/config."
            }
            Self::InvalidRemoteHome => "Remote server returned an invalid home directory.",
        }
    }
}

pub trait SftpConnector: Send + Sync {
    fn probe_home(&self, alias: &SshHostAlias) -> Result<RemotePath, SftpConnectError>;
}

pub struct OpenSshSftpConnector {
    executable: String,
}

pub struct OpenSshSftpSession {
    connector: OpenSshSftpConnector,
    alias: SshHostAlias,
}

impl OpenSshSftpSession {
    pub fn new(alias: SshHostAlias) -> Self {
        Self {
            connector: OpenSshSftpConnector::default(),
            alias,
        }
    }
}

impl RemoteReadBackend for OpenSshSftpSession {
    fn read_dir(&self, path: &RemotePath) -> Result<RemoteDirectoryListing, RemoteReadError> {
        self.connector.read_dir(&self.alias, path)
    }
}

impl Default for OpenSshSftpConnector {
    fn default() -> Self {
        Self::new("sftp")
    }
}

impl OpenSshSftpConnector {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    fn parse_home(output: &[u8]) -> Result<RemotePath, SftpConnectError> {
        const PREFIX: &[u8] = b"Remote working directory: ";
        output
            .split(|byte| *byte == b'\n')
            .find_map(|line| line.strip_prefix(PREFIX))
            .and_then(|path| RemotePath::from_absolute(path).ok())
            .ok_or(SftpConnectError::InvalidRemoteHome)
    }

    /// Lists a directory through the OpenSSH SFTP subsystem. The SSH client resolves the
    /// configured alias itself, so this never reconstructs a host, user, or key path.
    pub fn read_dir(
        &self,
        alias: &SshHostAlias,
        path: &RemotePath,
    ) -> Result<RemoteDirectoryListing, RemoteReadError> {
        let mut child = Command::new("ssh")
            .args([
                "-oBatchMode=yes",
                "-oConnectTimeout=15",
                "-s",
                alias.as_str(),
                "sftp",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RemoteReadError::ConnectionLost)?;
        let mut stdin = child.stdin.take().ok_or(RemoteReadError::ConnectionLost)?;
        let mut stdout = child.stdout.take().ok_or(RemoteReadError::ConnectionLost)?;
        let result = read_directory_protocol(&mut stdin, &mut stdout, path);
        drop(stdin);
        let status = child.wait().map_err(|_| RemoteReadError::ConnectionLost)?;
        if !status.success() {
            return Err(RemoteReadError::ConnectionLost);
        }
        result
    }
}

impl SftpConnector for OpenSshSftpConnector {
    fn probe_home(&self, alias: &SshHostAlias) -> Result<RemotePath, SftpConnectError> {
        let mut child = Command::new(&self.executable)
            .args([
                "-oBatchMode=yes",
                "-oConnectTimeout=15",
                "-b",
                "-",
                alias.as_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| SftpConnectError::ClientUnavailable)?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(b"pwd\nquit\n")
                .map_err(|_| SftpConnectError::AuthenticationOrHostVerification)?;
        }
        let output = child
            .wait_with_output()
            .map_err(|_| SftpConnectError::AuthenticationOrHostVerification)?;
        if !output.status.success() {
            return Err(SftpConnectError::AuthenticationOrHostVerification);
        }
        Self::parse_home(&output.stdout)
    }
}

pub fn executable_is_available(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

fn read_directory_protocol(
    writer: &mut impl Write,
    reader: &mut impl Read,
    path: &RemotePath,
) -> Result<RemoteDirectoryListing, RemoteReadError> {
    send_packet(writer, SFTP_INIT, &3u32.to_be_bytes())?;
    let (kind, _) = read_packet(reader)?;
    if kind != SFTP_VERSION {
        return Err(RemoteReadError::Protocol);
    }

    let handle = request_handle(writer, reader, SFTP_OPENDIR, 1, path.as_bytes())?;
    let mut entries = Vec::new();
    loop {
        let mut payload = Vec::new();
        payload.extend_from_slice(&2u32.to_be_bytes());
        push_string(&mut payload, &handle)?;
        send_packet(writer, SFTP_READDIR, &payload)?;
        let (kind, payload) = read_packet(reader)?;
        match kind {
            SFTP_NAME => entries.extend(parse_name_packet(&payload, 2)?),
            SFTP_STATUS => {
                if parse_status(&payload, 2)? == SFTP_STATUS_EOF {
                    break;
                }
                return Err(RemoteReadError::ConnectionLost);
            }
            _ => return Err(RemoteReadError::Protocol),
        }
    }

    let mut payload = Vec::new();
    payload.extend_from_slice(&3u32.to_be_bytes());
    push_string(&mut payload, &handle)?;
    send_packet(writer, SFTP_CLOSE, &payload)?;
    let (kind, payload) = read_packet(reader)?;
    if kind != SFTP_STATUS || parse_status(&payload, 3)? != 0 {
        return Err(RemoteReadError::ConnectionLost);
    }
    RemoteDirectoryListing::new(path.clone(), entries).map_err(|_| RemoteReadError::Protocol)
}

fn request_handle(
    writer: &mut impl Write,
    reader: &mut impl Read,
    kind: u8,
    request_id: u32,
    path: &[u8],
) -> Result<Vec<u8>, RemoteReadError> {
    let mut payload = request_id.to_be_bytes().to_vec();
    push_string(&mut payload, path)?;
    send_packet(writer, kind, &payload)?;
    let (kind, payload) = read_packet(reader)?;
    if kind == SFTP_STATUS {
        return Err(RemoteReadError::ConnectionLost);
    }
    if kind != SFTP_HANDLE || read_u32(&payload, 0)? != request_id {
        return Err(RemoteReadError::Protocol);
    }
    read_string(&payload, 4).map(|(value, _)| value.to_vec())
}

fn parse_name_packet(payload: &[u8], request_id: u32) -> Result<Vec<RemoteEntry>, RemoteReadError> {
    if read_u32(payload, 0)? != request_id {
        return Err(RemoteReadError::Protocol);
    }
    let count = read_u32(payload, 4)? as usize;
    let mut offset = 8;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let (name, next) = read_string(payload, offset)?;
        offset = next;
        let (_, next) = read_string(payload, offset)?;
        offset = next;
        let (kind, size, next) = parse_attributes(payload, offset)?;
        offset = next;
        entries.push(RemoteEntry {
            name: RemoteName::from_bytes(name).map_err(|_| RemoteReadError::Protocol)?,
            kind,
            size,
        });
    }
    (offset == payload.len())
        .then_some(entries)
        .ok_or(RemoteReadError::Protocol)
}

fn parse_attributes(
    payload: &[u8],
    mut offset: usize,
) -> Result<(RemoteEntryKind, Option<u64>, usize), RemoteReadError> {
    const SIZE: u32 = 0x0000_0001;
    const UID_GID: u32 = 0x0000_0002;
    const PERMISSIONS: u32 = 0x0000_0004;
    const ACMODTIME: u32 = 0x0000_0008;
    const EXTENDED: u32 = 0x8000_0000;
    let flags = read_u32(payload, offset)?;
    offset += 4;
    let size = if flags & SIZE != 0 {
        let value = read_u64(payload, offset)?;
        offset += 8;
        Some(value)
    } else {
        None
    };
    if flags & UID_GID != 0 {
        offset = offset
            .checked_add(8)
            .filter(|end| *end <= payload.len())
            .ok_or(RemoteReadError::Protocol)?;
    }
    let permissions = if flags & PERMISSIONS != 0 {
        let value = read_u32(payload, offset)?;
        offset += 4;
        Some(value)
    } else {
        None
    };
    if flags & ACMODTIME != 0 {
        offset = offset
            .checked_add(8)
            .filter(|end| *end <= payload.len())
            .ok_or(RemoteReadError::Protocol)?;
    }
    if flags & EXTENDED != 0 {
        let count = read_u32(payload, offset)? as usize;
        offset += 4;
        for _ in 0..count {
            let (_, next) = read_string(payload, offset)?;
            let (_, next_value) = read_string(payload, next)?;
            offset = next_value;
        }
    }
    let kind = match permissions.map(|permissions| permissions & 0o170000) {
        Some(0o040000) => RemoteEntryKind::Directory,
        Some(0o120000) => RemoteEntryKind::Symlink,
        Some(0o100000) | None => RemoteEntryKind::File,
        Some(_) => RemoteEntryKind::Other,
    };
    Ok((kind, size, offset))
}

fn parse_status(payload: &[u8], request_id: u32) -> Result<u32, RemoteReadError> {
    (read_u32(payload, 0)? == request_id)
        .then(|| read_u32(payload, 4))
        .transpose()?
        .ok_or(RemoteReadError::Protocol)
}

fn send_packet(writer: &mut impl Write, kind: u8, payload: &[u8]) -> Result<(), RemoteReadError> {
    let length = payload
        .len()
        .checked_add(1)
        .filter(|length| *length <= MAX_SFTP_PACKET_BYTES)
        .ok_or(RemoteReadError::Protocol)?;
    writer
        .write_all(&(length as u32).to_be_bytes())
        .and_then(|_| writer.write_all(&[kind]))
        .and_then(|_| writer.write_all(payload))
        .and_then(|_| writer.flush())
        .map_err(|_| RemoteReadError::ConnectionLost)
}

fn read_packet(reader: &mut impl Read) -> Result<(u8, Vec<u8>), RemoteReadError> {
    let mut length = [0; 4];
    reader
        .read_exact(&mut length)
        .map_err(|_| RemoteReadError::ConnectionLost)?;
    let length = u32::from_be_bytes(length) as usize;
    if !(1..=MAX_SFTP_PACKET_BYTES).contains(&length) {
        return Err(RemoteReadError::Protocol);
    }
    let mut packet = vec![0; length];
    reader
        .read_exact(&mut packet)
        .map_err(|_| RemoteReadError::ConnectionLost)?;
    Ok((packet[0], packet[1..].to_vec()))
}

fn push_string(payload: &mut Vec<u8>, value: &[u8]) -> Result<(), RemoteReadError> {
    let length = u32::try_from(value.len()).map_err(|_| RemoteReadError::Protocol)?;
    payload.extend_from_slice(&length.to_be_bytes());
    payload.extend_from_slice(value);
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, RemoteReadError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(RemoteReadError::Protocol)?;
    Ok(u32::from_be_bytes(
        value.try_into().expect("slice length checked"),
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, RemoteReadError> {
    let value = bytes
        .get(offset..offset + 8)
        .ok_or(RemoteReadError::Protocol)?;
    Ok(u64::from_be_bytes(
        value.try_into().expect("slice length checked"),
    ))
}

fn read_string(bytes: &[u8], offset: usize) -> Result<(&[u8], usize), RemoteReadError> {
    let length = read_u32(bytes, offset)? as usize;
    let start = offset.checked_add(4).ok_or(RemoteReadError::Protocol)?;
    let end = start.checked_add(length).ok_or(RemoteReadError::Protocol)?;
    bytes
        .get(start..end)
        .map(|value| (value, end))
        .ok_or(RemoteReadError::Protocol)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn packet(kind: u8, payload: &[u8]) -> Vec<u8> {
        let mut value = ((payload.len() + 1) as u32).to_be_bytes().to_vec();
        value.push(kind);
        value.extend_from_slice(payload);
        value
    }

    fn string(value: &[u8]) -> Vec<u8> {
        let mut encoded = (value.len() as u32).to_be_bytes().to_vec();
        encoded.extend_from_slice(value);
        encoded
    }

    #[test]
    fn parses_the_sftp_remote_home_without_turning_it_into_a_local_path() {
        let home = OpenSshSftpConnector::parse_home(
            b"Connected to host.\nRemote working directory: /srv/\xff\n",
        )
        .unwrap();
        assert_eq!(home.as_bytes(), b"/srv/\xff");
    }

    #[test]
    fn errors_are_redacted_and_actionable() {
        assert_eq!(
            SftpConnectError::AuthenticationOrHostVerification.message(),
            "SSH authentication or host verification failed. Configure it in ~/.ssh/config."
        );
        assert!(
            !SftpConnectError::AuthenticationOrHostVerification
                .message()
                .contains("token")
        );
    }

    #[test]
    fn directory_protocol_preserves_non_utf8_names_and_sftp_file_types() {
        let mut incoming = packet(SFTP_VERSION, &3u32.to_be_bytes());
        let mut handle = 1u32.to_be_bytes().to_vec();
        handle.extend_from_slice(&string(b"handle"));
        incoming.extend_from_slice(&packet(SFTP_HANDLE, &handle));

        let mut names = 2u32.to_be_bytes().to_vec();
        names.extend_from_slice(&1u32.to_be_bytes());
        names.extend_from_slice(&string(b"dir-\xff"));
        names.extend_from_slice(&string(b"ignored display"));
        names.extend_from_slice(&0x0000_0005u32.to_be_bytes());
        names.extend_from_slice(&42u64.to_be_bytes());
        names.extend_from_slice(&0o040755u32.to_be_bytes());
        incoming.extend_from_slice(&packet(SFTP_NAME, &names));

        let mut eof = 2u32.to_be_bytes().to_vec();
        eof.extend_from_slice(&SFTP_STATUS_EOF.to_be_bytes());
        incoming.extend_from_slice(&packet(SFTP_STATUS, &eof));
        let mut closed = 3u32.to_be_bytes().to_vec();
        closed.extend_from_slice(&0u32.to_be_bytes());
        incoming.extend_from_slice(&packet(SFTP_STATUS, &closed));

        let path = RemotePath::from_absolute(b"/home/\xff").unwrap();
        let mut outgoing = Vec::new();
        let listing =
            read_directory_protocol(&mut outgoing, &mut Cursor::new(incoming), &path).unwrap();
        assert_eq!(listing.path, path);
        assert_eq!(listing.entries[0].name.as_bytes(), b"dir-\xff");
        assert_eq!(listing.entries[0].kind, RemoteEntryKind::Directory);
        assert_eq!(listing.entries[0].size, Some(42));
        assert!(
            outgoing
                .windows(b"/home/\xff".len())
                .any(|window| window == b"/home/\xff")
        );
    }
}
