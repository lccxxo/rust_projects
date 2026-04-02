use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProfile {
    pub id: String,
    pub name: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_jump: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SshAuth {
    Password,
    Key { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionProfile {
    Local(LocalProfile),
    Ssh(SshProfile),
}

impl SessionProfile {
    pub fn id(&self) -> &str {
        match self {
            SessionProfile::Local(p) => &p.id,
            SessionProfile::Ssh(p) => &p.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SessionProfile::Local(p) => &p.name,
            SessionProfile::Ssh(p) => &p.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionGroup {
    pub name: String,
    pub sessions: Vec<SessionProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStore {
    pub groups: Vec<SessionGroup>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_json_roundtrip() {
        let group = SessionGroup {
            name: "Production".to_string(),
            sessions: vec![
                SessionProfile::Ssh(SshProfile {
                    id: "test-id".to_string(),
                    name: "web-server".to_string(),
                    host: "192.168.1.10".to_string(),
                    port: 22,
                    username: "user".to_string(),
                    auth: SshAuth::Password,
                    proxy_jump: None,
                }),
            ],
        };
        let json = serde_json::to_string(&group).unwrap();
        let decoded: SessionGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "Production");
        assert_eq!(decoded.sessions.len(), 1);
    }

    #[test]
    fn test_local_profile_json_roundtrip() {
        let profile = SessionProfile::Local(LocalProfile {
            id: "local-1".to_string(),
            name: "PowerShell".to_string(),
            shell: "powershell.exe".to_string(),
        });
        let json = serde_json::to_string(&profile).unwrap();
        let decoded: SessionProfile = serde_json::from_str(&json).unwrap();
        match decoded {
            SessionProfile::Local(p) => assert_eq!(p.shell, "powershell.exe"),
            _ => panic!("wrong variant"),
        }
    }
}
