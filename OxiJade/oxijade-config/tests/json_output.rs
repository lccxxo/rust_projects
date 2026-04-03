use oxijade_config::{LocalProfile, SessionProfile, SshAuth, SshProfile};

#[test]
fn test_json_output() {
    // Test 1: LocalProfile
    let local = SessionProfile::Local(LocalProfile {
        id: "local-1".to_string(),
        name: "PowerShell".to_string(),
        shell: "powershell.exe".to_string(),
    });
    println!("\n=== LocalProfile JSON ===");
    println!("{}", serde_json::to_string_pretty(&local).unwrap());

    // Test 2: SshProfile with Password
    let ssh_pwd = SessionProfile::Ssh(SshProfile {
        id: "ssh-1".to_string(),
        name: "web-server".to_string(),
        host: "192.168.1.10".to_string(),
        port: 22,
        username: "user".to_string(),
        auth: SshAuth::Password,
        proxy_jump: None,
    });
    println!("\n=== SshProfile with Password (no proxy_jump) ===");
    println!("{}", serde_json::to_string_pretty(&ssh_pwd).unwrap());

    // Test 3: SshProfile with Key
    let ssh_key = SessionProfile::Ssh(SshProfile {
        id: "ssh-2".to_string(),
        name: "db-server".to_string(),
        host: "10.0.0.5".to_string(),
        port: 2222,
        username: "ubuntu".to_string(),
        auth: SshAuth::Key {
            path: "/home/user/.ssh/id_rsa".to_string(),
        },
        proxy_jump: Some("jump-host".to_string()),
    });
    println!("\n=== SshProfile with Key auth and proxy_jump ===");
    println!("{}", serde_json::to_string_pretty(&ssh_key).unwrap());
}
