use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: String,
    pub details: Option<String>,
    pub timestamp: String,
}

pub fn ensure_audit_table(state: &AppState) {
    let conn = state.db.conn();
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS security_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            details TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    );
}

pub fn log_event_internal(state: &AppState, event_type: &str, details: Option<&str>) {
    let conn = state.db.conn();
    let id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO security_events (id, event_type, details) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, event_type, details],
    );
}

#[tauri::command]
pub async fn log_security_event(
    state: State<'_, AppState>,
    event_type: String,
    details: Option<String>,
) -> Result<(), String> {
    log_event_internal(state.inner(), &event_type, details.as_deref());
    Ok(())
}

#[tauri::command]
pub async fn get_security_events(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<SecurityEvent>, String> {
    let conn = state.db.conn();
    let limit_val = limit.unwrap_or(100);

    let mut stmt = conn
        .prepare(
            "SELECT id, event_type, details, timestamp FROM security_events ORDER BY timestamp DESC LIMIT ?1"
        )
        .map_err(|e| format!("Failed to query security events: {}", e))?;

    let events = stmt
        .query_map(rusqlite::params![limit_val], |row| {
            Ok(SecurityEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                details: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to read security events: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(events)
}

#[tauri::command]
pub async fn clear_clipboard() -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(b"")?;
                }
                child.wait()
            })
            .map_err(|e| format!("Failed to clear clipboard: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(b"")?;
                }
                child.wait()
            })
            .map_err(|e| format!("Failed to clear clipboard: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "echo.|clip"])
            .output()
            .map_err(|e| format!("Failed to clear clipboard: {}", e))?;
    }

    Ok(())
}
