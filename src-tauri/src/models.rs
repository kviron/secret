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

/// Сведения об установке: размер на диске и данные Steam `appmanifest` (см. `get_game_install_stats`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInstallStats {
    /// С обходом симлинков (основная метрика «Space used», как в Vortex).
    pub disk_usage_bytes: u64,
    /// Без обхода симлинков («Space used (no symlinks)»).
    pub disk_usage_bytes_no_symlinks: u64,
    pub steam_size_on_disk_bytes: Option<u64>,
    pub steam_build_id: Option<String>,
    /// PE FileVersion (Windows) или `Steam build …` из манифеста.
    pub installed_version_label: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub struct DeployedFile {
    pub source: String,
    pub target: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeployStrategy {
    #[default]
    Auto,
    Symlink,
    Hardlink,
    Copy,
}

impl DeployStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeployStrategy::Auto => "auto",
            DeployStrategy::Symlink => "symlink",
            DeployStrategy::Hardlink => "hardlink",
            DeployStrategy::Copy => "copy",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "symlink" => DeployStrategy::Symlink,
            "hardlink" => DeployStrategy::Hardlink,
            "copy" => DeployStrategy::Copy,
            _ => DeployStrategy::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub file_path: String,
    pub mod_a: String,
    pub mod_b: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionProgress {
    pub message: String,
    pub found: usize,
    pub total: usize,
    #[serde(alias = "current_game")]
    pub current_game: Option<String>,
}

// --- Downloads ---

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    #[default]
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadState::Pending => "pending",
            DownloadState::Downloading => "downloading",
            DownloadState::Paused => "paused",
            DownloadState::Completed => "completed",
            DownloadState::Failed => "failed",
            DownloadState::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub destination: String,
    pub game_id: Option<String>,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub state: String,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub download_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bps: f64,
    pub percent: f64,
    pub state: String,
}

// --- Load Order ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    #[default]
    Esp,
    Esm,
    Esl,
}

impl PluginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginType::Esp => "esp",
            PluginType::Esm => "esm",
            PluginType::Esl => "esl",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "esp" => Some(PluginType::Esp),
            "esm" => Some(PluginType::Esm),
            "esl" => Some(PluginType::Esl),
            _ => None,
        }
    }

    /// Sort priority: ESM < ESL < ESP
    pub fn sort_priority(&self) -> u32 {
        match self {
            PluginType::Esm => 0,
            PluginType::Esl => 1,
            PluginType::Esp => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub name: String,
    pub plugin_type: String,
    pub enabled: bool,
    pub load_order: u32,
    pub is_ghost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadOrderEntry {
    pub game_id: String,
    pub plugin_name: String,
    pub load_order_index: u32,
    pub enabled: bool,
    pub plugin_type: String,
}

// --- Extensions ---

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionType {
    #[default]
    Game,
    ModType,
    Installer,
    Feature,
}

impl ExtensionType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtensionType::Game => "game",
            ExtensionType::ModType => "modtype",
            ExtensionType::Installer => "installer",
            ExtensionType::Feature => "feature",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub extension_type: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub runtime: RuntimeDeps,
    pub detection: Option<GameDetectionConfig>,
    pub mod_paths: Option<std::collections::HashMap<String, String>>,
    pub merge_mods: Option<bool>,
    pub supported_mod_types: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDeps {
    pub requires: Vec<String>,
    pub optional: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetectionConfig {
    pub steam_app_id: Option<String>,
    pub gog_game_id: Option<String>,
    pub epic_offer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub extension_type: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub author: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredGame {
    pub id: String,
    pub extension_id: String,
    pub name: String,
    pub supported_mod_types: Vec<String>,
    pub merge_mods: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModTypeHandlerModel {
    pub id: String,
    pub extension_id: String,
    pub priority: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerHandlerModel {
    pub id: String,
    pub extension_id: String,
    pub priority: i32,
}

// --- Game Launcher ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderInfo {
    pub loader_id: String,
    pub loader_type: String,
    pub executable: String,
    pub version: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningGame {
    pub game_id: String,
    pub process_id: u32,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub process_id: u32,
    pub loader_used: Option<String>,
}

// --- FOMOD ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodInfo {
    pub module_name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub groups: Vec<FomodGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodGroup {
    pub name: String,
    pub options: Vec<FomodOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodOption {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub files: Vec<FomodFileEntry>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodFileEntry {
    pub source: String,
    pub destination: Option<String>,
    pub priority: Option<i32>,
    pub is_folder: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodInstallResult {
    pub files: Vec<FomodFileEntry>,
    pub installed_options: Vec<String>,
}

// --- Security ---

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatType {
    PathTraversal,
    SuspiciousExtension,
    OversizedFile,
    UnsignedPlugin,
    MaliciousPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatInfo {
    pub threat_type: String,
    pub file_path: String,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub is_valid: bool,
    pub threats: Vec<ThreatInfo>,
    pub file_count: usize,
    pub total_size: u64,
}
