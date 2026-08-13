use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum SessionCommand {
    Write(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    EnableLogging(PathBuf),
    DisableLogging,
    Close,
}

#[allow(dead_code)]
pub struct SshSession {
    pub session_id: String,
    pub connection_id: String,
    pub command_tx: mpsc::UnboundedSender<SessionCommand>,
    pub status: SessionStatus,
    pub charset: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub enum SessionStatus {
    Connected,
    Disconnected,
}

impl SshSession {
    pub fn new(
        session_id: String,
        connection_id: String,
        command_tx: mpsc::UnboundedSender<SessionCommand>,
        charset: String,
    ) -> Self {
        Self {
            session_id,
            connection_id,
            command_tx,
            status: SessionStatus::Connected,
            charset,
        }
    }
}
