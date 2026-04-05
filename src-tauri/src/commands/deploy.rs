use crate::models::DeploymentState;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn deploy_mod(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<DeploymentState, String> {
    state.deployer.deploy_mod(&mod_id).await
}

#[tauri::command]
pub async fn undeploy_mod(mod_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.deployer.undeploy_mod(&mod_id).await
}

#[tauri::command]
pub async fn deploy_all(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DeploymentState>, String> {
    state.deployer.deploy_all(&game_id).await
}
