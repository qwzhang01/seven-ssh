use tauri::State;
use uuid::Uuid;

use crate::crypto::{self, ENC_PREFIX};
use crate::db::models::*;
use crate::state::AppState;

pub async fn get_connection_with_secrets(state: &AppState, id: &str) -> Result<ConnectionInfo, String> {
    let mut conn_info = {
        let db = state.db.conn();
        db.query_row(
            "SELECT id, name, host, port, username, auth_method, password,
                    private_key_path, passphrase, group_id, tags, color, charset,
                    keepalive_interval, startup_command, proxy_jump_id, sort_order,
                    is_favorite, note, created_at, updated_at
             FROM connections WHERE id = ?1",
            [id],
            |row| {
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
            },
        )
        .map_err(|e| format!("Connection not found: {}", e))?
    };

    let master_key = state.master_key.read().await;
    if let Some(ref key) = *master_key {
        decrypt_connection_secrets(&mut conn_info, key);
    }

    Ok(conn_info)
}

fn decrypt_connection_secrets(conn: &mut ConnectionInfo, master_key: &[u8; 32]) {
    if let Some(ref pwd) = conn.password {
        if let Some(stripped) = pwd.strip_prefix(ENC_PREFIX) {
            if let Ok(decrypted) = crypto::decrypt_field(stripped, master_key) {
                conn.password = Some(decrypted);
            }
        }
    }
    if let Some(ref pp) = conn.passphrase {
        if let Some(stripped) = pp.strip_prefix(ENC_PREFIX) {
            if let Ok(decrypted) = crypto::decrypt_field(stripped, master_key) {
                conn.passphrase = Some(decrypted);
            }
        }
    }
}

fn encrypt_optional_field(
    value: &Option<String>,
    master_key: &[u8; 32],
) -> Result<Option<String>, String> {
    match value {
        Some(ref v) if !v.is_empty() => {
            let encrypted = crypto::encrypt_field(v, master_key)?;
            Ok(Some(format!("{}{}", ENC_PREFIX, encrypted)))
        }
        other => Ok(other.clone()),
    }
}

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> Result<Vec<ConnectionInfo>, String> {
    let conn = state.db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, host, port, username, auth_method, password,
                    private_key_path, passphrase, group_id, tags, color, charset,
                    keepalive_interval, startup_command, proxy_jump_id, sort_order,
                    is_favorite, note, created_at, updated_at
             FROM connections ORDER BY sort_order, name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| e.to_string())?;

    let mut connections = Vec::new();
    for row in rows {
        connections.push(row.map_err(|e| e.to_string())?);
    }
    Ok(connections)
}

#[tauri::command]
pub fn get_connection(state: State<AppState>, id: String) -> Result<ConnectionInfo, String> {
    let conn = state.db.conn();
    conn.query_row(
        "SELECT id, name, host, port, username, auth_method, password,
                private_key_path, passphrase, group_id, tags, color, charset,
                keepalive_interval, startup_command, proxy_jump_id, sort_order,
                is_favorite, note, created_at, updated_at
         FROM connections WHERE id = ?1",
        [&id],
        |row| {
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
        },
    )
    .map_err(|e| format!("Connection not found: {}", e))
}

#[tauri::command]
pub async fn create_connection(
    state: State<'_, AppState>,
    request: CreateConnectionRequest,
) -> Result<ConnectionInfo, String> {
    let id = Uuid::new_v4().to_string();
    let tags = serde_json::to_string(&request.tags.unwrap_or_default())
        .unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();

    let (store_password, store_passphrase) = {
        let master_key = state.master_key.read().await;
        if let Some(ref key) = *master_key {
            let pwd = encrypt_optional_field(&request.password, key)?;
            let pp = encrypt_optional_field(&request.passphrase, key)?;
            (pwd, pp)
        } else {
            (request.password.clone(), request.passphrase.clone())
        }
    };

    {
        let conn = state.db.conn();
        conn.execute(
            "INSERT INTO connections (id, name, host, port, username, auth_method, password,
             private_key_path, passphrase, group_id, tags, color, charset, keepalive_interval,
             startup_command, proxy_jump_id, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            rusqlite::params![
                id,
                request.name,
                request.host,
                request.port.unwrap_or(22) as u32,
                request.username,
                request.auth_method,
                store_password,
                request.private_key_path,
                store_passphrase,
                request.group_id,
                tags,
                request.color,
                request.charset.unwrap_or_else(|| "UTF-8".to_string()),
                request.keepalive_interval.unwrap_or(60),
                request.startup_command,
                request.proxy_jump_id,
                request.note,
                now,
                now,
            ],
        )
        .map_err(|e| format!("Failed to create connection: {}", e))?;
    }

    get_connection(state, id)
}

#[tauri::command]
pub async fn update_connection(
    state: State<'_, AppState>,
    request: UpdateConnectionRequest,
) -> Result<ConnectionInfo, String> {
    let now = chrono::Utc::now().to_rfc3339();

    let (enc_password, enc_passphrase) = {
        let master_key = state.master_key.read().await;
        if let Some(ref key) = *master_key {
            let pwd = encrypt_optional_field(&request.password, key)?;
            let pp = encrypt_optional_field(&request.passphrase, key)?;
            (pwd, pp)
        } else {
            (request.password.clone(), request.passphrase.clone())
        }
    };

    {
        let conn = state.db.conn();

        let mut sets = vec!["updated_at = ?1".to_string()];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2u32;

        macro_rules! maybe_set {
            ($field:expr, $col:expr) => {
                if let Some(ref v) = $field {
                    sets.push(format!("{} = ?{}", $col, idx));
                    params.push(Box::new(v.clone()));
                    idx += 1;
                }
            };
        }

        maybe_set!(request.name, "name");
        maybe_set!(request.host, "host");
        maybe_set!(request.username, "username");
        maybe_set!(request.auth_method, "auth_method");
        maybe_set!(enc_password, "password");
        maybe_set!(request.private_key_path, "private_key_path");
        maybe_set!(enc_passphrase, "passphrase");
        maybe_set!(request.group_id, "group_id");
        maybe_set!(request.color, "color");
        maybe_set!(request.charset, "charset");
        maybe_set!(request.startup_command, "startup_command");
        maybe_set!(request.proxy_jump_id, "proxy_jump_id");
        maybe_set!(request.note, "note");

        if let Some(port) = request.port {
            sets.push(format!("port = ?{}", idx));
            params.push(Box::new(port as u32));
            idx += 1;
        }
        if let Some(fav) = request.is_favorite {
            sets.push(format!("is_favorite = ?{}", idx));
            params.push(Box::new(fav));
            idx += 1;
        }
        if let Some(ka) = request.keepalive_interval {
            sets.push(format!("keepalive_interval = ?{}", idx));
            params.push(Box::new(ka));
            idx += 1;
        }
        if let Some(so) = request.sort_order {
            sets.push(format!("sort_order = ?{}", idx));
            params.push(Box::new(so));
            idx += 1;
        }
        if let Some(ref tags) = request.tags {
            let json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
            sets.push(format!("tags = ?{}", idx));
            params.push(Box::new(json));
            idx += 1;
        }

        let sql = format!(
            "UPDATE connections SET {} WHERE id = ?{}",
            sets.join(", "),
            idx
        );
        params.push(Box::new(request.id.clone()));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| format!("Failed to update connection: {}", e))?;
    }

    get_connection(state, request.id)
}

#[tauri::command]
pub fn delete_connection(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.conn();
    conn.execute("DELETE FROM connections WHERE id = ?1", [&id])
        .map_err(|e| format!("Failed to delete connection: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn list_groups(state: State<AppState>) -> Result<Vec<GroupInfo>, String> {
    let conn = state.db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, sort_order, color, icon, created_at, updated_at
             FROM groups ORDER BY sort_order, name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(GroupInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get::<_, i32>(3).unwrap_or(0),
                color: row.get(4)?,
                icon: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row.map_err(|e| e.to_string())?);
    }
    Ok(groups)
}

#[tauri::command]
pub fn create_group(
    state: State<AppState>,
    request: CreateGroupRequest,
) -> Result<GroupInfo, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO groups (id, name, parent_id, color, icon, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, request.name, request.parent_id, request.color, request.icon, now, now],
    )
    .map_err(|e| format!("Failed to create group: {}", e))?;

    Ok(GroupInfo {
        id,
        name: request.name,
        parent_id: request.parent_id,
        sort_order: 0,
        color: request.color,
        icon: request.icon,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_group(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    sort_order: Option<i32>,
) -> Result<(), String> {
    let conn = state.db.conn();
    if let Some(name) = name {
        conn.execute(
            "UPDATE groups SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(color) = color {
        conn.execute(
            "UPDATE groups SET color = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![color, id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(icon) = icon {
        conn.execute(
            "UPDATE groups SET icon = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![icon, id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(sort_order) = sort_order {
        conn.execute(
            "UPDATE groups SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![sort_order, id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_group(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE connections SET group_id = NULL WHERE group_id = ?1",
        [&id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM groups WHERE id = ?1", [&id])
        .map_err(|e| format!("Failed to delete group: {}", e))?;
    Ok(())
}
