use std::collections::HashMap;

use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.conn();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        settings.insert(key, value);
    }
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Failed to save setting: {}", e))?;
    Ok(())
}
