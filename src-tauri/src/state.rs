use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use crate::db::Database;
use crate::sftp::session::SftpHandle;
use crate::ssh::session::SshSession;

pub struct AppState {
    pub db: Database,
    pub sessions: Arc<RwLock<HashMap<String, SshSession>>>,
    pub sftp_sessions: Arc<RwLock<HashMap<String, SftpHandle>>>,
    pub master_key: Arc<RwLock<Option<[u8; 32]>>>,
    pub locked: Arc<RwLock<bool>>,
    pub last_activity: Arc<RwLock<Instant>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::new()?;
        db.run_migrations()?;

        let state = Self {
            db,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            sftp_sessions: Arc::new(RwLock::new(HashMap::new())),
            master_key: Arc::new(RwLock::new(None)),
            locked: Arc::new(RwLock::new(true)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
        };

        crate::commands::audit::ensure_audit_table(&state);

        Ok(state)
    }
}
