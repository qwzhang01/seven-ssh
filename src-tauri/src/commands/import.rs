use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedConnection {
    pub host_alias: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedConnectionSave {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    pub private_key_path: Option<String>,
    pub group_id: Option<String>,
}

fn home_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|d| d.home_dir().to_path_buf())
}

fn default_ssh_config_path() -> Result<String, String> {
    let home = home_dir().ok_or("Cannot determine home directory")?;
    Ok(home.join(".ssh").join("config").to_string_lossy().into_owned())
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}

fn current_os_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "root".to_string())
}

fn parse_ssh_config(content: &str) -> Vec<ImportedConnection> {
    let mut results: Vec<ImportedConnection> = Vec::new();
    let mut current: Option<HashMap<String, String>> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = match line.split_once(char::is_whitespace) {
            Some((k, v)) => (k.to_lowercase(), v.trim().to_string()),
            None => continue,
        };

        if key == "host" {
            if let Some(ref block) = current {
                if let Some(conn) = build_connection(block) {
                    results.push(conn);
                }
            }

            if value.contains('*') || value.contains('?') {
                current = None;
            } else {
                let mut block = HashMap::new();
                block.insert("host".to_string(), value);
                current = Some(block);
            }
        } else if let Some(ref mut block) = current {
            block.insert(key, value);
        }
    }

    if let Some(ref block) = current {
        if let Some(conn) = build_connection(block) {
            results.push(conn);
        }
    }

    results
}

fn build_connection(block: &HashMap<String, String>) -> Option<ImportedConnection> {
    let host_alias = block.get("host")?.clone();

    let hostname = block
        .get("hostname")
        .cloned()
        .unwrap_or_else(|| host_alias.clone());

    let port = block
        .get("port")
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(22);

    let username = block
        .get("user")
        .cloned()
        .unwrap_or_else(current_os_username);

    let identity_file = block.get("identityfile").map(|p| expand_tilde(p));
    let proxy_jump = block.get("proxyjump").cloned();

    Some(ImportedConnection {
        host_alias,
        hostname,
        port,
        username,
        identity_file,
        proxy_jump,
    })
}

#[tauri::command]
pub fn import_ssh_config(
    _state: State<AppState>,
    file_path: Option<String>,
) -> Result<Vec<ImportedConnection>, String> {
    let path = match file_path {
        Some(p) if !p.is_empty() => expand_tilde(&p),
        _ => default_ssh_config_path()?,
    };

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read SSH config at {}: {}", path, e))?;

    Ok(parse_ssh_config(&content))
}

#[tauri::command]
pub fn import_putty_sessions(
    _state: State<AppState>,
) -> Result<Vec<ImportedConnection>, String> {
    #[cfg(target_os = "windows")]
    {
        import_putty_from_registry()
    }
    #[cfg(not(target_os = "windows"))]
    {
        let putty_config = home_dir()
            .map(|h| h.join(".putty").join("sessions"))
            .unwrap_or_default();

        if putty_config.is_dir() {
            let mut results = Vec::new();
            let entries = fs::read_dir(&putty_config)
                .map_err(|e| format!("Failed to read PuTTY sessions dir: {}", e))?;

            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let session_name = urlencoding::decode(&filename)
                    .unwrap_or_else(|_| filename.clone().into())
                    .to_string();

                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Some(conn) = parse_putty_session_file(&session_name, &content) {
                        results.push(conn);
                    }
                }
            }
            if results.is_empty() {
                return Err("No PuTTY sessions found. On macOS/Linux, PuTTY sessions are looked up in ~/.putty/sessions/".to_string());
            }
            Ok(results)
        } else {
            Err("PuTTY sessions directory not found (~/.putty/sessions/). PuTTY registry import is only available on Windows.".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
fn import_putty_from_registry() -> Result<Vec<ImportedConnection>, String> {
    Err("Windows registry import requires the winreg crate. Add winreg to Cargo.toml dependencies to enable this feature.".to_string())
}

fn parse_putty_session_file(name: &str, content: &str) -> Option<ImportedConnection> {
    let mut hostname = String::new();
    let mut port: u16 = 22;
    let mut username = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "HostName" => hostname = value.trim().to_string(),
                "PortNumber" => port = value.trim().parse().unwrap_or(22),
                "UserName" => username = value.trim().to_string(),
                _ => {}
            }
        }
    }

    if hostname.is_empty() {
        return None;
    }

    Some(ImportedConnection {
        host_alias: name.to_string(),
        hostname,
        port,
        username: if username.is_empty() { current_os_username() } else { username },
        identity_file: None,
        proxy_jump: None,
    })
}

#[tauri::command]
pub fn import_xshell_sessions(
    _state: State<AppState>,
    file_path: String,
) -> Result<Vec<ImportedConnection>, String> {
    let path = expand_tilde(&file_path);
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Cannot access path '{}': {}", path, e))?;

    let mut results = Vec::new();

    if metadata.is_dir() {
        let entries = fs::read_dir(&path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "xsh").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Some(conn) = parse_xshell_session(&content, &p) {
                        results.push(conn);
                    }
                }
            }
        }
    } else {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read Xshell session file: {}", e))?;
        if let Some(conn) = parse_xshell_session(&content, &std::path::PathBuf::from(&path)) {
            results.push(conn);
        }
    }

    if results.is_empty() {
        return Err("No valid Xshell sessions found in the specified path.".to_string());
    }
    Ok(results)
}

fn parse_xshell_session(content: &str, path: &std::path::Path) -> Option<ImportedConnection> {
    let mut hostname = String::new();
    let mut port: u16 = 22;
    let mut username = String::new();
    let mut name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Host" | "HostName" => hostname = value.to_string(),
                "Port" | "PortNumber" => port = value.parse().unwrap_or(22),
                "UserName" | "User" => username = value.to_string(),
                "Name" | "SessionName" => {
                    if !value.is_empty() {
                        name = value.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    if hostname.is_empty() {
        return None;
    }

    Some(ImportedConnection {
        host_alias: name,
        hostname,
        port,
        username: if username.is_empty() { current_os_username() } else { username },
        identity_file: None,
        proxy_jump: None,
    })
}

#[tauri::command]
pub async fn save_imported_connections(
    state: State<'_, AppState>,
    connections: Vec<ImportedConnectionSave>,
) -> Result<u32, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut saved: u32 = 0;

    let conn = state.db.conn();
    for item in &connections {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO connections (id, name, host, port, username, auth_method,
             private_key_path, group_id, tags, charset, keepalive_interval,
             created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '[]', 'UTF-8', 60, ?9, ?10)",
            rusqlite::params![
                id,
                item.name,
                item.host,
                item.port as u32,
                item.username,
                item.auth_method,
                item.private_key_path,
                item.group_id,
                now,
                now,
            ],
        )
        .map_err(|e| format!("Failed to save connection '{}': {}", item.name, e))?;
        saved += 1;
    }

    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let config = r#"
Host myserver
    HostName 192.168.1.100
    Port 2222
    User admin
    IdentityFile ~/.ssh/id_rsa

Host *
    ServerAliveInterval 60
"#;
        let results = parse_ssh_config(config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].host_alias, "myserver");
        assert_eq!(results[0].hostname, "192.168.1.100");
        assert_eq!(results[0].port, 2222);
        assert_eq!(results[0].username, "admin");
        assert!(results[0].identity_file.is_some());
    }

    #[test]
    fn test_parse_multiple_hosts() {
        let config = r#"
Host web1
    HostName web1.example.com
    User deploy

Host db1
    HostName db1.example.com
    Port 22
    User dbadmin
    ProxyJump web1
"#;
        let results = parse_ssh_config(config);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].host_alias, "web1");
        assert_eq!(results[1].host_alias, "db1");
        assert_eq!(results[1].proxy_jump.as_deref(), Some("web1"));
    }

    #[test]
    fn test_skip_wildcards() {
        let config = r#"
Host *
    ServerAliveInterval 60

Host prod-?
    User deploy

Host myserver
    HostName 10.0.0.1
"#;
        let results = parse_ssh_config(config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].host_alias, "myserver");
    }

    #[test]
    fn test_defaults() {
        let config = "Host minimal\n    HostName 10.0.0.1\n";
        let results = parse_ssh_config(config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 22);
        assert!(!results[0].username.is_empty());
    }
}
