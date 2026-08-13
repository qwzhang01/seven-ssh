use std::path::Path;

use rusqlite::Connection;
use ssh_key::public::KeyData;
use ssh_key::PublicKey;

#[derive(Debug, PartialEq)]
pub enum HostKeyStatus {
    Trusted,
    NewKey,
    Changed { old_fingerprint: String },
}

fn fingerprint(key: &PublicKey) -> String {
    use ssh_key::HashAlg;
    key.fingerprint(HashAlg::Sha256).to_string()
}

fn key_type_str(key: &PublicKey) -> &'static str {
    match key.key_data() {
        KeyData::Rsa(_) => "ssh-rsa",
        KeyData::Ed25519(_) => "ssh-ed25519",
        KeyData::Ecdsa(_) => "ecdsa",
        KeyData::Dsa(_) => "ssh-dss",
        _ => "unknown",
    }
}

pub fn check_host_key(
    db_path: &Path,
    host: &str,
    port: u16,
    key: &PublicKey,
) -> Result<HostKeyStatus, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open known_hosts DB: {}", e))?;

    let kt = key_type_str(key);
    let fp = fingerprint(key);

    let result: Option<String> = conn
        .query_row(
            "SELECT key_fingerprint FROM known_hosts WHERE host = ?1 AND port = ?2 AND key_type = ?3",
            rusqlite::params![host, port as u32, kt],
            |row| row.get(0),
        )
        .ok();

    match result {
        Some(stored_fp) if stored_fp == fp => {
            let _ = conn.execute(
                "UPDATE known_hosts SET last_seen = datetime('now') WHERE host = ?1 AND port = ?2 AND key_type = ?3",
                rusqlite::params![host, port as u32, kt],
            );
            Ok(HostKeyStatus::Trusted)
        }
        Some(stored_fp) => Ok(HostKeyStatus::Changed {
            old_fingerprint: stored_fp,
        }),
        None => Ok(HostKeyStatus::NewKey),
    }
}

pub fn save_host_key(
    db_path: &Path,
    host: &str,
    port: u16,
    key: &PublicKey,
) -> Result<(), String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open known_hosts DB: {}", e))?;

    let kt = key_type_str(key);
    let fp = fingerprint(key);
    let key_data = key.to_openssh()
        .map_err(|e| format!("Failed to serialize public key: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO known_hosts (host, port, key_type, key_fingerprint, key_data, first_seen, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?5, COALESCE((SELECT first_seen FROM known_hosts WHERE host = ?1 AND port = ?2 AND key_type = ?3), datetime('now')), datetime('now'))",
        rusqlite::params![host, port as u32, kt, fp, key_data.to_string()],
    )
    .map_err(|e| format!("Failed to save host key: {}", e))?;

    Ok(())
}
