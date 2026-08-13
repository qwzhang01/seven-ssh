use std::path::PathBuf;
use std::sync::Arc;

use russh::client::Msg;
use russh::Channel;
use russh_sftp::client::SftpSession;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::db::models::ConnectionInfo;
use crate::ssh::auth::{AuthCredential, ClientHandler};

#[derive(Debug, Serialize, Clone)]
pub struct RemoteFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: u32,
    pub modified: Option<u64>,
    pub owner: Option<u32>,
    pub group: Option<u32>,
}

#[allow(dead_code)]
pub struct SftpHandle {
    pub session_id: String,
    pub connection_id: String,
    pub host: String,
    pub sftp: Arc<Mutex<SftpSession>>,
    _handle: russh::client::Handle<ClientHandler>,
}

impl SftpHandle {
    pub async fn connect(
        session_id: String,
        conn_info: &ConnectionInfo,
        db_path: PathBuf,
    ) -> Result<Self, String> {
        let config = russh::client::Config {
            keepalive_interval: Some(std::time::Duration::from_secs(
                conn_info.keepalive_interval as u64,
            )),
            keepalive_max: 3,
            ..Default::default()
        };

        let handler = ClientHandler {
            db_path,
            host: conn_info.host.clone(),
            port: conn_info.port,
            ki_password: None,
        };

        let addr = format!("{}:{}", conn_info.host, conn_info.port);
        let mut handle =
            russh::client::connect(Arc::new(config), &addr, handler)
                .await
                .map_err(|e| format!("SFTP connection failed: {}", e))?;

        let credential = AuthCredential::from_connection(conn_info)?;
        match credential {
            AuthCredential::Password(password) => {
                let ok = handle
                    .authenticate_password(&conn_info.username, &password)
                    .await
                    .map_err(|e| format!("Auth failed: {}", e))?;
                if !ok {
                    return Err("Authentication failed".to_string());
                }
            }
            AuthCredential::PublicKey { key } => {
                let ok = handle
                    .authenticate_publickey(&conn_info.username, key)
                    .await
                    .map_err(|e| format!("Auth failed: {}", e))?;
                if !ok {
                    return Err("Authentication failed".to_string());
                }
            }
            AuthCredential::Agent => {
                crate::ssh::client::SshClient::authenticate_with_agent(
                    &mut handle,
                    &conn_info.username,
                )
                .await?;
            }
            AuthCredential::KeyboardInteractive { .. } => {
                return Err("Keyboard-interactive auth not supported for SFTP".to_string());
            }
        }

        let channel: Channel<Msg> = handle
            .channel_open_session()
            .await
            .map_err(|e| format!("Failed to open channel: {}", e))?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| format!("Failed to request SFTP subsystem: {}", e))?;

        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| format!("Failed to init SFTP session: {}", e))?;

        Ok(Self {
            session_id,
            connection_id: conn_info.id.clone(),
            host: conn_info.host.clone(),
            sftp: Arc::new(Mutex::new(sftp)),
            _handle: handle,
        })
    }

    pub async fn list_dir(&self, path: &str) -> Result<Vec<RemoteFileEntry>, String> {
        let sftp = self.sftp.lock().await;
        let entries = sftp
            .read_dir(path)
            .await
            .map_err(|e| format!("Failed to list directory: {}", e))?;

        let base_path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        let mut result: Vec<RemoteFileEntry> = entries
            .into_iter()
            .filter(|e| e.file_name() != "." && e.file_name() != "..")
            .map(|entry| {
                let name = entry.file_name();
                let attrs = entry.metadata();
                let is_dir = attrs.is_dir();
                let size = attrs.size.unwrap_or(0);
                let permissions = attrs.permissions.unwrap_or(0);
                let modified = attrs.mtime.map(|t| t as u64);
                let owner = attrs.uid;
                let group = attrs.gid;

                RemoteFileEntry {
                    path: format!("{}{}", base_path, name),
                    name,
                    is_dir,
                    size,
                    permissions,
                    modified,
                    owner,
                    group,
                }
            })
            .collect();

        result.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(result)
    }

    pub async fn mkdir(&self, path: &str) -> Result<(), String> {
        let sftp = self.sftp.lock().await;
        sftp.create_dir(path)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))
    }

    pub async fn remove_file(&self, path: &str) -> Result<(), String> {
        let sftp = self.sftp.lock().await;
        sftp.remove_file(path)
            .await
            .map_err(|e| format!("Failed to remove file: {}", e))
    }

    pub async fn remove_dir(&self, path: &str) -> Result<(), String> {
        let sftp = self.sftp.lock().await;
        sftp.remove_dir(path)
            .await
            .map_err(|e| format!("Failed to remove directory: {}", e))
    }

    pub async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), String> {
        let sftp = self.sftp.lock().await;
        sftp.rename(old_path, new_path)
            .await
            .map_err(|e| format!("Failed to rename: {}", e))
    }

    pub async fn realpath(&self, path: &str) -> Result<String, String> {
        let sftp = self.sftp.lock().await;
        sftp.canonicalize(path)
            .await
            .map_err(|e| format!("Failed to resolve path: {}", e))
    }
}
