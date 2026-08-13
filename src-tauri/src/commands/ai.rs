use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, State};

use crate::ai::context::{build_system_prompt, collect_system_context, ConnectionContext};
use crate::ai::danger::{check_danger, DangerWarning};
use crate::ai::gateway::{create_provider, AiConfig, ChatMessage};
use crate::ai::privacy::redact_sensitive;
use crate::state::AppState;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[tauri::command]
pub async fn ai_chat(
    app: AppHandle,
    state: State<'_, AppState>,
    messages: Vec<ChatMessage>,
    config: AiConfig,
    context: Option<ConnectionContext>,
) -> Result<String, String> {
    let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let event_chunk = format!("ai-chunk-{request_id}");
    let event_complete = format!("ai-complete-{request_id}");

    // Emit the request_id so the frontend knows which events to listen to
    app.emit("ai-request-id", request_id)
        .map_err(|e| e.to_string())?;

    let provider = create_provider(&config)?;

    let mut final_messages = Vec::new();

    // Build system prompt with connection context
    let ctx_str = if let Some(conn_ctx) = context {
        collect_system_context(&conn_ctx)
    } else {
        "No connection context available.".to_string()
    };
    let system_prompt = build_system_prompt(&ctx_str);
    final_messages.push(ChatMessage {
        role: "system".into(),
        content: system_prompt,
    });

    // Check if privacy redaction is enabled
    let privacy_enabled = {
        let conn = state.db.conn();
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = 'aiPrivacyRedaction'")
            .map_err(|e| e.to_string())?;
        let val: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .unwrap_or(None);
        val.map(|v| v != "false").unwrap_or(true)
    };

    for msg in messages {
        let content = if privacy_enabled {
            redact_sensitive(&msg.content)
        } else {
            msg.content
        };
        final_messages.push(ChatMessage {
            role: msg.role,
            content,
        });
    }

    let app_handle = app.clone();
    let chunk_event = event_chunk.clone();

    let result = provider
        .chat_stream(
            final_messages,
            Box::new(move |chunk: String| {
                let _ = app_handle.emit(&chunk_event, &chunk);
            }),
        )
        .await;

    match &result {
        Ok(full) => {
            let _ = app.emit(&event_complete, full.as_str());
        }
        Err(err) => {
            let _ = app.emit(&event_complete, format!("__ERROR__:{err}").as_str());
        }
    }

    result
}

#[tauri::command]
pub fn ai_check_danger(command: String) -> Result<Option<DangerWarning>, String> {
    Ok(check_danger(&command))
}

#[tauri::command]
pub fn ai_redact(text: String) -> Result<String, String> {
    Ok(redact_sensitive(&text))
}

#[tauri::command]
pub fn ai_get_config(state: State<AppState>) -> Result<AiConfig, String> {
    let conn = state.db.conn();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings WHERE key LIKE 'ai_%'")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut config = AiConfig::default();
    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        match key.as_str() {
            "ai_provider" => config.provider = value,
            "ai_api_key" => config.api_key = Some(value),
            "ai_base_url" => config.base_url = Some(value),
            "ai_model" => config.model = value,
            "ai_temperature" => {
                config.temperature = value.parse().unwrap_or(0.7);
            }
            "ai_max_tokens" => {
                config.max_tokens = value.parse().unwrap_or(2048);
            }
            _ => {}
        }
    }
    Ok(config)
}

#[tauri::command]
pub fn ai_save_config(state: State<AppState>, config: AiConfig) -> Result<(), String> {
    let conn = state.db.conn();

    let pairs = [
        ("ai_provider", config.provider),
        ("ai_api_key", config.api_key.unwrap_or_default()),
        ("ai_base_url", config.base_url.unwrap_or_default()),
        ("ai_model", config.model),
        ("ai_temperature", config.temperature.to_string()),
        ("ai_max_tokens", config.max_tokens.to_string()),
    ];

    for (key, value) in &pairs {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )
        .map_err(|e| format!("Failed to save AI config: {e}"))?;
    }

    Ok(())
}
