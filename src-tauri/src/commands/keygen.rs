use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use ssh_key::{
    rand_core::OsRng, Algorithm, EcdsaCurve, HashAlg, LineEnding, PrivateKey,
};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct KeyGenResult {
    pub private_key_path: String,
    pub public_key_path: String,
    pub public_key_text: String,
    pub fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct LocalKeyInfo {
    pub name: String,
    pub path: String,
    pub key_type: String,
    pub fingerprint: String,
    pub bits: Option<u32>,
    pub comment: String,
    pub has_public_key: bool,
    pub public_key_path: String,
    pub created: String,
}

fn default_ssh_dir() -> Result<PathBuf, String> {
    dirs_path()
}

fn dirs_path() -> Result<PathBuf, String> {
    let home = directories::BaseDirs::new()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;
    Ok(home.home_dir().join(".ssh"))
}

#[tauri::command]
pub async fn generate_key_pair(
    key_type: String,
    bits: Option<u32>,
    passphrase: Option<String>,
    comment: Option<String>,
    save_path: Option<String>,
) -> Result<KeyGenResult, String> {
    let mut rng = OsRng;

    let algorithm = match key_type.to_lowercase().as_str() {
        "ed25519" => Algorithm::Ed25519,
        "rsa" => {
            let key_bits = bits.unwrap_or(4096);
            if key_bits != 2048 && key_bits != 3072 && key_bits != 4096 {
                return Err("RSA bits must be 2048, 3072, or 4096".to_string());
            }
            Algorithm::Rsa { hash: Some(HashAlg::Sha256) }
        }
        "ecdsa" => {
            let curve_bits = bits.unwrap_or(256);
            let _curve = match curve_bits {
                256 => EcdsaCurve::NistP256,
                384 => EcdsaCurve::NistP384,
                521 => EcdsaCurve::NistP521,
                _ => return Err("ECDSA bits must be 256, 384, or 521".to_string()),
            };
            Algorithm::Ecdsa {
                curve: match curve_bits {
                    256 => EcdsaCurve::NistP256,
                    384 => EcdsaCurve::NistP384,
                    521 => EcdsaCurve::NistP521,
                    _ => unreachable!(),
                },
            }
        }
        _ => return Err(format!("Unsupported key type: {}. Use ed25519, rsa, or ecdsa", key_type)),
    };

    let private_key = PrivateKey::random(&mut rng, algorithm)
        .map_err(|e| format!("Failed to generate key: {}", e))?;

    let comment_str = comment.unwrap_or_default();
    let mut private_key_with_comment = private_key.clone();
    if !comment_str.is_empty() {
        private_key_with_comment.set_comment(&comment_str);
    }

    let ssh_dir = default_ssh_dir()?;
    fs::create_dir_all(&ssh_dir)
        .map_err(|e| format!("Failed to create .ssh directory: {}", e))?;

    let default_name = format!("id_{}", key_type.to_lowercase());
    let private_path = if let Some(ref sp) = save_path {
        PathBuf::from(sp)
    } else {
        ssh_dir.join(&default_name)
    };
    let public_path = private_path.with_extension("pub");

    let private_pem = if let Some(ref pp) = passphrase {
        if pp.is_empty() {
            private_key_with_comment
                .to_openssh(LineEnding::LF)
                .map_err(|e| format!("Failed to encode private key: {}", e))?
                .to_string()
        } else {
            private_key_with_comment
                .encrypt(&mut rng, pp)
                .map_err(|e| format!("Failed to encrypt private key: {}", e))?
                .to_openssh(LineEnding::LF)
                .map_err(|e| format!("Failed to encode encrypted private key: {}", e))?
                .to_string()
        }
    } else {
        private_key_with_comment
            .to_openssh(LineEnding::LF)
            .map_err(|e| format!("Failed to encode private key: {}", e))?
            .to_string()
    };

    let public_key = private_key_with_comment.public_key();
    let public_key_text = public_key.to_openssh()
        .map_err(|e| format!("Failed to encode public key: {}", e))?;

    let fingerprint = public_key.fingerprint(HashAlg::Sha256).to_string();

    if let Some(parent) = private_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&private_path, &private_pem)
        .map_err(|e| format!("Failed to write private key: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&private_path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set private key permissions: {}", e))?;
    }

    fs::write(&public_path, &public_key_text)
        .map_err(|e| format!("Failed to write public key: {}", e))?;

    Ok(KeyGenResult {
        private_key_path: private_path.to_string_lossy().to_string(),
        public_key_path: public_path.to_string_lossy().to_string(),
        public_key_text,
        fingerprint,
    })
}

#[tauri::command]
pub async fn list_local_keys() -> Result<Vec<LocalKeyInfo>, String> {
    let ssh_dir = default_ssh_dir()?;
    if !ssh_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&ssh_dir)
        .map_err(|e| format!("Failed to read .ssh directory: {}", e))?;

    let mut keys = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        if name.ends_with(".pub") || name == "known_hosts" || name == "known_hosts.old"
            || name == "config" || name == "authorized_keys" || name.starts_with('.')
        {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !content.contains("PRIVATE KEY") && !content.starts_with("-----BEGIN OPENSSH PRIVATE KEY") {
            continue;
        }

        let (key_type, fingerprint, bits, comment) = match PrivateKey::from_openssh(&content) {
            Ok(pk) => {
                let algo = match pk.algorithm() {
                    Algorithm::Ed25519 => "ED25519".to_string(),
                    Algorithm::Rsa { .. } => "RSA".to_string(),
                    Algorithm::Ecdsa { curve } => format!("ECDSA-{}", match curve {
                        EcdsaCurve::NistP256 => "256",
                        EcdsaCurve::NistP384 => "384",
                        EcdsaCurve::NistP521 => "521",
                    }),
                    _ => "Unknown".to_string(),
                };
                let fp = pk.public_key().fingerprint(HashAlg::Sha256).to_string();
                let b = match pk.algorithm() {
                    Algorithm::Rsa { .. } => Some(4096u32),
                    Algorithm::Ed25519 => Some(256),
                    Algorithm::Ecdsa { curve } => Some(match curve {
                        EcdsaCurve::NistP256 => 256,
                        EcdsaCurve::NistP384 => 384,
                        EcdsaCurve::NistP521 => 521,
                    }),
                    _ => None,
                };
                let c = pk.comment().to_string();
                (algo, fp, b, c)
            }
            Err(_) => {
                ("Unknown".to_string(), String::new(), None, String::new())
            }
        };

        let public_path = path.with_extension("pub");
        let has_public_key = public_path.exists();

        let created = fs::metadata(&path)
            .and_then(|m| m.created())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default();

        keys.push(LocalKeyInfo {
            name,
            path: path.to_string_lossy().to_string(),
            key_type,
            fingerprint,
            bits,
            comment,
            has_public_key,
            public_key_path: public_path.to_string_lossy().to_string(),
            created,
        });
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(keys)
}

#[tauri::command]
pub async fn delete_key(path: String) -> Result<(), String> {
    let key_path = PathBuf::from(&path);

    if !key_path.exists() {
        return Err("Key file not found".to_string());
    }

    let ssh_dir = default_ssh_dir()?;
    if !key_path.starts_with(&ssh_dir) {
        return Err("Can only delete keys in ~/.ssh directory".to_string());
    }

    fs::remove_file(&key_path)
        .map_err(|e| format!("Failed to delete private key: {}", e))?;

    let pub_path = key_path.with_extension("pub");
    if pub_path.exists() {
        fs::remove_file(&pub_path)
            .map_err(|e| format!("Failed to delete public key: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn deploy_public_key(
    state: State<'_, AppState>,
    connection_id: String,
    public_key_path: String,
) -> Result<(), String> {
    use crate::commands::connection::get_connection_with_secrets;
    use crate::ssh::client::SshClient;

    let pub_key_content = fs::read_to_string(&public_key_path)
        .map_err(|e| format!("Failed to read public key: {}", e))?;
    let pub_key_content = pub_key_content.trim().to_string();

    let conn_info = get_connection_with_secrets(state.inner(), &connection_id).await?;
    let db_path = crate::db::Database::db_path()
        .map_err(|e| format!("Failed to get DB path: {}", e))?;

    let (_handle, mut channel) = SshClient::connect(&conn_info, db_path).await?;

    let command = format!(
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
        pub_key_content
    );

    channel
        .exec(true, command.into_bytes())
        .await
        .map_err(|e| format!("Failed to execute deploy command: {}", e))?;

    use russh::ChannelMsg;
    let mut exit_status = None;
    loop {
        match channel.wait().await {
            Some(ChannelMsg::ExitStatus { exit_status: code }) => {
                exit_status = Some(code);
            }
            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
            _ => {}
        }
    }

    if let Some(code) = exit_status {
        if code != 0 {
            return Err(format!("Deploy command failed with exit code: {}", code));
        }
    }

    Ok(())
}
