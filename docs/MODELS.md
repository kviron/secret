# Pantheon Data Models

## Overview

Complete type definitions for all data structures in the system. Types are defined in both Rust (backend) and TypeScript (frontend) with identical semantics.

## Game

### Rust

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,                    // Unique game ID (e.g., "skyrim", "fallout4")
    pub name: String,                  // Display name (e.g., "The Elder Scrolls V: Skyrim")
    pub installPath: PathBuf,          // Path to game executable directory
    pub supportPath: PathBuf,         // Path to game Data folder
    pub launcher: GameLauncher,        // Launcher type (Steam, GOG, Epic, etc.)
    pub extensionId: Option<String>,   // Associated extension ID
    pub supportedModTypes: Vec<ModType>, // List of supported mod types
    pub mergeMods: bool,               // Whether mods should be merged in VFS
    pub details: GameDetails,          // Additional game-specific details
    pub createdAt: DateTime<Utc>,      // When game was first discovered
    pub updatedAt: DateTime<Utc>,      // Last modification time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetails {
    pub steamAppId: Option<u32>,       // Steam application ID
    pub gogId: Option<String>,         // GOG galaxy ID
    pub epicAppId: Option<String>,     // Epic games launcher ID
    pub logo: Option<String>,          // Game logo asset name
    pub requiredFiles: Vec<String>,    // Files that indicate valid installation
    pub environment: HashMap<String, String>, // Environment variables for launch
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameLauncher {
    Steam,
    GOG,
    Epic,
    Xbox,
    Origin,
    Manual,    // User-specified path
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
```

### TypeScript

```typescript
// Game entity types

export interface Game {
  id: string;
  name: string;
  installPath: string;
  supportPath: string;
  launcher: GameLauncher;
  extensionId: string | null;
  supportedModTypes: ModType[];
  mergeMods: boolean;
  details: GameDetails;
  createdAt: string;  // ISO 8601
  updatedAt: string;  // ISO 8601
}

export interface GameDetails {
  steamAppId: number | null;
  gogId: string | null;
  epicAppId: string | null;
  logo: string | null;
  requiredFiles: string[];
  environment: Record<string, string>;
}

export type GameLauncher = 
  | 'steam' 
  | 'gog' 
  | 'epic' 
  | 'xbox' 
  | 'origin' 
  | 'manual';

export type ModType = 
  | 'simple' 
  | 'fomod' 
  | 'foomad' 
  | 'bsat' 
  | 'bepinex' 
  | 'dazip' 
  | 'enb' 
  | 'scriptExtender' 
  | 'modPlugin';
```

---

## Mod

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,                    // UUID
    pub gameId: String,                 // Parent game ID
    pub name: String,                  // Display name
    pub version: Option<String>,       // Semantic version
    pub category: Option<String>,      // Category name
    pub modType: ModType,              // Type of mod
    pub installPath: PathBuf,          // Staging directory path
    pub enabled: bool,                 // Is mod enabled
    pub flags: Vec<String>,            // State flags
    pub attributes: HashMap<String, String>, // Custom attributes
    pub installTime: DateTime<Utc>,    // Installation timestamp
    pub lastModified: DateTime<Utc>,   // Last update timestamp
    pub metadata: Option<ModMetadata>,  // Extended metadata
    pub conflicts: Vec<String>,        // IDs of conflicting mods
    pub dependencies: Vec<String>,     // IDs of required mods
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub author: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub installerVersion: Option<String>,
    pub installationFiles: Vec<InstallationFile>,
    pub screenshot: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationFile {
    pub fileType: String,       // "replace" | "optional" | "alternate"
    pub source: String,         // Archive path
    pub destination: String,    // Extraction path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub id: i64,                // Auto-increment ID
    pub modId: String,          // Parent mod ID
    pub path: String,           // Relative path in staging
    pub size: u64,              // File size in bytes
    pub hash: Option<String>,   // SHA256 hash
    pub isArchive: bool,        // Is this a BSA/BA2 archive
}
```

### TypeScript

```typescript
export interface Mod {
  id: string;
  gameId: string;
  name: string;
  version: string | null;
  category: string | null;
  modType: ModType;
  installPath: string;
  enabled: boolean;
  flags: ModFlag[];
  attributes: Record<string, string>;
  installTime: string;  // ISO 8601
  lastModified: string;  // ISO 8601
  metadata: ModMetadata | null;
  conflicts: string[];
  dependencies: string[];
}

export type ModFlag = 
  | 'installed' 
  | 'upgradeable' 
  | 'missing' 
  | 'overwrite' 
  | 'disabled';

export interface ModMetadata {
  author: string | null;
  description: string | null;
  website: string | null;
  installerVersion: string | null;
  installationFiles: InstallationFile[];
  screenshot: string | null;
  category: string | null;
}

export interface InstallationFile {
  fileType: 'replace' | 'optional' | 'alternate';
  source: string;
  destination: string;
}

export interface ModFile {
  id: number;
  modId: string;
  path: string;
  size: number;
  hash: string | null;
  isArchive: boolean;
}
```

---

## Deployment

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub modId: String,
    pub gameId: String,
    pub status: DeployStatus,
    pub strategy: DeployStrategy,
    pub deployedFiles: Vec<DeployedFile>,
    pub conflicts: Vec<Conflict>,
    pub deployedAt: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStatus {
    Pending,               // Mod enabled, not yet deployed
    Deployed,              // Successfully deployed
    PartiallyDeployed,     // Some files deployed
    Failed,                // Deployment failed
    Conflict,              // Has unresolved conflicts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStrategy {
    Symlink,    // Default on Windows
    Hardlink,   // Fallback for same filesystem
    Copy,       // Full copy (no linking)
    Merge,      // VFS folder merge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedFile {
    pub source: PathBuf,     // In staging (relative to mod folder)
    pub target: PathBuf,     // In game folder (relative to game path)
    pub linkType: LinkType,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    Symlink,
    Hardlink,
    Copy,
    DirectoryJunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Conflict {
    FileConflict {
        modA: String,
        modB: String,
        file: PathBuf,
        sizeA: u64,
        sizeB: u64,
    },
    MissingDependency {
        modId: String,
        dependencyId: String,
    },
    PluginConflict {
        pluginA: String,
        pluginB: String,
        loadOrderA: u32,
        loadOrderB: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflictId: String,
    pub resolution: ConflictResolutionType,
    pub winnerModId: Option<String>,
    pub mergedFilePath: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionType {
    UseModA,
    UseModB,
    Merge,
    Skip,
}
```

### TypeScript

```typescript
export interface DeploymentState {
  modId: string;
  gameId: string;
  status: DeployStatus;
  strategy: DeployStrategy;
  deployedFiles: DeployedFile[];
  conflicts: Conflict[];
  deployedAt: string | null;  // ISO 8601
}

export type DeployStatus = 
  | 'pending' 
  | 'deployed' 
  | 'partiallyDeployed' 
  | 'failed' 
  | 'conflict';

export type DeployStrategy = 
  | 'symlink' 
  | 'hardlink' 
  | 'copy' 
  | 'merge';

export interface DeployedFile {
  source: string;
  target: string;
  linkType: LinkType;
  size: number;
  hash: string;
}

export type LinkType = 
  | 'symlink' 
  | 'hardlink' 
  | 'copy' 
  | 'directoryJunction';

export type Conflict = 
  | FileConflict 
  | MissingDependency 
  | PluginConflict;

export interface FileConflict {
  type: 'FileConflict';
  modA: string;
  modB: string;
  file: string;
  sizeA: number;
  sizeB: number;
}

export interface MissingDependency {
  type: 'MissingDependency';
  modId: string;
  dependencyId: string;
}

export interface PluginConflict {
  type: 'PluginConflict';
  pluginA: string;
  pluginB: string;
  loadOrderA: number;
  loadOrderB: number;
}

export interface ConflictResolution {
  conflictId: string;
  resolution: ConflictResolutionType;
  winnerModId: string | null;
  mergedFilePath: string | null;
}

export type ConflictResolutionType = 
  | 'useModA' 
  | 'useModB' 
  | 'merge' 
  | 'skip';
```

---

## Load Order

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOrderEntry {
    pub gameId: String,
    pub pluginName: String,
    pub loadOrderIndex: u32,
    pub enabled: bool,
    pub groupName: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub filePath: PathBuf,
    pub isMaster: bool,
    pub isLight: bool,
    pub isMedium: bool,
    pub isDummy: bool,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masterList: Vec<String>,
    pub revision: u32,
    pub loadOrderIndex: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGroup {
    pub name: String,
    pub after: Vec<String>,        // Group names this group should come after
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserlistRule {
    pub pluginName: String,
    pub after: Vec<ILootReference>,
    pub require: Vec<ILootReference>,
    pub warn: Vec<ILootReference>,
    pub hide: bool,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ILootReference {
    pub name: String,
    pub display: Option<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Masterlist {
    pub globals: Vec<ILootReference>,
    pub plugins: Vec<UserlistRule>,
    pub groups: Vec<PluginGroup>,
}
```

### TypeScript

```typescript
export interface LoadOrderEntry {
  gameId: string;
  pluginName: string;
  loadOrderIndex: number;
  enabled: boolean;
  groupName: string | null;
}

export interface PluginInfo {
  name: string;
  filePath: string;
  isMaster: boolean;
  isLight: boolean;
  isMedium: boolean;
  isDummy: boolean;
  author: string | null;
  description: string | null;
  masterList: string[];
  revision: number;
  loadOrderIndex: number | null;
}

export interface PluginGroup {
  name: string;
  after: string[];
  description: string | null;
}

export interface UserlistRule {
  pluginName: string;
  after: ILootReference[];
  require: ILootReference[];
  warn: ILootReference[];
  hide: boolean;
  group: string | null;
}

export interface ILootReference {
  name: string;
  display: string | null;
  condition: string | null;
}

export interface Masterlist {
  globals: ILootReference[];
  plugins: UserlistRule[];
  groups: PluginGroup[];
}
```

---

## Downloads

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: String,
    pub fileName: String,
    pub url: String,
    pub destination: PathBuf,
    pub state: DownloadState,
    pub bytesWritten: u64,
    pub bytesTotal: Option<u64>,
    pub etag: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub completedAt: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: String,
    pub bytesWritten: u64,
    pub bytesTotal: u64,
    pub speed: u64,              // bytes per second
    pub progressPercent: f64,    // 0.0 to 100.0
    pub state: DownloadState,
}
```

### TypeScript

```typescript
export interface Download {
  id: string;
  fileName: string;
  url: string;
  destination: string;
  state: DownloadState;
  bytesWritten: number;
  bytesTotal: number | null;
  etag: string | null;
  createdAt: string;  // ISO 8601
  completedAt: string | null;  // ISO 8601
  error: string | null;
}

export type DownloadState = 
  | 'pending' 
  | 'downloading' 
  | 'paused' 
  | 'completed' 
  | 'failed';

export interface DownloadProgress {
  id: string;
  bytesWritten: number;
  bytesTotal: number;
  speed: number;
  progressPercent: number;
  state: DownloadState;
}
```

---

## Profile

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub gameId: String,
    pub name: String,
    pub modState: HashMap<String, ModStateEntry>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModStateEntry {
    pub enabled: bool,
    pub customFileOverrides: Vec<String>,
}
```

### TypeScript

```typescript
export interface Profile {
  id: string;
  gameId: string;
  name: string;
  modState: Record<string, ModStateEntry>;
  createdAt: string;  // ISO 8601
  updatedAt: string;  // ISO 8601
}

export interface ModStateEntry {
  enabled: boolean;
  customFileOverrides: string[];
}
```

---

## Settings

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub deploymentStrategy: DeployStrategy,
    pub downloadConcurrency: u32,
    pub stagingPath: PathBuf,
    pub language: String,
    pub autoDeploy: bool,
    pub validateMods: bool,
    pub backupEnabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}
```

### TypeScript

```typescript
export interface AppSettings {
  theme: Theme;
  deploymentStrategy: DeployStrategy;
  downloadConcurrency: number;
  stagingPath: string;
  language: string;
  autoDeploy: boolean;
  validateMods: boolean;
  backupEnabled: boolean;
}

export type Theme = 'light' | 'dark' | 'system';
```

---

## App State

### TypeScript

```typescript
export interface AppState {
  // Persistent (SQLite-backed)
  games: Game[];
  mods: Record<string, Mod[]>;  // keyed by gameId
  deployment: Record<string, DeploymentState>;  // keyed by modId
  loadOrder: Record<string, LoadOrderEntry[]>;  // keyed by gameId
  downloads: Download[];
  settings: AppSettings;
  profiles: Record<string, Profile[]>;  // keyed by gameId

  // Session (ephemeral, in-memory)
  session: {
    selectedGameId: string | null;
    activeProfileId: string | null;
    ui: {
      isLoading: boolean;
      notifications: Notification[];
      dialog: DialogState | null;
    };
  };

  // Security
  validationResults: Record<string, ValidationResult>;

  // Updates
  availableUpdates: ModUpdateInfo[];

  // Repository
  repoAuth: RepositoryAuth | null;
}

export interface Notification {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  message: string;
  actions?: NotificationAction[];
}

export interface NotificationAction {
  title: string;
  action: () => void;
}

export interface DialogState {
  type: string;
  props: Record<string, unknown>;
}

export interface ValidationResult {
  modId: string;
  isValid: boolean;
  issues: ValidationIssue[];
  scannedAt: string;
}

export interface ValidationIssue {
  severity: 'info' | 'warning' | 'error';
  code: string;
  message: string;
  filePath?: string;
}

export interface ModUpdateInfo {
  modId: string;
  currentVersion: string;
  newVersion: string;
  downloadUrl: string;
}

export interface RepositoryAuth {
  token: string;
  expiresAt: string;
}
```

---

## Extension Types

### Rust

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&self, ctx: &mut ExtensionContext) -> Result<(), String>;
}

pub trait GameExtension: Extension {
    fn detect(&self) -> Option<GameInfo>;
    fn getModPaths(&self, installPath: &Path) -> HashMap<String, PathBuf>;
    fn listPlugins(&self, gamePath: &Path) -> Result<Vec<PluginInfo>, String>;
    fn getLauncherArgs(&self, game: &Game) -> Vec<String>;
}

pub trait ModTypeExtension: Extension {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
    fn test(&self, archive: &Path) -> bool;
    fn install(&self, archive: &Path, dest: &Path) -> Result<Mod, String>;
}

pub trait InstallerExtension: Extension {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn supportedTypes(&self) -> &[ModType];
    fn test(&self, archive: &Path) -> Result<bool, String>;
    fn install(
        &self,
        archive: &Path,
        dest: &Path,
        game: &Game,
        options: InstallOptions,
    ) -> Result<Mod, String>;
}

pub struct InstallOptions {
    pub stagingPath: PathBuf,
    pub gameSupportPath: PathBuf,
    pub modType: Option<ModType>,
    pub fileOverrides: HashMap<String, String>,
}

pub struct ExtensionContext {
    pub registerGame: Box<dyn FnMut(GameRegistration)>,
    pub registerModType: Box<dyn FnMut(ModTypeRegistration)>,
    pub registerInstaller: Box<dyn FnMut(InstallerRegistration)>,
    pub registerTool: Box<dyn FnMut(ToolRegistration)>,
}
```

---

## Error Types

### Rust

```rust
#[derive(Debug, thiserror::Error)]
pub enum PantheonError {
    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Mod not found: {0}")]
    ModNotFound(String),

    #[error("Mod already installed: {0}")]
    ModAlreadyInstalled(String),

    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    #[error("Conflict detected: {0}")]
    ConflictDetected(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Extension error: {0}")]
    ExtensionError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
```

### TypeScript

```typescript
export type PantheonError = 
  | { code: 'GAME_NOT_FOUND'; message: string }
  | { code: 'MOD_NOT_FOUND'; message: string }
  | { code: 'MOD_ALREADY_INSTALLED'; message: string }
  | { code: 'DEPLOYMENT_FAILED'; message: string }
  | { code: 'CONFLICT_DETECTED'; message: string; conflicts?: Conflict[] }
  | { code: 'DOWNLOAD_ERROR'; message: string }
  | { code: 'EXTENSION_ERROR'; message: string }
  | { code: 'DATABASE_ERROR'; message: string }
  | { code: 'IO_ERROR'; message: string };
```

---

## Events

### Rust Event Payloads

```rust
// Download progress
pub struct DownloadProgressEvent {
    pub downloadId: String,
    pub bytesWritten: u64,
    pub bytesTotal: u64,
    pub speed: u64,
}

// Download completed
pub struct DownloadCompletedEvent {
    pub downloadId: String,
    pub filePath: PathBuf,
}

// Mod installed
pub struct ModInstalledEvent {
    pub modId: String,
    pub gameId: String,
    pub modType: ModType,
}

// Deploy completed
pub struct DeployCompletedEvent {
    pub modId: String,
    pub status: DeployStatus,
}

// Conflict detected
pub struct ConflictDetectedEvent {
    pub gameId: String,
    pub conflicts: Vec<Conflict>,
}

// Game launched
pub struct GameLaunchedEvent {
    pub gameId: String,
    pub processId: u32,
}

// Game exited
pub struct GameExitedEvent {
    pub gameId: String,
    pub exitCode: i32,
}

// Validation complete
pub struct ValidationCompleteEvent {
    pub modId: String,
    pub isValid: bool,
    pub issues: Vec<ValidationIssue>,
}

// Malware detected
pub struct MalwareDetectedEvent {
    pub modId: String,
    pub filePath: PathBuf,
    pub threatType: String,
}
```

### TypeScript Event Payloads

```typescript
export interface DownloadProgressEvent {
  downloadId: string;
  bytesWritten: number;
  bytesTotal: number;
  speed: number;
}

export interface DownloadCompletedEvent {
  downloadId: string;
  filePath: string;
}

export interface ModInstalledEvent {
  modId: string;
  gameId: string;
  modType: ModType;
}

export interface DeployCompletedEvent {
  modId: string;
  status: DeployStatus;
}

export interface ConflictDetectedEvent {
  gameId: string;
  conflicts: Conflict[];
}

export interface GameLaunchedEvent {
  gameId: string;
  processId: number;
}

export interface GameExitedEvent {
  gameId: string;
  exitCode: number;
}

export interface ValidationCompleteEvent {
  modId: string;
  isValid: boolean;
  issues: ValidationIssue[];
}

export interface MalwareDetectedEvent {
  modId: string;
  filePath: string;
  threatType: string;
}

export interface UpdatesAvailableEvent {
  updates: ModUpdateInfo[];
}

export interface ProfileSwitchedEvent {
  profileId: string;
  gameId: string;
}
```

---

## Protocol Types

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolAction {
    #[serde(rename = "InstallMod")]
    InstallMod {
        gameId: String,
        modId: String,
        fileId: Option<String>,
        version: Option<String>,
    },
    #[serde(rename = "DownloadMod")]
    DownloadMod {
        gameId: String,
        modId: String,
        fileId: Option<String>,
    },
    #[serde(rename = "InstallCollection")]
    InstallCollection {
        gameId: String,
        collectionId: String,
    },
    #[serde(rename = "LaunchGame")]
    LaunchGame {
        gameId: String,
        profile: Option<String>,
    },
    #[serde(rename = "SwitchProfile")]
    SwitchProfile {
        profileId: Option<String>,
        profileName: Option<String>,
    },
    #[serde(rename = "OpenSettings")]
    OpenSettings {
        page: Option<String>,
    },
    #[serde(rename = "OAuthCallback")]
    OAuthCallback {
        code: String,
        state: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub accessToken: String,
    pub refreshToken: String,
    pub expiresAt: DateTime<Utc>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: ProtocolErrorCode,
    pub message: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolErrorCode {
    #[serde(rename = "INVALID_URL")]
    InvalidUrl,
    #[serde(rename = "MISSING_PARAMS")]
    MissingParams,
    #[serde(rename = "UNKNOWN_ACTION")]
    UnknownAction,
    #[serde(rename = "AUTH_REQUIRED")]
    AuthRequired,
    #[serde(rename = "GAME_NOT_FOUND")]
    GameNotFound,
    #[serde(rename = "MOD_NOT_FOUND")]
    ModNotFound,
    #[serde(rename = "OAUTH_ERROR")]
    OAuthError,
}
```

### TypeScript

```typescript
export type ProtocolAction =
  | { type: 'InstallMod'; gameId: string; modId: string; fileId?: string; version?: string }
  | { type: 'DownloadMod'; gameId: string; modId: string; fileId?: string }
  | { type: 'InstallCollection'; gameId: string; collectionId: string }
  | { type: 'LaunchGame'; gameId: string; profile?: string }
  | { type: 'SwitchProfile'; profileId?: string; profileName?: string }
  | { type: 'OpenSettings'; page?: string }
  | { type: 'OAuthCallback'; code: string; state: string };

export interface ProtocolUrl {
  scheme: string;
  host: string;
  path: string;
  params: Record<string, string>;
}

export interface OAuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresAt: string;  // ISO 8601
  scope: string;
}

export interface ProtocolError {
  code: ProtocolErrorCode;
  message: string;
  url?: string;
}

export type ProtocolErrorCode =
  | 'INVALID_URL'
  | 'MISSING_PARAMS'
  | 'UNKNOWN_ACTION'
  | 'AUTH_REQUIRED'
  | 'GAME_NOT_FOUND'
  | 'MOD_NOT_FOUND'
  | 'OAUTH_ERROR';
```