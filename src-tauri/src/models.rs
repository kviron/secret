use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModSupportLevel {
    #[default]
    None,
    Partial,
    Full,
}

impl ModSupportLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModSupportLevel::Full => "full",
            ModSupportLevel::Partial => "partial",
            ModSupportLevel::None => "none",
        }
    }
}

impl std::fmt::Display for ModSupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameLauncher {
    #[default]
    Steam,
    Gog,
    Epic,
    Xbox,
    Origin,
    Ubisoft,
    Battlenet,
    Amazon,
    MicrosoftStore,
    Manual,
}

impl GameLauncher {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameLauncher::Steam => "steam",
            GameLauncher::Gog => "gog",
            GameLauncher::Epic => "epic",
            GameLauncher::Xbox => "xbox",
            GameLauncher::Origin => "origin",
            GameLauncher::Ubisoft => "ubisoft",
            GameLauncher::Battlenet => "battlenet",
            GameLauncher::Amazon => "amazon",
            GameLauncher::MicrosoftStore => "microsoftstore",
            GameLauncher::Manual => "manual",
        }
    }
}

impl std::fmt::Display for GameLauncher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    #[serde(alias = "steam_app_id")]
    pub steam_app_id: Option<u32>,
    #[serde(alias = "gog_id")]
    pub gog_id: Option<String>,
    #[serde(alias = "epic_app_id")]
    pub epic_app_id: Option<String>,
    pub logo: Option<String>,
    #[serde(alias = "required_files")]
    pub required_files: Vec<String>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    #[serde(alias = "install_path")]
    pub install_path: PathBuf,
    #[serde(alias = "support_path")]
    pub support_path: PathBuf,
    /// Не хранится в БД: выставляется при чтении из API по факту `install_path.exists()`.
    #[serde(default)]
    pub install_path_missing: bool,
    pub launcher: GameLauncher,
    #[serde(alias = "extension_id")]
    pub extension_id: Option<String>,
    #[serde(alias = "supported_mod_types")]
    pub supported_mod_types: Vec<String>,
    #[serde(alias = "merge_mods")]
    pub merge_mods: bool,
    #[serde(alias = "mod_support")]
    pub mod_support: ModSupportLevel,
    pub details: GameDetails,
    #[serde(alias = "created_at")]
    pub created_at: String,
    #[serde(alias = "updated_at")]
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
            install_path_missing: false,
            launcher,
            extension_id: None,
            supported_mod_types: vec!["simple".to_string(), "fomod".to_string()],
            merge_mods: true,
            mod_support: ModSupportLevel::None,
            details: GameDetails::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub id: String,
    #[serde(alias = "game_id")]
    pub game_id: String,
    pub name: String,
    pub version: Option<String>,
    #[serde(alias = "mod_type")]
    pub mod_type: String,
    #[serde(alias = "install_path")]
    pub install_path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    pub id: i64,
    #[serde(alias = "mod_id")]
    pub mod_id: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentState {
    #[serde(alias = "mod_id")]
    pub mod_id: String,
    #[serde(alias = "game_id")]
    pub game_id: String,
    pub status: String,
    pub strategy: String,
    #[serde(alias = "deployed_files")]
    pub deployed_files: Vec<DeployedFile>,
    #[serde(alias = "deployed_at")]
    pub deployed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedFile {
    pub source: String,
    pub target: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionProgress {
    pub message: String,
    pub found: usize,
    pub total: usize,
    #[serde(alias = "current_game")]
    pub current_game: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveGameResult {
    #[serde(alias = "deleted_mods")]
    pub deleted_mods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetectionError {
    #[serde(alias = "game_id")]
    pub game_id: String,
    #[serde(alias = "game_name")]
    pub game_name: String,
    pub error: String,
    pub recoverable: bool,
}
