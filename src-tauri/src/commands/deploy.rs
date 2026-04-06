use crate::models::{Conflict, DeployStrategy, DeploymentState};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn deploy_mod(
    mod_id: String,
    strategy: Option<String>,
    state: State<'_, AppState>,
) -> Result<DeploymentState, String> {
    let strat = strategy
        .map(|s| DeployStrategy::from_str(&s))
        .unwrap_or_default();
    state.deployer.deploy_mod(&mod_id, strat).await
}

#[tauri::command]
pub async fn undeploy_mod(mod_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.deployer.undeploy_mod(&mod_id).await
}

#[tauri::command]
pub async fn deploy_all(
    game_id: String,
    strategy: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DeploymentState>, String> {
    let strat = strategy
        .map(|s| DeployStrategy::from_str(&s))
        .unwrap_or_default();
    state.deployer.deploy_all(&game_id, strat).await
}

#[tauri::command]
pub async fn check_conflicts(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Conflict>, String> {
    state.deployer.check_conflicts(&game_id).await
}
