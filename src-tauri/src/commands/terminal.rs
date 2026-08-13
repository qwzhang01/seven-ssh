use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::commands::connection::get_connection_with_secrets;
use crate::ssh::client::SshClient;
use crate::ssh::encoding;
use crate::ssh::session::{SessionCommand, SshSession};
use crate::state::AppState;

#[tauri::command]
pub async fn ssh_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    connection_id: String,
) -> Result<String, String> {
    let conn_info = get_connection_with_secrets(state.inner(), &connection_id).await?;

    let db_path = crate::db::Database::db_path()
        .map_err(|e| format!("Failed to get DB path: {}", e))?;

    let session_id = Uuid::new_v4().to_string();

    let (_handle, mut channel) = if let Some(ref jump_id) = conn_info.proxy_jump_id {
        let jump_info = get_connection_with_secrets(state.inner(), jump_id).await?;
        SshClient::connect_via_jump(&conn_info, &jump_info, db_path).await?
    } else {
        SshClient::connect(&conn_info, db_path).await?
    };

    channel
        .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
        .await
        .map_err(|e| format!("Failed to request PTY: {}", e))?;

    channel
        .request_shell(false)
        .await
        .map_err(|e| format!("Failed to request shell: {}", e))?;

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();

    let charset = conn_info.charset.clone();

    let session = SshSession::new(
        session_id.clone(),
        connection_id.clone(),
        cmd_tx,
        charset.clone(),
    );

    state.sessions.write().await.insert(session_id.clone(), session);

    let sid = session_id.clone();
    let app_handle = app.clone();
    let sessions_ref = state.sessions.clone();
    let session_charset = charset;

    tokio::spawn(async move {
        use russh::ChannelMsg;

        let mut log_file: Option<tokio::fs::File> = None;

        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { data }) => {
                            let raw_bytes = data.to_vec();
                            if let Some(f) = log_file.as_mut() {
                                let _ = f.write_all(&raw_bytes).await;
                            }
                            let output = encoding::decode_to_utf8(&raw_bytes, &session_charset);
                            let _ = app_handle.emit(
                                &format!("terminal-output-{}", sid),
                                output,
                            );
                        }
                        Some(ChannelMsg::ExtendedData { data, .. }) => {
                            let raw_bytes = data.to_vec();
                            if let Some(f) = log_file.as_mut() {
                                let _ = f.write_all(&raw_bytes).await;
                            }
                            let output = encoding::decode_to_utf8(&raw_bytes, &session_charset);
                            let _ = app_handle.emit(
                                &format!("terminal-output-{}", sid),
                                output,
                            );
                        }
                        Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                            let _ = app_handle.emit(
                                &format!("terminal-status-{}", sid),
                                "disconnected",
                            );
                            break;
                        }
                        _ => {}
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(SessionCommand::Write(data)) => {
                            let encoded = encoding::encode_from_utf8(&data, &session_charset);
                            if channel.data(&encoded[..]).await.is_err() {
                                break;
                            }
                        }
                        Some(SessionCommand::Resize { cols, rows }) => {
                            let _ = channel.window_change(cols, rows, 0, 0).await;
                        }
                        Some(SessionCommand::EnableLogging(path)) => {
                            match tokio::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&path)
                                .await
                            {
                                Ok(mut f) => {
                                    let header = format!(
                                        "\n--- Session log started: {} ---\n",
                                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                                    );
                                    let _ = f.write_all(header.as_bytes()).await;
                                    log_file = Some(f);
                                    let _ = app_handle.emit(
                                        &format!("terminal-status-{}", sid),
                                        "logging-enabled",
                                    );
                                }
                                Err(e) => {
                                    log::error!("Failed to open log file: {}", e);
                                }
                            }
                        }
                        Some(SessionCommand::DisableLogging) => {
                            if let Some(mut f) = log_file.take() {
                                let footer = format!(
                                    "\n--- Session log ended: {} ---\n",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                                );
                                let _ = f.write_all(footer.as_bytes()).await;
                                let _ = f.flush().await;
                            }
                            let _ = app_handle.emit(
                                &format!("terminal-status-{}", sid),
                                "logging-disabled",
                            );
                        }
                        Some(SessionCommand::Close) | None => {
                            if let Some(mut f) = log_file.take() {
                                let footer = format!(
                                    "\n--- Session ended: {} ---\n",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                                );
                                let _ = f.write_all(footer.as_bytes()).await;
                                let _ = f.flush().await;
                            }
                            let _ = channel.close().await;
                            break;
                        }
                    }
                }
            }
        }

        sessions_ref.write().await.remove(&sid);
    });

    // Record connection history
    {
        let db = state.db.conn();
        let history_id = Uuid::new_v4().to_string();
        let _ = db.execute(
            "INSERT INTO connection_history (id, connection_id) VALUES (?1, ?2)",
            rusqlite::params![history_id, connection_id],
        );
    }

    Ok(session_id)
}

#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        let _ = session.command_tx.send(SessionCommand::Close);
    }
    Ok(())
}

#[tauri::command]
pub async fn ssh_write(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        session.command_tx.send(SessionCommand::Write(data))
            .map_err(|_| "Session closed".to_string())?;
    } else {
        return Err("Session not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        let _ = session.command_tx.send(SessionCommand::Resize { cols, rows });
    }
    Ok(())
}

#[tauri::command]
pub async fn session_log_toggle(
    state: State<'_, AppState>,
    session_id: String,
    enable: bool,
    log_path: Option<String>,
) -> Result<(), String> {
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        if enable {
            let path = log_path
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    let dir = directories::ProjectDirs::from("com", "sevenssh", "SevenSSH")
                        .map(|d| d.data_dir().to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."));
                    let logs_dir = dir.join("logs");
                    let _ = std::fs::create_dir_all(&logs_dir);
                    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    logs_dir.join(format!("session_{}_{}.log", &session_id[..8], ts))
                });
            session.command_tx.send(SessionCommand::EnableLogging(path))
                .map_err(|_| "Session closed".to_string())?;
        } else {
            session.command_tx.send(SessionCommand::DisableLogging)
                .map_err(|_| "Session closed".to_string())?;
        }
    } else {
        return Err("Session not found".to_string());
    }
    Ok(())
}
