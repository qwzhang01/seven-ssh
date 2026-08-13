pub mod models;

use std::path::PathBuf;

use directories::ProjectDirs;
use rusqlite::Connection;
use parking_lot::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dirs = ProjectDirs::from("com", "sevenssh", "SevenSSH")
            .ok_or("Cannot determine app data directory")?;
        Ok(dirs.data_dir().join("sevenssh.db"))
    }

    pub fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                sort_order INTEGER DEFAULT 0,
                color TEXT,
                icon TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (parent_id) REFERENCES groups(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL,
                auth_method TEXT NOT NULL DEFAULT 'password',
                password TEXT,
                private_key_path TEXT,
                passphrase TEXT,
                group_id TEXT,
                tags TEXT DEFAULT '[]',
                color TEXT,
                charset TEXT DEFAULT 'UTF-8',
                keepalive_interval INTEGER DEFAULT 60,
                startup_command TEXT,
                proxy_jump_id TEXT,
                sort_order INTEGER DEFAULT 0,
                is_favorite INTEGER DEFAULT 0,
                note TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE SET NULL,
                FOREIGN KEY (proxy_jump_id) REFERENCES connections(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS ssh_keys (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key_type TEXT NOT NULL,
                private_key_path TEXT,
                public_key TEXT,
                passphrase TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                category TEXT,
                description TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS connection_history (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                connected_at TEXT NOT NULL DEFAULT (datetime('now')),
                disconnected_at TEXT,
                duration INTEGER,
                FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS known_hosts (
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                key_type TEXT NOT NULL,
                key_fingerprint TEXT NOT NULL,
                key_data TEXT NOT NULL,
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (host, port, key_type)
            );

            CREATE TABLE IF NOT EXISTS security_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                details TEXT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }
}
