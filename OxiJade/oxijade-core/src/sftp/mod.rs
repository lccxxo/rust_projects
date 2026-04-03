// oxijade-core/src/sftp/mod.rs
use anyhow::{bail, Context};
use async_trait::async_trait;
use russh::client;
use russh::keys::key::PublicKey;
use russh_sftp::client::SftpSession;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 单个远程目录条目
#[derive(Debug, Clone)]
pub struct SftpEntry {
    pub name: String,
    pub full_path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// SFTP 操作请求（发给后台任务）
#[derive(Debug)]
pub enum SftpRequest {
    ListDir(String),
    Download { remote: String, local: std::path::PathBuf },
    Upload { local: std::path::PathBuf, remote: String },
    Disconnect,
}

/// SFTP 操作结果（后台任务回传给 UI）
#[derive(Debug)]
pub enum SftpResponse {
    DirListing { path: String, entries: Vec<SftpEntry> },
    DownloadDone { local: std::path::PathBuf },
    UploadDone,
    Error(String),
}

struct AcceptAllKeys;

#[async_trait]
impl client::Handler for AcceptAllKeys {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true) // MVP：信任所有主机密钥
    }
}

/// 建立 SFTP 连接（密钥认证）
pub async fn connect_key(
    host: &str,
    port: u16,
    username: &str,
    key_path: &str,
) -> anyhow::Result<SftpSession> {
    let config = Arc::new(client::Config::default());
    let addr = format!("{host}:{port}");
    let mut handle = client::connect(config, addr, AcceptAllKeys)
        .await
        .context("TCP 连接失败")?;

    let key = russh::keys::load_secret_key(key_path, None)
        .context("无法加载私钥")?;

    let auth_ok = handle
        .authenticate_publickey(username, Arc::new(key))
        .await
        .context("密钥认证失败")?;

    if !auth_ok {
        bail!("密钥认证被服务器拒绝");
    }

    open_sftp_session(&mut handle).await
}

/// 建立 SFTP 连接（密码认证）
pub async fn connect_password(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> anyhow::Result<SftpSession> {
    let config = Arc::new(client::Config::default());
    let addr = format!("{host}:{port}");
    let mut handle = client::connect(config, addr, AcceptAllKeys)
        .await
        .context("TCP 连接失败")?;

    let auth_ok = handle
        .authenticate_password(username, password)
        .await
        .context("密码认证失败")?;

    if !auth_ok {
        bail!("密码认证被服务器拒绝");
    }

    open_sftp_session(&mut handle).await
}

async fn open_sftp_session(
    handle: &mut client::Handle<AcceptAllKeys>,
) -> anyhow::Result<SftpSession> {
    let channel = handle
        .channel_open_session()
        .await
        .context("无法打开 channel")?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .context("无法请求 sftp subsystem")?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .context("SFTP 握手失败")?;
    Ok(sftp)
}

/// 列出目录
pub async fn list_dir(sftp: &SftpSession, path: &str) -> anyhow::Result<Vec<SftpEntry>> {
    let read_dir = sftp.read_dir(path).await.context("read_dir 失败")?;
    let mut result: Vec<SftpEntry> = read_dir
        .into_iter()
        .map(|entry| {
            let name = entry.file_name();
            let is_dir = entry.file_type().is_dir();
            let size = entry.metadata().len();
            let full_path = format!("{}/{}", path.trim_end_matches('/'), &name);
            SftpEntry {
                name,
                full_path,
                is_dir,
                size,
            }
        })
        .collect();
    result.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(result)
}

/// 下载文件
pub async fn download(
    sftp: &SftpSession,
    remote: &str,
    local: &std::path::Path,
) -> anyhow::Result<()> {
    let mut remote_file = sftp.open(remote).await.context("打开远程文件失败")?;
    let mut local_file =
        tokio::fs::File::create(local).await.context("创建本地文件失败")?;
    let mut buf = vec![0u8; 65536];
    loop {
        let n = remote_file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        local_file.write_all(&buf[..n]).await?;
    }
    Ok(())
}

/// 上传文件
pub async fn upload(
    sftp: &SftpSession,
    local: &std::path::Path,
    remote: &str,
) -> anyhow::Result<()> {
    let mut local_file = tokio::fs::File::open(local).await.context("打开本地文件失败")?;
    let mut remote_file = sftp.create(remote).await.context("创建远程文件失败")?;
    let mut buf = vec![0u8; 65536];
    loop {
        let n = local_file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        remote_file.write_all(&buf[..n]).await?;
    }
    Ok(())
}
