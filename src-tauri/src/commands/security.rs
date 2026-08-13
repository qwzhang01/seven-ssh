use std::time::Instant;

use tauri::State;

use crate::crypto::{self, ENC_PREFIX};
use crate::state::AppState;

#[tauri::command]
pub async fn check_has_master_password(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.conn();
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'master_password_hash'",
        [],
        |row| row.get(0),
    );
    Ok(result.is_ok())
}

#[tauri::command]
pub async fn set_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let hash = crypto::hash_master_password(&password)?;

    {
        let conn = state.db.conn();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('master_password_hash', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![hash],
        )
        .map_err(|e| format!("Failed to store master password hash: {}", e))?;
    }

    let master_key = crypto::derive_master_key(&password, &hash)?;

    migrate_plaintext_passwords(&state, &master_key)?;

    *state.master_key.write().await = Some(master_key);
    *state.locked.write().await = false;
    *state.last_activity.write().await = Instant::now();

    Ok(())
}

#[tauri::command]
pub async fn verify_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<bool, String> {
    let stored_hash = {
        let conn = state.db.conn();
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'master_password_hash'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| format!("No master password set: {}", e))?
    };

    let valid = crypto::verify_master_password(&password, &stored_hash)?;
    if valid {
        let master_key = crypto::derive_master_key(&password, &stored_hash)?;
        *state.master_key.write().await = Some(master_key);
        *state.locked.write().await = false;
        *state.last_activity.write().await = Instant::now();
    }

    Ok(valid)
}

#[tauri::command]
pub async fn lock_app(state: State<'_, AppState>) -> Result<(), String> {
    *state.master_key.write().await = None;
    *state.locked.write().await = true;
    Ok(())
}

#[tauri::command]
pub async fn check_locked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.locked.read().await)
}

#[tauri::command]
pub async fn touch_activity(state: State<'_, AppState>) -> Result<(), String> {
    *state.last_activity.write().await = Instant::now();
    Ok(())
}

#[tauri::command]
pub async fn check_auto_lock(
    state: State<'_, AppState>,
    timeout_secs: u64,
) -> Result<bool, String> {
    let elapsed = state.last_activity.read().await.elapsed().as_secs();
    if elapsed > timeout_secs {
        *state.master_key.write().await = None;
        *state.locked.write().await = true;
        return Ok(true);
    }
    Ok(false)
}

fn migrate_plaintext_passwords(state: &AppState, master_key: &[u8; 32]) -> Result<(), String> {
    let conn = state.db.conn();

    let mut stmt = conn
        .prepare("SELECT id, password, passphrase FROM connections")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (id, password, passphrase) in rows {
        let mut updates: Vec<(&str, String)> = Vec::new();

        if let Some(ref pwd) = password {
            if !pwd.is_empty() && !pwd.starts_with(ENC_PREFIX) {
                let encrypted = crypto::encrypt_field(pwd, master_key)?;
                updates.push(("password", format!("{}{}", ENC_PREFIX, encrypted)));
            }
        }

        if let Some(ref pp) = passphrase {
            if !pp.is_empty() && !pp.starts_with(ENC_PREFIX) {
                let encrypted = crypto::encrypt_field(pp, master_key)?;
                updates.push(("passphrase", format!("{}{}", ENC_PREFIX, encrypted)));
            }
        }

        for (col, val) in updates {
            conn.execute(
                &format!("UPDATE connections SET {} = ?1 WHERE id = ?2", col),
                rusqlite::params![val, id],
            )
            .map_err(|e| format!("Failed to migrate {}: {}", col, e))?;
        }
    }

    Ok(())
}
