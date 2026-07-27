use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use super::{location::RemotePath, openssh_hosts::SshHostAlias};

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
}

impl SftpConnector for OpenSshSftpConnector {
    fn probe_home(&self, alias: &SshHostAlias) -> Result<RemotePath, SftpConnectError> {
        let mut child = Command::new(&self.executable)
            .args(["-oBatchMode=yes", "-b", "-", alias.as_str()])
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
