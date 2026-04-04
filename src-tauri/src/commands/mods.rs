use tauri::State;
use crate::AppState;
use crate::models::Mod;

#[tauri::command]
pub async fn install_mod(
    game_id: String,
    archive_path: String,
    state: State<'_, AppState>,
) -> Result<Mod, String> {
    let mod_ = state.installer.install(&state.db, &game_id, std::path::Path::new(&archive_path)).await?;
    Ok(mod_)
}

#[tauri::command]
pub async fn uninstall_mod(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.installer.uninstall(&state.db, &mod_id).await
}

#[tauri::command]
pub async fn get_mods(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Mod>, String> {
    state.db.list_mods(&game_id)
}

#[tauri::command]
pub async fn set_mod_enabled(
    mod_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if enabled {
        state.deployer.enable_mod(&mod_id).await?;
    } else {
        state.deployer.disable_mod(&mod_id).await?;
    }
    Ok(())
}
