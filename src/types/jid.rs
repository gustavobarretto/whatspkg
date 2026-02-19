use std::fmt;
use std::str::FromStr;

/// Known JID servers on WhatsApp.
pub const DEFAULT_USER_SERVER: &str = "s.whatsapp.net";
pub const GROUP_SERVER: &str = "g.us";
#[allow(dead_code)]
pub const LEGACY_USER_SERVER: &str = "c.us";
pub const BROADCAST_SERVER: &str = "broadcast";
#[allow(dead_code)]
pub const HIDDEN_USER_SERVER: &str = "lid";
#[allow(dead_code)]
pub const NEWSLETTER_SERVER: &str = "newsletter";

/// JID represents a WhatsApp user/entity ID (user@server or AD-JID).
///
/// JID (user/group/server identifier).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Jid {
    pub user: String,
    pub raw_agent: u8,
    pub device: u16,
    pub integrator: u16,
    pub server: String,
}

impl Jid {
    /// New regular JID (user@server).
    pub fn new(user: impl Into<String>, server: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            raw_agent: 0,
            device: 0,
            integrator: 0,
            server: server.into(),
        }
    }

    /// New AD-JID (user.agent:device@server) for device-specific addressing.
    pub fn new_ad(
        user: impl Into<String>,
        agent: u8,
        device: u16,
        server: impl Into<String>,
    ) -> Self {
        Self {
            user: user.into(),
            raw_agent: agent,
            device,
            integrator: 0,
            server: server.into(),
        }
    }

    /// Server JID (no user).
    pub fn server(server: impl Into<String>) -> Self {
        Self::new("", server)
    }

    /// Well-known JIDs.
    pub fn group_server() -> Self {
        Self::server(GROUP_SERVER)
    }
    pub fn default_server() -> Self {
        Self::server(DEFAULT_USER_SERVER)
    }
    pub fn broadcast_server() -> Self {
        Self::server(BROADCAST_SERVER)
    }
    pub fn status_broadcast() -> Self {
        Self::new("status", BROADCAST_SERVER)
    }

    /// User part as u64 (for normal user JIDs).
    pub fn user_int(&self) -> u64 {
        self.user.parse().unwrap_or(0)
    }

    /// JID without agent/device (regular user@server).
    pub fn to_non_ad(&self) -> Self {
        Self {
            user: self.user.clone(),
            raw_agent: 0,
            device: 0,
            integrator: self.integrator,
            server: self.server.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.server.is_empty()
    }

    pub fn is_broadcast_list(&self) -> bool {
        self.server == BROADCAST_SERVER && self.user != "status"
    }
}

impl FromStr for Jid {
    type Err = JidParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() == 1 {
            return Ok(Self::server(parts[0]));
        }
        if parts.len() != 2 {
            return Err(JidParseError);
        }
        let mut jid = Self {
            user: parts[0].to_string(),
            raw_agent: 0,
            device: 0,
            integrator: 0,
            server: parts[1].to_string(),
        };
        if jid.user.contains('.') {
            let ud: Vec<&str> = jid.user.splitn(2, '.').collect();
            if ud.len() != 2 {
                return Err(JidParseError);
            }
            let u0 = ud[0].to_string();
            let rest = ud[1].to_string();
            jid.user = u0;
            let ad: Vec<&str> = rest.split(':').collect();
            jid.raw_agent = ad[0].parse().map_err(|_| JidParseError)?;
            if ad.len() == 2 {
                jid.device = ad[1].parse().map_err(|_| JidParseError)?;
            }
        } else if jid.user.contains(':') {
            let ud: Vec<&str> = jid.user.splitn(2, ':').collect();
            if ud.len() != 2 {
                return Err(JidParseError);
            }
            let u0 = ud[0].to_string();
            let u1 = ud[1].to_string();
            jid.user = u0;
            jid.device = u1.parse().map_err(|_| JidParseError)?;
        }
        Ok(jid)
    }
}

#[derive(Debug)]
pub struct JidParseError;

impl std::fmt::Display for JidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid JID format")
    }
}

impl std::error::Error for JidParseError {}

impl fmt::Display for Jid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw_agent > 0 {
            write!(
                f,
                "{}.{}:{}@{}",
                self.user, self.raw_agent, self.device, self.server
            )
        } else if self.device > 0 {
            write!(f, "{}:{}@{}", self.user, self.device, self.server)
        } else if !self.user.is_empty() {
            write!(f, "{}@{}", self.user, self.server)
        } else {
            write!(f, "{}", self.server)
        }
    }
}

impl serde::Serialize for Jid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Jid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Jid::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jid_new_and_display() {
        let j = Jid::new("123456789", "s.whatsapp.net");
        assert_eq!(j.to_string(), "123456789@s.whatsapp.net");
        assert!(!j.is_empty());
        assert!(!j.is_broadcast_list());
    }

    #[test]
    fn jid_parse_roundtrip() {
        let s = "123456789@g.us";
        let j: Jid = s.parse().unwrap();
        assert_eq!(j.user, "123456789");
        assert_eq!(j.server, "g.us");
        assert_eq!(j.to_string(), s);
    }

    #[test]
    fn jid_parse_server_only() {
        let j: Jid = "g.us".parse().unwrap();
        assert_eq!(j.user, "");
        assert_eq!(j.server, "g.us");
        assert_eq!(j.to_string(), "g.us");
    }

    #[test]
    fn jid_with_device() {
        let j: Jid = "123:0@s.whatsapp.net".parse().unwrap();
        assert_eq!(j.user, "123");
        assert_eq!(j.device, 0);
        assert_eq!(j.server, "s.whatsapp.net");
    }

    #[test]
    fn jid_well_known() {
        assert_eq!(Jid::group_server().server, GROUP_SERVER);
        assert_eq!(Jid::default_server().server, DEFAULT_USER_SERVER);
        assert!(Jid::status_broadcast().is_broadcast_list() == false);
        let list = Jid::new("abc", BROADCAST_SERVER);
        assert!(list.is_broadcast_list());
    }

    #[test]
    fn jid_to_non_ad() {
        let j = Jid::new_ad("user", 1, 2, "s.whatsapp.net");
        let n = j.to_non_ad();
        assert_eq!(n.raw_agent, 0);
        assert_eq!(n.device, 0);
        assert_eq!(n.user, "user");
    }

    #[test]
    fn jid_user_int() {
        let j = Jid::new("987654321", "s.whatsapp.net");
        assert_eq!(j.user_int(), 987_654_321);
    }
}
