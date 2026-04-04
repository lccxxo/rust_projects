// oxijade-core/src/session/ssh.rs
use oxijade_config::{SshAuth, SshProfile};

/// 将 SshProfile 转换为 ssh.exe 的命令行参数向量。
/// 使用系统自带 ssh.exe（Windows 10+），无需额外 crate。
pub fn build_args(profile: &SshProfile) -> Vec<String> {
    let mut args = vec!["ssh".to_string()];
    args.push("-p".to_string());
    args.push(profile.port.to_string());

    if let SshAuth::Key { path } = &profile.auth {
        args.push("-i".to_string());
        args.push(path.clone());
    }

    if let Some(jump) = &profile.proxy_jump {
        if !jump.is_empty() {
            args.push("-J".to_string());
            args.push(jump.clone());
        }
    }

    // 首次连接自动接受 host key，避免交互式阻塞
    args.push("-o".to_string());
    args.push("StrictHostKeyChecking=accept-new".to_string());

    args.push(format!("{}@{}", profile.username, profile.host));
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxijade_config::{SshAuth, SshProfile};

    fn make_profile(auth: SshAuth, jump: Option<&str>) -> SshProfile {
        SshProfile {
            id: "test".into(),
            name: "test".into(),
            host: "192.168.1.1".into(),
            port: 22,
            username: "admin".into(),
            auth,
            proxy_jump: jump.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_key_auth_args() {
        let args = build_args(&make_profile(
            SshAuth::Key { path: "/home/.ssh/id_rsa".into() },
            None,
        ));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"/home/.ssh/id_rsa".to_string()));
        assert!(!args.contains(&"-J".to_string()));
        assert_eq!(args.last().unwrap(), "admin@192.168.1.1");
    }

    #[test]
    fn test_proxy_jump_args() {
        let args = build_args(&make_profile(SshAuth::Password { password: String::new() }, Some("jump.example.com")));
        assert!(args.contains(&"-J".to_string()));
        assert!(args.contains(&"jump.example.com".to_string()));
    }

    #[test]
    fn test_password_auth_no_key_flag() {
        let args = build_args(&make_profile(SshAuth::Password { password: String::new() }, None));
        assert!(!args.contains(&"-i".to_string()));
        assert!(!args.contains(&"-J".to_string()));
    }

    #[test]
    fn test_empty_proxy_jump_ignored() {
        let args = build_args(&make_profile(SshAuth::Password { password: String::new() }, Some("")));
        assert!(!args.contains(&"-J".to_string()));
    }
}
