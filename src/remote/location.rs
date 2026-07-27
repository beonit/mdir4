use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(String);

impl LocationId {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 32
            && value.bytes().enumerate().all(|(index, byte)| match byte {
                b'a'..=b'z' | b'0'..=b'9' => true,
                b'_' | b'-' => index > 0,
                _ => false,
            });
        valid
            .then_some(Self(value))
            .ok_or_else(|| "Location id must be an ASCII slug.".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemotePath(Vec<u8>);

impl RemotePath {
    pub fn root() -> Self {
        Self(vec![b'/'])
    }

    pub fn from_absolute(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let bytes = bytes.as_ref();
        if !bytes.starts_with(b"/") || bytes.contains(&0) {
            return Err("Remote path must be an absolute path without NUL bytes.".into());
        }
        let mut components: Vec<&[u8]> = Vec::new();
        for component in bytes.split(|byte| *byte == b'/') {
            match component {
                b"" | b"." => {}
                b".." => {
                    components.pop();
                }
                component => components.push(component),
            }
        }
        let mut normalized = vec![b'/'];
        for (index, component) in components.iter().enumerate() {
            if index > 0 {
                normalized.push(b'/');
            }
            normalized.extend_from_slice(component);
        }
        Ok(Self(normalized))
    }

    pub fn join(&self, child: impl AsRef<[u8]>) -> Result<Self, String> {
        let child = child.as_ref();
        if child.is_empty()
            || child.contains(&0)
            || child.contains(&b'/')
            || matches!(child, b"." | b"..")
        {
            return Err("Remote path component is invalid.".into());
        }
        let mut value = self.0.clone();
        if value != b"/" {
            value.push(b'/');
        }
        value.extend_from_slice(child);
        Ok(Self(value))
    }

    pub fn parent(&self) -> Self {
        if self.0 == b"/" {
            return Self::root();
        }
        let mut value = self.0.clone();
        let index = value.iter().rposition(|byte| *byte == b'/').unwrap_or(0);
        value.truncate(index.max(1));
        Self(value)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn display(&self) -> RemotePathDisplay<'_> {
        RemotePathDisplay(&self.0)
    }
}

impl fmt::Debug for RemotePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RemotePath")
            .field(&self.display())
            .finish()
    }
}

pub struct RemotePathDisplay<'a>(&'a [u8]);

impl fmt::Debug for RemotePathDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for RemotePathDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(text) = std::str::from_utf8(self.0) {
            return formatter.write_str(text);
        }
        for byte in self.0 {
            if byte.is_ascii_graphic() || *byte == b'/' || *byte == b' ' {
                formatter.write_str(&char::from(*byte).to_string())?;
            } else {
                write!(formatter, "\\x{byte:02X}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_id_is_a_stable_ascii_slug() {
        assert_eq!(
            LocationId::new("dev-west_2").unwrap().as_str(),
            "dev-west_2"
        );
        assert!(LocationId::new("DEV").is_err());
        assert!(LocationId::new("-dev").is_err());
    }

    #[test]
    fn remote_paths_normalize_without_escaping_root() {
        let path = RemotePath::from_absolute(b"/srv//project/../repo/").unwrap();
        assert_eq!(path.as_bytes(), b"/srv/repo");
        assert_eq!(path.parent().as_bytes(), b"/srv");
        assert_eq!(
            RemotePath::from_absolute(b"/../../").unwrap(),
            RemotePath::root()
        );
    }

    #[test]
    fn remote_paths_keep_protocol_bytes_separate_from_display_text() {
        let path = RemotePath::from_absolute(b"/data/\xff.bin").unwrap();
        assert_eq!(
            path.join(b"child").unwrap().as_bytes(),
            b"/data/\xff.bin/child"
        );
        assert_eq!(path.display().to_string(), "/data/\\xFF.bin");
        assert!(path.join(b"../escape").is_err());
    }
}
