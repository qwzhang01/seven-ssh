use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    #[serde(skip_serializing)]
    pub passphrase: Option<String>,
    pub group_id: Option<String>,
    pub tags: String,
    pub color: Option<String>,
    pub charset: String,
    pub keepalive_interval: u32,
    pub startup_command: Option<String>,
    pub proxy_jump_id: Option<String>,
    pub sort_order: i32,
    pub is_favorite: bool,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub auth_method: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
    pub group_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub charset: Option<String>,
    pub keepalive_interval: Option<u32>,
    pub startup_command: Option<String>,
    pub proxy_jump_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConnectionRequest {
    pub id: String,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
    pub group_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub charset: Option<String>,
    pub keepalive_interval: Option<u32>,
    pub startup_command: Option<String>,
    pub proxy_jump_id: Option<String>,
    pub is_favorite: Option<bool>,
    pub note: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}
