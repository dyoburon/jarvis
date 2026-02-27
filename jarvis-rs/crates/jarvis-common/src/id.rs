use serde::{Deserialize, Serialize};
use std::fmt;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn new_correlation_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    let bytes = uuid.as_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        Self(new_id())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_valid_uuid() {
        let id = new_id();
        let parsed = uuid::Uuid::parse_str(&id);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().get_version_num(), 4);
    }

    #[test]
    fn new_id_is_unique() {
        let a = new_id();
        let b = new_id();
        assert_ne!(a, b);
    }

    #[test]
    fn correlation_id_length() {
        let cid = new_correlation_id();
        assert_eq!(cid.len(), 8);
    }

    #[test]
    fn correlation_id_is_hex() {
        let cid = new_correlation_id();
        assert!(cid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn correlation_id_is_unique() {
        let a = new_correlation_id();
        let b = new_correlation_id();
        assert_ne!(a, b);
    }

    #[test]
    fn session_id_new() {
        let sid = SessionId::new();
        let parsed = uuid::Uuid::parse_str(sid.as_str());
        assert!(parsed.is_ok());
    }

    #[test]
    fn session_id_display() {
        let sid = SessionId::new();
        let display = sid.to_string();
        assert_eq!(display, sid.as_str());
    }

    #[test]
    fn session_id_default() {
        let sid = SessionId::default();
        assert!(!sid.as_str().is_empty());
    }

    #[test]
    fn session_id_equality() {
        let sid = SessionId::new();
        let cloned = sid.clone();
        assert_eq!(sid, cloned);

        let other = SessionId::new();
        assert_ne!(sid, other);
    }

    #[test]
    fn session_id_serialization() {
        let sid = SessionId::new();
        let json = serde_json::to_string(&sid).unwrap();
        let deserialized: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(sid, deserialized);
    }

    #[test]
    fn session_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let s1 = SessionId::new();
        let s2 = s1.clone();
        set.insert(s1);
        set.insert(s2);
        assert_eq!(set.len(), 1);
    }
}
