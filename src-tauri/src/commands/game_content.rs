use std::fs;
use std::path::PathBuf;

use tauri::State;

use crate::models::Game;
use crate::AppState;

/// Plugin-like archives for Gamebryo-style games live under `support_path` (e.g. .../Data).
#[tauri::command]
pub async fn list_game_plugins(game_id: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;
    list_plugins_in_data_dir(&game)
}

fn list_plugins_in_data_dir(game: &Game) -> Result<Vec<String>, String> {
    let dir = &game.support_path;
    if !dir.is_dir() {
        return Err(format!("Game data folder not found: {:?}", dir));
    }
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        if matches!(ext.as_deref(), Some("esp" | "esm" | "esl")) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Known Windows "Documents/My Games/.../Saves" paths per catalog `game.id`.
/// Keep arms in sync with `gameSupportsKnownSavesLocation` / `gameSaves` in the frontend catalog.
fn saves_dir_for_game_id(game_id: &str) -> Option<PathBuf> {
    let docs = documents_dir()?;
    let my_games = docs.join("My Games");
    let path = match game_id {
        "skyrimse" => my_games.join("Skyrim Special Edition").join("Saves"),
        "skyrim" => my_games.join("Skyrim").join("Saves"),
        "skyrimvr" => my_games.join("Skyrim VR").join("Saves"),
        "fallout4" => my_games.join("Fallout4").join("Saves"),
        "fallout4vr" => my_games.join("Fallout4VR").join("Saves"),
        "falloutnv" => my_games.join("FalloutNV").join("Saves"),
        "oblivion" => my_games.join("Oblivion").join("Saves"),
        "starfield" => my_games.join("Starfield").join("Saves"),
        _ => return None,
    };
    Some(path)
}

fn documents_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("Documents"))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Documents"))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileEntry {
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub async fn list_game_saves(game_id: String, state: State<'_, AppState>) -> Result<Vec<SaveFileEntry>, String> {
    let game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let Some(dir) = saves_dir_for_game_id(game.id.as_str()) else {
        return Ok(Vec::new());
    };

    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut out: Vec<SaveFileEntry> = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push(SaveFileEntry {
                    name: name.to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}
