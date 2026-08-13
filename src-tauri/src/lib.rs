mod ai;
mod commands;
mod crypto;
mod db;
mod sftp;
mod ssh;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state = AppState::new().expect("Failed to initialize application state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::connection::list_connections,
            commands::connection::get_connection,
            commands::connection::create_connection,
            commands::connection::update_connection,
            commands::connection::delete_connection,
            commands::connection::list_groups,
            commands::connection::create_group,
            commands::connection::update_group,
            commands::connection::delete_group,
            commands::terminal::ssh_connect,
            commands::terminal::ssh_disconnect,
            commands::terminal::ssh_write,
            commands::terminal::ssh_resize,
            commands::terminal::session_log_toggle,
            commands::sftp::sftp_open,
            commands::sftp::sftp_close,
            commands::sftp::sftp_list_dir,
            commands::sftp::sftp_mkdir,
            commands::sftp::sftp_remove,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_realpath,
            commands::sftp::sftp_upload,
            commands::sftp::sftp_download,
            commands::sftp::sftp_stat,
            commands::sftp::sftp_read_file,
            commands::sftp::sftp_write_file,
            commands::sftp::sftp_upload_resume,
            commands::sftp::sftp_download_resume,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::security::check_has_master_password,
            commands::security::set_master_password,
            commands::security::verify_master_password,
            commands::security::lock_app,
            commands::security::check_locked,
            commands::security::touch_activity,
            commands::security::check_auto_lock,
            commands::import::import_ssh_config,
            commands::import::save_imported_connections,
            commands::import::import_putty_sessions,
            commands::import::import_xshell_sessions,
            commands::export::export_connections,
            commands::export::export_connections_to_file,
            commands::ai::ai_chat,
            commands::ai::ai_check_danger,
            commands::ai::ai_redact,
            commands::ai::ai_get_config,
            commands::ai::ai_save_config,
            commands::keygen::generate_key_pair,
            commands::keygen::list_local_keys,
            commands::keygen::delete_key,
            commands::keygen::deploy_public_key,
            commands::audit::log_security_event,
            commands::audit::get_security_events,
            commands::audit::clear_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
