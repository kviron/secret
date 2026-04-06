mod commands;
mod db;
mod extensions;
mod models;
mod services;

use db::Database;
use extensions::ExtensionRegistry;
use services::deploy_manager::DeployManager;
use services::download_manager::DownloadManager;
use services::game_launcher::GameLauncherService;
use services::load_order_manager::LoadOrderManager;
use services::mod_installer::ModInstaller;
use std::path::PathBuf;
use tauri::Manager;

pub struct AppState {
    pub db: Database,
    pub installer: ModInstaller,
    pub deployer: DeployManager,
    pub downloader: DownloadManager,
    pub load_order: LoadOrderManager,
    pub extensions: ExtensionRegistry,
    pub launcher: GameLauncherService,
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
    let downloads_dir = app_data_dir.join("downloads");
    let installer = ModInstaller::new(staging_path);
    let deployer = DeployManager::new(db.clone());
    let load_order = LoadOrderManager::new(db.clone());
    let launcher = GameLauncherService::new(db.clone());

    let mut extensions = ExtensionRegistry::new();

    // Scan extensions directory for game.json manifests
    let _ = extensions.auto_scan(&app_data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let downloader = DownloadManager::new(db.clone(), downloads_dir, app.handle().clone());

            app.manage(AppState {
                db,
                installer,
                deployer,
                downloader,
                load_order,
                extensions,
                launcher,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::get_games,
            commands::games::get_game,
            commands::games::get_game_install_stats,
            commands::games::detect_games,
            commands::games::scan_custom_path,
            commands::games::register_game,
            commands::games::unregister_game,
            commands::games::remove_game_from_library,
            commands::mods::install_mod,
            commands::mods::uninstall_mod,
            commands::mods::get_mods,
            commands::mods::set_mod_enabled,
            commands::mods::parse_fomod,
            commands::mods::validate_mod_files,
            commands::deploy::deploy_mod,
            commands::deploy::undeploy_mod,
            commands::deploy::deploy_all,
            commands::deploy::check_conflicts,
            commands::downloads::start_download,
            commands::downloads::pause_download,
            commands::downloads::resume_download,
            commands::downloads::cancel_download,
            commands::downloads::get_download,
            commands::downloads::list_downloads,
            commands::downloads::list_download_queue,
            commands::load_order::refresh_plugin_list,
            commands::load_order::get_load_order,
            commands::load_order::set_plugin_enabled,
            commands::load_order::move_plugin,
            commands::load_order::auto_sort_plugins,
            commands::load_order::write_plugins_txt,
            commands::load_order::read_plugins_txt,
            commands::load_order::set_plugin_ghost,
            commands::game_launcher::launch_game,
            commands::game_launcher::detect_game_loaders,
            commands::game_launcher::is_game_running,
            commands::game_launcher::list_running_games,
            commands::game_launcher::get_running_game,
            commands::game_launcher::kill_game,
            commands::game_content::list_game_plugins,
            commands::game_content::list_game_saves,
            commands::game_content::delete_save,
            commands::game_content::backup_save,
            commands::game_content::restore_save,
            commands::game_content::list_save_backups,
            commands::game_content::get_saves_dir_path,
            commands::system::open_folder,
            commands::extensions::list_extensions,
            commands::extensions::get_extension_info,
            commands::extensions::rescan_extensions,
            commands::extensions::get_extension_manifest,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
