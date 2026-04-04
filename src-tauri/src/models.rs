use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameLauncher {
    Steam,
    GOG,
    Epic,
    Xbox,
    Origin,
    Manual,
}

impl GameLauncher {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameLauncher::Steam => "steam",
            GameLauncher::GOG => "gog",
            GameLauncher::Epic => "epic",
            GameLauncher::Xbox => "xbox",
            GameLauncher::Origin => "origin",
            GameLauncher::Manual => "manual",
        }
    }
}

impl Default for GameLauncher {
    fn default() -> Self {
        GameLauncher::Steam
    }
}

impl std::fmt::Display for GameLauncher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetails {
    pub steam_app_id: Option<u32>,
    pub gog_id: Option<String>,
    pub epic_app_id: Option<String>,
    pub logo: Option<String>,
    pub required_files: Vec<String>,
    pub environment: HashMap<String, String>,
}

impl Default for GameDetails {
    fn default() -> Self {
        Self {
            steam_app_id: None,
            gog_id: None,
            epic_app_id: None,
            logo: None,
            required_files: Vec::new(),
            environment: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub install_path: PathBuf,
    pub support_path: PathBuf,
    pub launcher: GameLauncher,
    pub extension_id: Option<String>,
    pub supported_mod_types: Vec<String>,
    pub merge_mods: bool,
    pub details: GameDetails,
    pub created_at: String,
    pub updated_at: String,
}

impl Game {
    pub fn new(id: &str, name: &str, install_path: PathBuf, launcher: GameLauncher) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            install_path: install_path.clone(),
            support_path: install_path,
            launcher,
            extension_id: None,
            supported_mod_types: vec!["simple".to_string(), "fomod".to_string()],
            merge_mods: true,
            details: GameDetails::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub version: Option<String>,
    pub mod_type: String,
    pub install_path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub id: i64,
    pub mod_id: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub mod_id: String,
    pub game_id: String,
    pub status: String,
    pub strategy: String,
    pub deployed_files: Vec<DeployedFile>,
    pub deployed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedFile {
    pub source: String,
    pub target: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionProgress {
    pub message: String,
    pub found: usize,
    pub total: usize,
    pub current_game: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetectionError {
    pub game_id: String,
    pub game_name: String,
    pub error: String,
    pub recoverable: bool,
}
