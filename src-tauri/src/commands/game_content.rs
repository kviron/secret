use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tauri::State;

use crate::models::Game;
use crate::AppState;

/// Plugin-like archives for Gamebryo-style games live under `support_path` (e.g. .../Data).
#[tauri::command]
pub async fn list_game_plugins(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
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

fn system_time_to_iso(st: SystemTime) -> String {
    st.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| {
            let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0);
            match datetime {
                Some(dt) => dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
                None => String::new(),
            }
        })
        .unwrap_or_default()
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub size_label: String,
    pub created: Option<String>,
    pub modified: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackupEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub size_label: String,
    pub created: Option<String>,
    pub original_save_name: String,
}

#[tauri::command]
pub async fn list_game_saves(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SaveFileEntry>, String> {
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
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let size = metadata.len();
                let created = metadata.created().ok().map(system_time_to_iso);
                let modified = metadata.modified().ok().map(system_time_to_iso);
                out.push(SaveFileEntry {
                    name: name.to_string(),
                    path: path.to_string_lossy().into_owned(),
                    size,
                    size_label: format_file_size(size),
                    created,
                    modified,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Validate that a path is inside the game's saves directory (security check).
fn validate_save_path(game_id: &str, save_path: &str) -> Result<PathBuf, String> {
    let saves_dir = saves_dir_for_game_id(game_id)
        .ok_or_else(|| format!("No saves directory for game: {}", game_id))?;
    let canonical_save = PathBuf::from(save_path)
        .canonicalize()
        .map_err(|e| format!("Invalid save path: {}", e))?;
    let canonical_dir = saves_dir
        .canonicalize()
        .map_err(|e| format!("Invalid saves dir: {}", e))?;
    if !canonical_save.starts_with(&canonical_dir) {
        return Err("Save path is outside the game's saves directory".to_string());
    }
    Ok(canonical_save)
}

#[tauri::command]
pub async fn delete_save(
    game_id: String,
    save_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let path = validate_save_path(&game_id, &save_path)?;
    fs::remove_file(&path).map_err(|e| format!("Failed to delete save: {}", e))
}

fn backup_dir_for_game(app_data: &Path, game_id: &str) -> PathBuf {
    app_data.join("backups").join(game_id).join("saves")
}

#[tauri::command]
pub async fn backup_save(
    game_id: String,
    save_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let src = validate_save_path(&game_id, &save_path)?;

    let app_data = get_app_data_dir()?;
    let backup_dir = backup_dir_for_game(&app_data, &game_id);
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let save_name = src
        .file_name()
        .ok_or_else(|| "Invalid save file name".to_string())?
        .to_string_lossy();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", timestamp, save_name);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(&src, &backup_path).map_err(|e| format!("Failed to backup save: {}", e))?;

    Ok(backup_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn restore_save(
    game_id: String,
    backup_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let src = PathBuf::from(&backup_path);
    if !src.is_file() {
        return Err(format!("Backup file not found: {}", backup_path));
    }

    let saves_dir = saves_dir_for_game_id(&game_id)
        .ok_or_else(|| format!("No saves directory for game: {}", game_id))?;

    if !saves_dir.is_dir() {
        fs::create_dir_all(&saves_dir).map_err(|e| format!("Failed to create saves dir: {}", e))?;
    }

    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid backup file name".to_string())?;
    // Strip the timestamp prefix (YYYYMMDD_HHMMSS_) from backup name to get original save name
    let original_name = {
        let name = file_name.to_string_lossy().into_owned();
        if let Some(pos) = name.find('_') {
            let rest = &name[pos + 1..];
            if let Some(pos2) = rest.find('_') {
                rest[pos2 + 1..].to_string()
            } else {
                name
            }
        } else {
            name
        }
    };
    let dest = saves_dir.join(&original_name);

    fs::copy(&src, &dest).map_err(|e| format!("Failed to restore save: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn list_save_backups(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SaveBackupEntry>, String> {
    let _game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let app_data = get_app_data_dir()?;
    let backup_dir = backup_dir_for_game(&app_data, &game_id);

    if !backup_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut out: Vec<SaveBackupEntry> = Vec::new();
    for entry in fs::read_dir(&backup_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let size = metadata.len();
                let created = metadata.created().ok().map(system_time_to_iso);
                // Extract original save name from timestamp prefix
                let original_save_name = {
                    let s = name.to_string();
                    if let Some(pos) = s.find('_') {
                        let rest = &s[pos + 1..];
                        if let Some(pos2) = rest.find('_') {
                            rest[pos2 + 1..].to_string()
                        } else {
                            s
                        }
                    } else {
                        s
                    }
                };
                out.push(SaveBackupEntry {
                    name: name.to_string(),
                    path: path.to_string_lossy().into_owned(),
                    size,
                    size_label: format_file_size(size),
                    created,
                    original_save_name,
                });
            }
        }
    }
    out.sort_by(|a, b| b.name.cmp(&a.name)); // newest first
    Ok(out)
}

fn get_app_data_dir() -> Result<PathBuf, String> {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine app data directory".to_string())?;
    let dir = PathBuf::from(app_data).join("pantheon");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir)
}

#[tauri::command]
pub async fn get_saves_dir_path(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let _game = state
        .db
        .find_game(&game_id)?
        .ok_or_else(|| format!("Game not found: {}", game_id))?;
    Ok(saves_dir_for_game_id(&game_id).map(|p| p.to_string_lossy().into_owned()))
}
