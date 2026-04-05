mod commands;
mod db;
mod models;
mod services;

use db::Database;
use services::deploy_manager::DeployManager;
use services::mod_installer::ModInstaller;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub installer: ModInstaller,
    pub deployer: DeployManager,
}

fn get_app_data_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data).join("pantheon")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = get_app_data_dir();

    let db = Database::new(&app_data_dir).expect("Failed to initialize database");
    db.migrate().expect("Failed to run migrations");

    let staging_path = app_data_dir.join("staging");
    let installer = ModInstaller::new(staging_path);
    let deployer = DeployManager::new(db.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db,
            installer,
            deployer,
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::get_games,
            commands::games::get_game,
            commands::games::detect_games,
            commands::games::scan_custom_path,
            commands::games::register_game,
            commands::games::unregister_game,
            commands::games::remove_game_from_library,
            commands::mods::install_mod,
            commands::mods::uninstall_mod,
            commands::mods::get_mods,
            commands::mods::set_mod_enabled,
            commands::deploy::deploy_mod,
            commands::deploy::undeploy_mod,
            commands::deploy::deploy_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
