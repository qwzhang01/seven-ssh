use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::commands::connection::get_connection_with_secrets;
use crate::sftp::session::{RemoteFileEntry, SftpHandle};
use crate::state::AppState;

#[tauri::command]
pub async fn sftp_open(
    state: State<'_, AppState>,
    connection_id: String,
) -> Result<String, String> {
    let conn_info = get_connection_with_secrets(state.inner(), &connection_id).await?;

    let db_path = crate::db::Database::db_path()
        .map_err(|e| format!("Failed to get DB path: {}", e))?;

    let session_id = Uuid::new_v4().to_string();
    let handle = SftpHandle::connect(session_id.clone(), &conn_info, db_path).await?;
    state.sftp_sessions.write().await.insert(session_id.clone(), handle);

    Ok(session_id)
}

#[tauri::command]
pub async fn sftp_close(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.sftp_sessions.write().await.remove(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn sftp_list_dir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<Vec<RemoteFileEntry>, String> {
    let sessions = state.sftp_sessions.read().await;
    let handle = sessions
        .get(&session_id)
        .ok_or("SFTP session not found")?;
    handle.list_dir(&path).await
}

#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    let sessions = state.sftp_sessions.read().await;
    let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
    handle.mkdir(&path).await
}

#[tauri::command]
pub async fn sftp_remove(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
    is_dir: bool,
) -> Result<(), String> {
    let sessions = state.sftp_sessions.read().await;
    let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
    if is_dir {
        handle.remove_dir(&path).await
    } else {
        handle.remove_file(&path).await
    }
}

#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let sessions = state.sftp_sessions.read().await;
    let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
    handle.rename(&old_path, &new_path).await
}

#[tauri::command]
pub async fn sftp_realpath(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<String, String> {
    let sessions = state.sftp_sessions.read().await;
    let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
    handle.realpath(&path).await
}

#[tauri::command]
pub async fn sftp_upload(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    local_path: String,
    remote_path: String,
    transfer_id: String,
) -> Result<(), String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };

    let metadata = tokio::fs::metadata(&local_path)
        .await
        .map_err(|e| format!("Cannot read local file: {}", e))?;
    let total_size = metadata.len();

    let local_data = tokio::fs::read(&local_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let sftp_guard = sftp.lock().await;

    const CHUNK_SIZE: usize = 32768;
    let mut offset: usize = 0;

    let mut file = sftp_guard
        .create(&remote_path)
        .await
        .map_err(|e| format!("Failed to create remote file: {}", e))?;

    use tokio::io::AsyncWriteExt;
    while offset < local_data.len() {
        let end = std::cmp::min(offset + CHUNK_SIZE, local_data.len());
        file.write_all(&local_data[offset..end])
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        offset = end;

        let progress = (offset as f64 / total_size as f64 * 100.0).min(100.0);
        let _ = app.emit(
            &format!("transfer-progress-{}", transfer_id),
            serde_json::json!({
                "transfer_id": transfer_id,
                "bytes_transferred": offset,
                "total_bytes": total_size,
                "progress": progress,
            }),
        );
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush failed: {}", e))?;
    file.shutdown()
        .await
        .map_err(|e| format!("Close failed: {}", e))?;

    let _ = app.emit(
        &format!("transfer-complete-{}", transfer_id),
        serde_json::json!({ "transfer_id": transfer_id, "status": "completed" }),
    );

    Ok(())
}

#[tauri::command]
pub async fn sftp_download(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
    local_path: String,
    transfer_id: String,
) -> Result<(), String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };

    let sftp_guard = sftp.lock().await;

    let metadata = sftp_guard
        .metadata(&remote_path)
        .await
        .map_err(|e| format!("Failed to stat remote file: {}", e))?;
    let total_size = metadata.size.unwrap_or(0);

    use tokio::io::AsyncReadExt;
    let mut file = sftp_guard
        .open(&remote_path)
        .await
        .map_err(|e| format!("Failed to open remote file: {}", e))?;

    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 32768];
    let mut bytes_read: u64 = 0;

    loop {
        let n = file
            .read(&mut chunk)
            .await
            .map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
        bytes_read += n as u64;

        if total_size > 0 {
            let progress = (bytes_read as f64 / total_size as f64 * 100.0).min(100.0);
            let _ = app.emit(
                &format!("transfer-progress-{}", transfer_id),
                serde_json::json!({
                    "transfer_id": transfer_id,
                    "bytes_transferred": bytes_read,
                    "total_bytes": total_size,
                    "progress": progress,
                }),
            );
        }
    }

    drop(sftp_guard);

    tokio::fs::write(&local_path, &buffer)
        .await
        .map_err(|e| format!("Failed to write local file: {}", e))?;

    let _ = app.emit(
        &format!("transfer-complete-{}", transfer_id),
        serde_json::json!({ "transfer_id": transfer_id, "status": "completed" }),
    );

    Ok(())
}

#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<serde_json::Value, String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };
    let sftp_guard = sftp.lock().await;
    let metadata = sftp_guard
        .metadata(&path)
        .await
        .map_err(|e| format!("Failed to stat: {}", e))?;

    Ok(serde_json::json!({
        "size": metadata.size.unwrap_or(0),
        "is_dir": metadata.is_dir(),
        "permissions": metadata.permissions.unwrap_or(0),
        "modified": metadata.mtime.map(|t| t as u64),
    }))
}

#[tauri::command]
pub async fn sftp_read_file(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<String, String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };
    let sftp_guard = sftp.lock().await;

    use tokio::io::AsyncReadExt;
    let mut file = sftp_guard
        .open(&path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut content = Vec::new();
    let mut chunk = vec![0u8; 32768];
    loop {
        let n = file.read(&mut chunk).await.map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 { break; }
        content.extend_from_slice(&chunk[..n]);
    }

    String::from_utf8(content).map_err(|e| format!("File is not valid UTF-8: {}", e))
}

#[tauri::command]
pub async fn sftp_write_file(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };
    let sftp_guard = sftp.lock().await;

    use tokio::io::AsyncWriteExt;
    let mut file = sftp_guard
        .create(&path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(content.as_bytes())
        .await
        .map_err(|e| format!("Write failed: {}", e))?;
    file.flush().await.map_err(|e| format!("Flush failed: {}", e))?;
    file.shutdown().await.map_err(|e| format!("Close failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn sftp_upload_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    local_path: String,
    remote_path: String,
    transfer_id: String,
    offset: u64,
) -> Result<(), String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };

    let local_data = tokio::fs::read(&local_path)
        .await
        .map_err(|e| format!("Failed to read local file: {}", e))?;
    let total_size = local_data.len() as u64;
    let start = offset as usize;
    if start >= local_data.len() {
        return Ok(());
    }

    let sftp_guard = sftp.lock().await;

    use tokio::io::{AsyncWriteExt, AsyncSeekExt};
    let mut file = sftp_guard
        .open(&remote_path)
        .await
        .map_err(|_| "Remote file not found for resume".to_string())?;

    file.seek(std::io::SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("Seek failed: {}", e))?;

    const CHUNK_SIZE: usize = 32768;
    let mut pos = start;
    while pos < local_data.len() {
        let end = std::cmp::min(pos + CHUNK_SIZE, local_data.len());
        file.write_all(&local_data[pos..end])
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        pos = end;

        let progress = (pos as f64 / total_size as f64 * 100.0).min(100.0);
        let _ = app.emit(
            &format!("transfer-progress-{}", transfer_id),
            serde_json::json!({
                "transfer_id": transfer_id,
                "bytes_transferred": pos,
                "total_bytes": total_size,
                "progress": progress,
            }),
        );
    }

    file.flush().await.map_err(|e| format!("Flush failed: {}", e))?;
    file.shutdown().await.map_err(|e| format!("Close failed: {}", e))?;

    let _ = app.emit(
        &format!("transfer-complete-{}", transfer_id),
        serde_json::json!({ "transfer_id": transfer_id, "status": "completed" }),
    );
    Ok(())
}

#[tauri::command]
pub async fn sftp_download_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
    local_path: String,
    transfer_id: String,
    offset: u64,
) -> Result<(), String> {
    let sftp = {
        let sessions = state.sftp_sessions.read().await;
        let handle = sessions.get(&session_id).ok_or("SFTP session not found")?;
        handle.sftp.clone()
    };
    let sftp_guard = sftp.lock().await;

    let metadata = sftp_guard
        .metadata(&remote_path)
        .await
        .map_err(|e| format!("Failed to stat remote file: {}", e))?;
    let total_size = metadata.size.unwrap_or(0);

    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    let mut file = sftp_guard
        .open(&remote_path)
        .await
        .map_err(|e| format!("Failed to open remote file: {}", e))?;

    if offset > 0 {
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|e| format!("Seek failed: {}", e))?;
    }

    let existing_data = if offset > 0 {
        tokio::fs::read(&local_path).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut buffer = existing_data;
    let mut chunk = vec![0u8; 32768];
    let mut bytes_read = offset;

    loop {
        let n = file.read(&mut chunk).await.map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 { break; }
        buffer.extend_from_slice(&chunk[..n]);
        bytes_read += n as u64;

        if total_size > 0 {
            let progress = (bytes_read as f64 / total_size as f64 * 100.0).min(100.0);
            let _ = app.emit(
                &format!("transfer-progress-{}", transfer_id),
                serde_json::json!({
                    "transfer_id": transfer_id,
                    "bytes_transferred": bytes_read,
                    "total_bytes": total_size,
                    "progress": progress,
                }),
            );
        }
    }
    drop(sftp_guard);

    tokio::fs::write(&local_path, &buffer)
        .await
        .map_err(|e| format!("Failed to write local file: {}", e))?;

    let _ = app.emit(
        &format!("transfer-complete-{}", transfer_id),
        serde_json::json!({ "transfer_id": transfer_id, "status": "completed" }),
    );
    Ok(())
}
