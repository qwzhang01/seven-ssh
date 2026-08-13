use std::fs;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::models::ConnectionInfo;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    pub private_key_path: Option<String>,
    pub group_id: Option<String>,
    pub tags: String,
    pub color: Option<String>,
    pub charset: String,
    pub keepalive_interval: u32,
    pub startup_command: Option<String>,
    pub proxy_jump_id: Option<String>,
    pub sort_order: i32,
    pub is_favorite: bool,
    pub note: Option<String>,
}

impl From<&ConnectionInfo> for ExportedConnection {
    fn from(c: &ConnectionInfo) -> Self {
        Self {
            name: c.name.clone(),
            host: c.host.clone(),
            port: c.port,
            username: c.username.clone(),
            auth_method: c.auth_method.clone(),
            private_key_path: c.private_key_path.clone(),
            group_id: c.group_id.clone(),
            tags: c.tags.clone(),
            color: c.color.clone(),
            charset: c.charset.clone(),
            keepalive_interval: c.keepalive_interval,
            startup_command: c.startup_command.clone(),
            proxy_jump_id: c.proxy_jump_id.clone(),
            sort_order: c.sort_order,
            is_favorite: c.is_favorite,
            note: c.note.clone(),
        }
    }
}

fn fetch_connections(state: &AppState, connection_ids: &Option<Vec<String>>) -> Result<Vec<ConnectionInfo>, String> {
    let conn = state.db.conn();

    let connections: Vec<ConnectionInfo> = if let Some(ref ids) = connection_ids {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let sql = format!(
            "SELECT id, name, host, port, username, auth_method, password,
                    private_key_path, passphrase, group_id, tags, color, charset,
                    keepalive_interval, startup_command, proxy_jump_id, sort_order,
                    is_favorite, note, created_at, updated_at
             FROM connections WHERE id IN ({}) ORDER BY sort_order, name",
            placeholders.join(", ")
        );
        let params: Vec<Box<dyn rusqlite::types::ToSql>> = ids.iter().map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>).collect();
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(ConnectionInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, u32>(3)? as u16,
                username: row.get(4)?,
                auth_method: row.get(5)?,
                password: row.get(6)?,
                private_key_path: row.get(7)?,
                passphrase: row.get(8)?,
                group_id: row.get(9)?,
                tags: row.get::<_, String>(10).unwrap_or_else(|_| "[]".to_string()),
                color: row.get(11)?,
                charset: row.get::<_, String>(12).unwrap_or_else(|_| "UTF-8".to_string()),
                keepalive_interval: row.get::<_, u32>(13).unwrap_or(60),
                startup_command: row.get(14)?,
                proxy_jump_id: row.get(15)?,
                sort_order: row.get::<_, i32>(16).unwrap_or(0),
                is_favorite: row.get::<_, bool>(17).unwrap_or(false),
                note: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, auth_method, password,
                    private_key_path, passphrase, group_id, tags, color, charset,
                    keepalive_interval, startup_command, proxy_jump_id, sort_order,
                    is_favorite, note, created_at, updated_at
             FROM connections ORDER BY sort_order, name"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            Ok(ConnectionInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, u32>(3)? as u16,
                username: row.get(4)?,
                auth_method: row.get(5)?,
                password: row.get(6)?,
                private_key_path: row.get(7)?,
                passphrase: row.get(8)?,
                group_id: row.get(9)?,
                tags: row.get::<_, String>(10).unwrap_or_else(|_| "[]".to_string()),
                color: row.get(11)?,
                charset: row.get::<_, String>(12).unwrap_or_else(|_| "UTF-8".to_string()),
                keepalive_interval: row.get::<_, u32>(13).unwrap_or(60),
                startup_command: row.get(14)?,
                proxy_jump_id: row.get(15)?,
                sort_order: row.get::<_, i32>(16).unwrap_or(0),
                is_favorite: row.get::<_, bool>(17).unwrap_or(false),
                note: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    Ok(connections)
}

fn to_json_string(connections: &[ConnectionInfo]) -> Result<String, String> {
    let exported: Vec<ExportedConnection> = connections.iter().map(ExportedConnection::from).collect();
    serde_json::to_string_pretty(&exported)
        .map_err(|e| format!("Failed to serialize connections: {}", e))
}

fn to_ssh_config_string(connections: &[ConnectionInfo]) -> String {
    let mut output = String::new();
    output.push_str("# Generated by SevenSSH\n\n");

    for conn in connections {
        output.push_str(&format!("Host {}\n", conn.name.replace(' ', "-")));
        output.push_str(&format!("    HostName {}\n", conn.host));
        if conn.port != 22 {
            output.push_str(&format!("    Port {}\n", conn.port));
        }
        output.push_str(&format!("    User {}\n", conn.username));
        if let Some(ref key_path) = conn.private_key_path {
            if !key_path.is_empty() {
                output.push_str(&format!("    IdentityFile {}\n", key_path));
            }
        }
        if conn.keepalive_interval > 0 {
            output.push_str(&format!("    ServerAliveInterval {}\n", conn.keepalive_interval));
        }
        output.push('\n');
    }

    output
}

#[tauri::command]
pub fn export_connections(
    state: State<AppState>,
    format: String,
    connection_ids: Option<Vec<String>>,
) -> Result<String, String> {
    let connections = fetch_connections(&state, &connection_ids)?;

    match format.as_str() {
        "json" => to_json_string(&connections),
        "ssh_config" => Ok(to_ssh_config_string(&connections)),
        _ => Err(format!("Unsupported export format: '{}'. Use 'json' or 'ssh_config'.", format)),
    }
}

#[tauri::command]
pub fn export_connections_to_file(
    state: State<AppState>,
    format: String,
    path: String,
    connection_ids: Option<Vec<String>>,
) -> Result<(), String> {
    let content = export_connections(state, format, connection_ids)?;
    fs::write(&path, &content)
        .map_err(|e| format!("Failed to write export file '{}': {}", path, e))?;
    Ok(())
}
