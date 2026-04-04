use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::HKEY_CURRENT_USER;
#[cfg(windows)]
use winreg::RegKey;

use crate::models::{DetectionProgress, Game, GameDetails, GameDetectionError, GameLauncher};

const KNOWN_GAMES: &[(&str, &str, &str)] = &[
    ("skyrim", "The Elder Scrolls V: Skyrim", "TESV.exe"),
    ("skyrimse", "Skyrim Special Edition", "SkyrimSE.exe"),
    ("fallout4", "Fallout 4", "Fallout4.exe"),
    ("falloutnv", "Fallout: New Vegas", "FalloutNV.exe"),
    ("oblivion", "The Elder Scrolls IV: Oblivion", "Oblivion.exe"),
];

pub struct GameDetector;

impl GameDetector {
    pub fn new() -> Self {
        GameDetector
    }

    pub fn detect_games<F, E>(detector: &GameDetector, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static,
    {
        let mut games = Vec::new();

        #[cfg(windows)]
        {
            on_progress(DetectionProgress {
                message: "Scanning Steam library...".into(),
                found: 0,
                total: KNOWN_GAMES.len(),
                current_game: None,
            });

            games.extend(detector.detect_steam_games(&on_progress, &on_error));
        }

        #[cfg(not(windows))]
        {
            on_progress(DetectionProgress {
                message: "Game detection not supported on this platform".into(),
                found: 0,
                total: 0,
                current_game: None,
            });
        }

        let found_count = games.len();
        on_progress(DetectionProgress {
            message: format!(
                "Scan complete. Found {} game{}.",
                found_count,
                if found_count == 1 { "" } else { "s" }
            ),
            found: found_count,
            total: found_count,
            current_game: None,
        });

        games
    }

    pub fn scan_custom_path<F, E>(&self, path: &Path, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static,
    {
        let mut games = Vec::new();

        if !path.exists() {
            on_error(GameDetectionError {
                game_id: "custom".into(),
                game_name: "Custom Path".into(),
                error: format!("Path does not exist: {}", path.display()),
                recoverable: false,
            });
            return games;
        }

        if !path.is_dir() {
            on_error(GameDetectionError {
                game_id: "custom".into(),
                game_name: "Custom Path".into(),
                error: "Path is not a directory".into(),
                recoverable: false,
            });
            return games;
        }

        on_progress(DetectionProgress {
            message: format!("Scanning {}...", path.display()),
            found: 0,
            total: KNOWN_GAMES.len(),
            current_game: None,
        });

        for &(id, name, exe) in KNOWN_GAMES {
            let game_path = path.join(exe);
            if game_path.exists() {
                let install_path = path.to_path_buf();
                games.push(Game {
                    id: id.to_string(),
                    name: name.to_string(),
                    install_path: install_path.clone(),
                    support_path: install_path,
                    launcher: GameLauncher::Manual,
                    extension_id: None,
                    supported_mod_types: vec!["simple".to_string(), "fomod".to_string()],
                    merge_mods: true,
                    details: GameDetails {
                        steam_app_id: None,
                        gog_id: None,
                        epic_app_id: None,
                        logo: None,
                        required_files: vec![exe.to_string()],
                        environment: std::collections::HashMap::new(),
                    },
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                });

                on_progress(DetectionProgress {
                    message: format!("Found {}", name),
                    found: games.len(),
                    total: KNOWN_GAMES.len(),
                    current_game: Some(name.to_string()),
                });
            }
        }

        if games.is_empty() {
            on_error(GameDetectionError {
                game_id: "custom".into(),
                game_name: "Custom Path".into(),
                error: "No supported games found in this folder".into(),
                recoverable: true,
            });
        }

        let found_count = games.len();
        on_progress(DetectionProgress {
            message: format!(
                "Scan complete. Found {} game{}.",
                found_count,
                if found_count == 1 { "" } else { "s" }
            ),
            found: found_count,
            total: found_count,
            current_game: None,
        });

        games
    }

    #[cfg(windows)]
    fn detect_steam_games<F, E>(&self, on_progress: &F, on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut games = Vec::new();

        let hkcu = match RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            Ok(key) => key,
            Err(_) => {
                on_error(GameDetectionError {
                    game_id: "steam".into(),
                    game_name: "Steam".into(),
                    error: "Steam not found in registry".into(),
                    recoverable: true,
                });
                return games;
            }
        };

        let steam_path: String = match hkcu.get_value("SteamPath") {
            Ok(p) => p,
            Err(_) => {
                on_error(GameDetectionError {
                    game_id: "steam".into(),
                    game_name: "Steam".into(),
                    error: "Steam path not found in registry".into(),
                    recoverable: true,
                });
                return games;
            }
        };

        let steam_path = PathBuf::from(steam_path);
        let library_folders = self.get_steam_library_folders(&steam_path);

        let mut search_paths = vec![steam_path.join("steamapps")];
        search_paths.extend(library_folders.iter().map(|p| p.join("steamapps")));

        let total = KNOWN_GAMES.len();

        for (idx, &(id, name, exe)) in KNOWN_GAMES.iter().enumerate() {
            on_progress(DetectionProgress {
                message: format!("Checking {}...", name),
                found: games.len(),
                total,
                current_game: Some(name.to_string()),
            });

            let mut found = false;

            for search_path in &search_paths {
                if !search_path.exists() {
                    continue;
                }

                let app_manifest = search_path.join(format!("appmanifest_{}.acf", id));
                if app_manifest.exists() {
                    match self.parse_app_manifest(&app_manifest) {
                        Some(install_dir) => {
                            let install_path = search_path.join("common").join(&install_dir);
                            if install_path.join(exe).exists() {
                                games.push(Game {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    install_path: install_path.clone(),
                                    support_path: install_path,
                                    launcher: GameLauncher::Steam,
                                    extension_id: None,
                                    supported_mod_types: vec![
                                        "simple".to_string(),
                                        "fomod".to_string(),
                                    ],
                                    merge_mods: true,
                                    details: GameDetails {
                                        steam_app_id: id.parse().ok(),
                                        gog_id: None,
                                        epic_app_id: None,
                                        logo: None,
                                        required_files: vec![exe.to_string()],
                                        environment: std::collections::HashMap::new(),
                                    },
                                    created_at: chrono::Utc::now().to_rfc3339(),
                                    updated_at: chrono::Utc::now().to_rfc3339(),
                                });
                                found = true;
                                break;
                            }
                        }
                        None => {
                            on_error(GameDetectionError {
                                game_id: id.to_string(),
                                game_name: name.to_string(),
                                error: "Failed to parse appmanifest".into(),
                                recoverable: true,
                            });
                        }
                    }
                }
            }

            if !found && idx == KNOWN_GAMES.len() - 1 {}
        }

        games
    }

    #[cfg(windows)]
    fn get_steam_library_folders(&self, steam_path: &Path) -> Vec<PathBuf> {
        let mut folders = Vec::new();
        let library_file = steam_path.join("steamapps").join("libraryfolders.vdf");

        if library_file.exists() {
            if let Ok(content) = fs::read_to_string(&library_file) {
                for line in content.lines() {
                    if line.contains("\"path\"") {
                        if let Some(path) = line.split('"').nth(3) {
                            folders.push(PathBuf::from(path));
                        }
                    }
                }
            }
        }

        folders
    }

    #[cfg(windows)]
    fn parse_app_manifest(&self, path: &Path) -> Option<String> {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                if line.contains("\"installdir\"") {
                    return line.split('"').nth(3).map(String::from);
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn manual_register_game(&self, id: &str, name: &str, path: &Path) -> Option<Game> {
        for &(_, _, exe) in KNOWN_GAMES {
            if path.join(exe).exists() || path.join(format!("{}.exe", id)).exists() {
                return Some(Game::new(
                    id,
                    name,
                    path.to_path_buf(),
                    GameLauncher::Manual,
                ));
            }
        }

        if path.exists() && path.is_dir() {
            return Some(Game::new(
                id,
                name,
                path.to_path_buf(),
                GameLauncher::Manual,
            ));
        }

        None
    }
}

impl Default for GameDetector {
    fn default() -> Self {
        Self::new()
    }
}
