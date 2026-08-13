use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionContext {
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub os_hint: Option<String>,
    pub cwd: Option<String>,
}

pub fn collect_system_context(ctx: &ConnectionContext) -> String {
    let mut parts = Vec::new();

    if let Some(host) = &ctx.hostname {
        parts.push(format!("Connected to: {host}"));
    }
    if let Some(user) = &ctx.username {
        parts.push(format!("User: {user}"));
    }
    if let Some(os) = &ctx.os_hint {
        parts.push(format!("OS: {os}"));
    }
    if let Some(cwd) = &ctx.cwd {
        parts.push(format!("Working directory: {cwd}"));
    }

    if parts.is_empty() {
        "No connection context available.".to_string()
    } else {
        parts.join("\n")
    }
}

pub fn build_system_prompt(context: &str) -> String {
    format!(
        "You are an expert Linux/Unix system administrator assistant embedded in an SSH client.\n\
         You help users with: command syntax, troubleshooting, scripting, system administration.\n\
         \n\
         Current context:\n\
         {context}\n\
         \n\
         Rules:\n\
         - Be concise and practical\n\
         - Show commands that can be copy-pasted\n\
         - Warn about destructive operations\n\
         - When unsure, ask for clarification\n\
         - Format code blocks with the appropriate language tag\n\
         - If a command could be dangerous, explicitly warn the user"
    )
}
