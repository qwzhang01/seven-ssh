use std::path::PathBuf;
use std::sync::Arc;

use russh::*;
use russh::client::Msg;

use super::auth::{AuthCredential, ClientHandler};
use crate::db::models::ConnectionInfo;

pub struct SshClient;

impl SshClient {
    pub async fn connect(
        conn_info: &ConnectionInfo,
        db_path: PathBuf,
    ) -> Result<(client::Handle<ClientHandler>, Channel<Msg>), String> {
        let (handle, _) = Self::connect_inner(conn_info, &db_path, None).await?;

        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| format!("Failed to open channel: {}", e))?;

        Ok((handle, channel))
    }

    /// Connect through a jump host, returning a handle and session channel on the target.
    pub async fn connect_via_jump(
        target_info: &ConnectionInfo,
        jump_info: &ConnectionInfo,
        db_path: PathBuf,
    ) -> Result<(client::Handle<ClientHandler>, Channel<Msg>), String> {
        let (jump_handle, _) = Self::connect_inner(jump_info, &db_path, None).await?;

        let tunnel = jump_handle
            .channel_open_direct_tcpip(
                &target_info.host,
                target_info.port as u32,
                "127.0.0.1",
                0,
            )
            .await
            .map_err(|e| format!("Failed to open tunnel through jump host: {}", e))?;

        let (target_handle, _) =
            Self::connect_inner(target_info, &db_path, Some(tunnel)).await?;

        let channel = target_handle
            .channel_open_session()
            .await
            .map_err(|e| format!("Failed to open channel on target: {}", e))?;

        Ok((target_handle, channel))
    }

    async fn connect_inner(
        conn_info: &ConnectionInfo,
        db_path: &PathBuf,
        _tunnel: Option<Channel<Msg>>,
    ) -> Result<(client::Handle<ClientHandler>, ()), String> {
        let ki_password = match conn_info.auth_method.as_str() {
            "keyboard-interactive" => conn_info.password.clone(),
            _ => None,
        };

        let config = client::Config {
            keepalive_interval: Some(std::time::Duration::from_secs(
                conn_info.keepalive_interval as u64,
            )),
            keepalive_max: 3,
            ..Default::default()
        };

        let handler = ClientHandler {
            db_path: db_path.clone(),
            host: conn_info.host.clone(),
            port: conn_info.port,
            ki_password,
        };

        let addr = format!("{}:{}", conn_info.host, conn_info.port);
        let mut handle = client::connect(Arc::new(config), &addr, handler)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let credential = AuthCredential::from_connection(conn_info)?;
        match credential {
            AuthCredential::Password(password) => {
                let auth_result = handle
                    .authenticate_password(&conn_info.username, &password)
                    .await
                    .map_err(|e| format!("Password auth failed: {}", e))?;
                if !auth_result {
                    return Err("Authentication failed: invalid credentials".to_string());
                }
            }
            AuthCredential::PublicKey { key } => {
                let auth_result = handle
                    .authenticate_publickey(&conn_info.username, key)
                    .await
                    .map_err(|e| format!("Public key auth failed: {}", e))?;
                if !auth_result {
                    return Err("Authentication failed: key rejected".to_string());
                }
            }
            AuthCredential::Agent => {
                Self::authenticate_with_agent(&mut handle, &conn_info.username).await?;
            }
            AuthCredential::KeyboardInteractive { password } => {
                use russh::client::KeyboardInteractiveAuthResponse;

                let resp = handle
                    .authenticate_keyboard_interactive_start(&conn_info.username, None)
                    .await
                    .map_err(|e| format!("Keyboard-interactive auth failed: {}", e))?;

                match resp {
                    KeyboardInteractiveAuthResponse::Success => {}
                    KeyboardInteractiveAuthResponse::Failure => {
                        return Err(
                            "Authentication failed: keyboard-interactive rejected".to_string()
                        );
                    }
                    KeyboardInteractiveAuthResponse::InfoRequest { prompts, .. } => {
                        let responses: Vec<String> = prompts
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                if i == 0 {
                                    password.clone().unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            })
                            .collect();

                        let final_resp = handle
                            .authenticate_keyboard_interactive_respond(responses)
                            .await
                            .map_err(|e| {
                                format!("Keyboard-interactive respond failed: {}", e)
                            })?;

                        match final_resp {
                            KeyboardInteractiveAuthResponse::Success => {}
                            _ => {
                                return Err(
                                    "Authentication failed: keyboard-interactive rejected"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok((handle, ()))
    }

    pub async fn authenticate_with_agent(
        handle: &mut client::Handle<ClientHandler>,
        username: &str,
    ) -> Result<(), String> {
        let ssh_auth_sock = std::env::var("SSH_AUTH_SOCK")
            .map_err(|_| "SSH_AUTH_SOCK not set — is ssh-agent running?".to_string())?;

        let stream = tokio::net::UnixStream::connect(&ssh_auth_sock)
            .await
            .map_err(|e| format!("Failed to connect to ssh-agent at {}: {}", ssh_auth_sock, e))?;

        let mut agent = russh_keys::agent::client::AgentClient::connect(stream);

        let identities = agent
            .request_identities()
            .await
            .map_err(|e| format!("Failed to list agent keys: {}", e))?;

        if identities.is_empty() {
            return Err("No keys available in ssh-agent".to_string());
        }

        for key in identities {
            let auth_result = handle
                .authenticate_publickey_with(username, key, &mut agent)
                .await;

            match auth_result {
                Ok(true) => return Ok(()),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }

        Err("Authentication failed: no agent key accepted by server".to_string())
    }
}
