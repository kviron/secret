use crate::models::{DeployStrategy, FomodInfo, Mod, ValidationResult};
use crate::services::fomod_parser;
use crate::services::security_validator::SecurityValidator;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn install_mod(
    game_id: String,
    archive_path: String,
    state: State<'_, AppState>,
) -> Result<Mod, String> {
    let mod_ = state
        .installer
        .install(&state.db, &game_id, std::path::Path::new(&archive_path))
        .await?;
    Ok(mod_)
}

#[tauri::command]
pub async fn uninstall_mod(mod_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.installer.uninstall(&state.db, &mod_id).await
}

#[tauri::command]
pub async fn get_mods(game_id: String, state: State<'_, AppState>) -> Result<Vec<Mod>, String> {
    state.db.list_mods(&game_id)
}

#[tauri::command]
pub async fn set_mod_enabled(
    mod_id: String,
    enabled: bool,
    strategy: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if enabled {
        let strat = strategy
            .map(|s| DeployStrategy::from_str(&s))
            .unwrap_or_default();
        state.deployer.enable_mod(&mod_id, strat).await?;
    } else {
        state.deployer.disable_mod(&mod_id).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn parse_fomod(archive_dir: String) -> Result<FomodInfo, String> {
    fomod_parser::parse_fomod(std::path::Path::new(&archive_dir))
}

#[tauri::command]
pub fn validate_mod_files(mod_dir: String) -> Result<ValidationResult, String> {
    let validator = SecurityValidator::new();
    validator.validate_mod(std::path::Path::new(&mod_dir))
}
