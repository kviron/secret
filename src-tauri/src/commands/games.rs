use std::path::PathBuf;
use tauri::{Emitter, State};
use crate::AppState;
use crate::models::Game;
use crate::services::game_detector::GameDetector;

#[tauri::command]
pub async fn get_games(state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    state.db.list_games()
}

#[tauri::command]
pub async fn detect_games(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    app.emit("game_detection_started", serde_json::json!({}))
        .map_err(|e| e.to_string())?;

    let detector = GameDetector::new();
    let app_progress = app.clone();
    let app_error = app.clone();

    let games = GameDetector::detect_games(
        &detector,
        move |progress| {
            app_progress.emit("game_detection_progress", &progress).ok();
        },
        move |error| {
            app_error.emit("game_detection_error", &error).ok();
        },
    );

    for game in &games {
        state.db.insert_or_update_game(game)?;
        app.emit("game_detected", game).ok();
    }

    app.emit("game_detection_completed", serde_json::json!({
        "count": games.len()
    })).map_err(|e| e.to_string())?;

    Ok(games)
}

#[tauri::command]
pub async fn scan_custom_path(
    path: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    let path = PathBuf::from(&path);

    app.emit("game_detection_started", serde_json::json!({
        "message": format!("Scanning {}...", path.display())
    })).map_err(|e| e.to_string())?;

    let detector = GameDetector::new();
    let app_progress = app.clone();
    let app_error = app.clone();

    let games = detector.scan_custom_path(
        &path,
        move |progress| {
            app_progress.emit("game_detection_progress", &progress).ok();
        },
        move |error| {
            app_error.emit("game_detection_error", &error).ok();
        },
    );

    for game in &games {
        state.db.insert_or_update_game(game)?;
        app.emit("game_detected", game).ok();
    }

    app.emit("game_detection_completed", serde_json::json!({
        "count": games.len()
    })).map_err(|e| e.to_string())?;

    Ok(games)
}

#[tauri::command]
pub async fn register_game(game: Game, state: State<'_, AppState>) -> Result<Game, String> {
    state.db.insert_or_update_game(&game)?;
    Ok(game)
}

#[tauri::command]
pub async fn unregister_game(game_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.db.delete_game(&game_id)
}