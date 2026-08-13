use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use russh::client;
use russh_keys::load_secret_key;
use ssh_key::PrivateKey;

use crate::db::models::ConnectionInfo;
use super::known_hosts::{self, HostKeyStatus};

pub enum AuthCredential {
    Password(String),
    PublicKey {
        key: Arc<PrivateKey>,
    },
    Agent,
    KeyboardInteractive {
        password: Option<String>,
    },
}

impl AuthCredential {
    pub fn from_connection(conn: &ConnectionInfo) -> Result<Self, String> {
        match conn.auth_method.as_str() {
            "password" => {
                let password = conn.password.as_ref()
                    .ok_or("Password required for password authentication")?;
                Ok(AuthCredential::Password(password.clone()))
            }
            "publickey" => {
                let key_path = conn.private_key_path.as_ref()
                    .ok_or("Private key path required for public key authentication")?;
                let path = Path::new(key_path);
                if !path.exists() {
                    return Err(format!("Private key file not found: {}", key_path));
                }
                let passphrase = conn.passphrase.as_deref();
                let key = load_secret_key(path, passphrase)
                    .map_err(|e| format!("Failed to load private key: {}", e))?;
                Ok(AuthCredential::PublicKey { key: Arc::new(key) })
            }
            "agent" => Ok(AuthCredential::Agent),
            "keyboard-interactive" => {
                Ok(AuthCredential::KeyboardInteractive {
                    password: conn.password.clone(),
                })
            }
            other => Err(format!("Unsupported auth method: {}", other)),
        }
    }
}

pub struct ClientHandler {
    pub db_path: PathBuf,
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub ki_password: Option<String>,
}

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        match known_hosts::check_host_key(&self.db_path, &self.host, self.port, server_public_key) {
            Ok(HostKeyStatus::Trusted) => {
                log::debug!("Host key trusted for {}:{}", self.host, self.port);
                Ok(true)
            }
            Ok(HostKeyStatus::NewKey) => {
                log::info!(
                    "TOFU: accepting new host key for {}:{}",
                    self.host, self.port
                );
                if let Err(e) = known_hosts::save_host_key(
                    &self.db_path, &self.host, self.port, server_public_key,
                ) {
                    log::error!("Failed to save host key: {}", e);
                }
                Ok(true)
            }
            Ok(HostKeyStatus::Changed { old_fingerprint }) => {
                log::error!(
                    "HOST KEY CHANGED for {}:{} ! Old fingerprint: {}. Connection rejected.",
                    self.host, self.port, old_fingerprint
                );
                Ok(false)
            }
            Err(e) => {
                log::error!("Error checking host key: {}. Allowing connection.", e);
                Ok(true)
            }
        }
    }

    async fn auth_banner(
        &mut self,
        _banner: &str,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
