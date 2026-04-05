# Pantheon Module Specifications

## Overview

Complete specifications for all backend (Rust) and frontend (Solid.js) modules.

---

## Backend Modules (Rust)

### 1. GameDetector

**Responsibility**: Detect installed games via registry scanning and manifest files.

**Location**: `src-tauri/src/services/game_detector.rs`

**Public Rust API** (what the Tauri layer calls):

```rust
pub struct GameDetector;

impl GameDetector {
    pub fn new() -> Self;

    /// Runs the full detection pipeline (Steam → GOG → Epic → Xbox on Windows; Steam on Linux).
    /// Progress and errors are reported via callbacks (used to emit Tauri events).
    pub fn detect_games<F, E>(&self, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static;

    /// Scan a user-selected folder: try `KNOWN_GAMES` executables, then optional generic single-exe game.
    pub fn scan_custom_path<F, E>(&self, path: &Path, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static;
}
```

Platform-specific helpers (`detect_steam_games`, GOG/Epic/Xbox scans) are **private** implementation details inside the same module.

**Detection Methods (implemented)**:

1. **Steam (Windows)**: `HKCU\Software\Valve\Steam` → `SteamPath`, `steamapps/appmanifest_*.acf`, `libraryfolders.vdf` (normalized + deduplicated paths), `KNOWN_GAMES` + generic `steam_<appid>`, skip-list for tool depots.

2. **Steam (Linux)**: `~/.steam/steam` and `~/.local/share/Steam` — same manifest logic as Windows.

3. **GOG (Windows)**: `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games` (with fallback to `HKLM\SOFTWARE\GOG.com\Games`) — resolve install path from registry values, match against `KNOWN_GAMES` by executable paths.

4. **GOG Galaxy (Windows)**: `%LOCALAPPDATA%\GOG.com\Galaxy\storage\galaxy-2.0.db` — `DbGame.installationPath`.

5. **Epic (Windows)**: `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` — JSON `InstallLocation`, match against `KNOWN_GAMES`.

6. **EA / Origin (Windows)**: `HKLM\SOFTWARE\...\EA Games\*\Install Dir`.

7. **Ubisoft Connect (Windows)**: `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\*\InstallDir`.

8. **Battle.net (Windows)**: `%ProgramData%\Battle.net\Agent\product.db` (SQLite).

9. **Amazon Games (Windows)**: `%LOCALAPPDATA%\Amazon Games\Data\**\*.json`.

10. **Xbox / PC Game Pass (Windows, best-effort)**: `HKLM\SOFTWARE\Microsoft\Windows\XboxGameExport` when present.

11. **Microsoft Store (Windows, best-effort)**: `HKLM\...\Uninstall` — `InstallLocation` scan (capped).

12. **Heroic (Linux)**: `~/.config/heroic/**/*.json`.

13. **Lutris (Linux)**: `~/.local/share/lutris/games/*.yml`.

**Planned / not in v1**:

- Per-game **extension manifests** and non-`None` `extension_id` (ExtensionManager).

**Game Registration Flow**:
```
1. Collect candidates per launcher (full pipeline; see docs/modules/game-detector.md)
2. Match install folders against KNOWN_GAMES executables (and Steam App ID map); exe path index accelerates matching
3. Deduplicate by Game.id
4. Return Vec<Game> (Tauri command persists to SQLite)
```

**Key Interactions**:
- `Database`: Store discovered games (`insert_or_update_game`)

**Vortex Comparison**:
- Vortex: `gamestore-steam/`, `gamestore-gog/`, `gamestore-origin/`
- Pantheon: Unified `GameDetector` with internal launcher passes

---

### 2. ModInstaller

**Responsibility**: Install mods from archives to staging area with validation and type detection.

**Location**: `src-tauri/src/services/mod_installer.rs`

**API**:

```rust
pub struct ModInstaller;

impl ModInstaller {
    pub fn new() -> Self;
    
    pub async fn install(
        &self,
        gameId: &str,
        archivePath: &Path,
        options: InstallOptions,
    ) -> Result<Mod, String>;
    
    pub async fn uninstall(&self, modId: &str) -> Result<(), String>;
    
    pub async fn parse_info(
        &self,
        archivePath: &Path,
    ) -> Result<ModMetadata, String>;
    
    pub fn detect_mod_type(&self, archivePath: &Path) -> Result<ModType, String>;
    
    pub fn extract_archive(
        &self,
        archivePath: &Path,
        destPath: &Path,
    ) -> Result<Vec<PathBuf>, String>;
}
```

**Archive Handling**:

| Extension | Crate | Notes |
|-----------|-------|-------|
| `.zip` | `zip` | Native Rust |
| `.7z` | `sevenz-rust` | 7-Zip format |
| `.rar` | `unrar` or external | Requires external tool |
| `.bsa` | Custom (gamebryo-bsa-support pattern) | Bethesda archives |
| `.ba2` | Custom (gamebryo-ba2-support pattern) | Creation Engine |

**FOMOD Parsing**:
```rust
pub struct FomodInfo {
    pub moduleName: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub groups: Vec<FomodGroup>,
    pub files: Vec<FomodFile>,
}

pub struct FomodGroup {
    pub name: String,
    pub options: Vec<FomodOption>,
    pub defaultOptions: Vec<usize>,
    pub condition: Option<String>,
}

pub struct FomodOption {
    pub name: String,
    pub description: Option<String>,
    pub files: Vec<String>,
    pub installerScript: Option<String>,
}
```

**Mod Type Detection Priority**:

1. **FOMOD**: Check for `fomod/info.xml` or `fomod/module_config.xml`
2. **BepInEx**: Check for `BepInEx/core/*.dll` or `doorstop_config.ini`
3. **Script Extender**: Check for `skse_loader.exe` (SKSE), `f4se_loader.exe` (F4SE)
4. **BSAT/BA2**: Check for archive magic bytes
5. **DAZip**: Dragon Age specific patterns
6. **ENB**: Check for `enbseries/` folder
7. **Simple**: Default fallback

**Key Interactions**:
- `Database`: Store mod records
- `DeployManager`: Pass mod for deployment after install
- `SecurityValidator`: Validate files before install
- `ExtensionManager`: Get game-specific installers

**Vortex Comparison**:
- Vortex: `modtype-bepinex/`, `fomod-installer/`, `gamebryo-bsa-support/`
- Pantheon: Unified installer with extension registry

---

### 3. DeployManager

**Responsibility**: Deploy mods from staging to game folder using symlinks/hardlinks/copy.

**Location**: `src-tauri/src/services/deploy_manager.rs`

**API**:

```rust
pub struct DeployManager;

impl DeployManager {
    pub fn new(db: Database) -> Self;
    
    pub async fn deploy_mod(&self, modId: &str) -> Result<DeploymentState, String>;
    
    pub async fn undeploy_mod(&self, modId: &str) -> Result<(), String>;
    
    pub async fn enable_mod(&self, modId: &str) -> Result<(), String>;
    
    pub async fn disable_mod(&self, modId: &str) -> Result<(), String>;
    
    pub async fn deploy_all(&self, gameId: &str) -> Result<Vec<DeploymentState>, String>;
    
    pub async fn check_conflicts(
        &self,
        gameId: &str,
    ) -> Result<Vec<Conflict>, String>;
    
    pub async fn resolve_conflicts(
        &self,
        gameId: &str,
        resolutions: Vec<ConflictResolution>,
    ) -> Result<(), String>;
    
    pub async fn set_strategy(
        &self,
        gameId: &str,
        strategy: DeployStrategy,
    ) -> Result<(), String>;
}
```

**Deployment Flow**:

```
1. User enables mod
       │
       ▼
2. Check for file conflicts with other deployed mods
       │
       ├──► Conflicts found ──► Emit conflict_detected event
       │                              │
       │                              ▼
       │                     Show conflict resolution UI
       │                              │
       ▼                              │
3. Create symlinks in game folder ◄─┘
       │
       ├──► If symlink fails (not admin) ──► Try hardlink
       │                                          │
       │                                          ▼
       │                              If hardlink fails ──► Copy files
       │
       ▼
4. Update deployment state in database
       │
       ▼
5. Emit deploy_completed event
       │
       ▼
6. (Bethesda games) Trigger Archive Invalidation update
```

**Symlink Implementation (Windows)**:

```rust
use std::os::windows::fs::{symlink_file, symlink_dir};

pub fn create_symlink(source: &Path, target: &Path) -> Result<LinkType, String> {
    if source.is_dir() {
        symlink_dir(source, target).map_err(|e| e.to_string())?;
        Ok(LinkType::DirectoryJunction)
    } else {
        symlink_file(source, target).map_err(|e| e.to_string())?;
        Ok(LinkType::Symlink)
    }
}
```

**Hardlink Fallback**:

```rust
use std::fs::hard_link;

pub fn create_hardlink(source: &Path, target: &Path) -> Result<(), String> {
    hard_link(source, target).map_err(|e| format!("Hard link failed: {}", e))
}
```

**Conflict Detection**:

```rust
pub fn detect_conflicts(
    modFiles: &[PathBuf],
    deployedFiles: &HashMap<PathBuf, String>,
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    
    for file in modFiles {
        if let Some(existingModId) = deployedFiles.get(file) {
            conflicts.push(Conflict::FileConflict {
                modA: existingModId.clone(),
                modB: currentModId.clone(),
                file: file.clone(),
                sizeA: get_file_size(existingModId, file),
                sizeB: get_file_size(currentModId, file),
            });
        }
    }
    
    conflicts
}
```

**Key Interactions**:
- `Database`: Store deployment state
- `LoadOrderManager`: Update plugin list after deploy
- `GameLauncher`: Ensure mods deployed before launch
- `BackupManager`: Backup before changes

**Vortex Comparison**:
- Vortex: Uses VFS (virtual file system) with staging
- Pantheon: Direct symlink/hardlink with conflict detection

---

### 4. LoadOrderManager

**Responsibility**: Manage plugin load order for Bethesda games (ESP/ESM/ESL).

**Location**: `src-tauri/src/services/load_order_manager.rs`

**API**:

```rust
pub struct LoadOrderManager;

impl LoadOrderManager {
    pub fn new(db: Database) -> Self;
    
    pub fn get_load_order(&self, gameId: &str) -> Result<Vec<LoadOrderEntry>, String>;
    
    pub fn set_load_order(
        &self,
        gameId: &str,
        entries: Vec<LoadOrderEntry>,
    ) -> Result<(), String>;
    
    pub fn add_plugin(&self, gameId: &str, pluginName: &str) -> Result<(), String>;
    
    pub fn remove_plugin(&self, gameId: &str, pluginName: &str) -> Result<(), String>;
    
    pub fn enable_plugin(&self, gameId: &str, pluginName: &str) -> Result<(), String>;
    
    pub fn disable_plugin(&self, gameId: &str, pluginName: &str) -> Result<(), String>;
    
    pub fn move_plugin(
        &self,
        gameId: &str,
        pluginName: &str,
        newIndex: u32,
    ) -> Result<(), String>;
    
    pub fn auto_sort(&self, gameId: &str) -> Result<Vec<LoadOrderEntry>, String>;
    
    pub fn set_plugin_ghost(
        &self,
        gameId: &str,
        pluginName: &str,
        ghosted: bool,
    ) -> Result<(), String>;
    
    pub fn convert_plugin_light(
        &self,
        gameId: &str,
        pluginName: &str,
        toLight: bool,
    ) -> Result<(), String>;
    
    pub fn get_plugin_info(
        &self,
        gameId: &str,
        pluginName: &str,
    ) -> Result<PluginInfo, String>;
    
    pub fn refresh_plugin_list(&self, gameId: &str) -> Result<Vec<PluginInfo>, String>;
}
```

**Plugin Types**:

| Extension | Type | Notes |
|-----------|------|-------|
| `.esm` | Master | Always load before ESP, can't be disabled |
| `.esp` | Regular | Standard plugin file |
| `.esl` | Light | Can be converted, limited to 4096 |

**Load Order Persistence**:

```rust
// Store in loadOrder table
pub struct LoadOrderEntry {
    pub gameId: String,
    pub pluginName: String,
    pub loadOrderIndex: u32,
    pub enabled: bool,
    pub groupName: Option<String>,
}
```

**plugins.txt Management**:

```rust
// Write enabled plugins to plugins.txt
pub fn write_plugins_txt(gameId: &str, enabledPlugins: &[String]) -> Result<(), String> {
    let gamePath = get_game_path(gameId)?;
    let pluginsPath = gamePath.join("plugins.txt");
    
    let content = enabledPlugins.join("\n");
    std::fs::write(&pluginsPath, content)?;
    
    Ok(())
}

// Read plugins.txt
pub fn read_plugins_txt(gameId: &str) -> Result<Vec<String>, String> {
    let gamePath = get_game_path(gameId)?;
    let pluginsPath = gamePath.join("plugins.txt");
    
    if !pluginsPath.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(&pluginsPath)?;
    let plugins: Vec<String> = content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect();
    
    Ok(plugins)
}
```

**Ghost Files**:

```rust
// .ghost files allow viewing but not loading plugins
pub fn set_plugin_ghost(
    pluginPath: &Path,
    ghosted: bool,
) -> Result<(), String> {
    let ghostPath = if ghosted {
        PathBuf::from(format!("{}.ghost", pluginPath.display()))
    } else {
        PathBuf::from(pluginPath.file_stem().unwrap().to_string_lossy().as_ref())
    };
    
    if ghosted {
        std::fs::rename(pluginPath, &ghost_path)?;
    } else {
        std::fs::rename(&ghost_path, pluginPath)?;
    }
    
    Ok(())
}
```

**LOOT Integration**:

```rust
pub struct LootApi {
    pub async fn get_masterlist(gameId: &str) -> Result<Masterlist, String>;
    pub async fn sort_plugins(
        gameId: &str,
        plugins: Vec<String>,
    ) -> Result<Vec<String>, String>;
}

pub fn apply_loot_sort(
    currentOrder: Vec<LoadOrderEntry>,
    masterlist: &Masterlist,
) -> Vec<LoadOrderEntry> {
    // Topological sort based on masterlist rules
    let mut sorted = currentOrder.clone();
    // ... sorting logic
    sorted
}
```

**Key Interactions**:
- `Database`: Persist load order
- `DeployManager`: Update after deployment
- `GameLauncher`: Read plugins.txt before launch

**Vortex Comparison**:
- Vortex: `gamebryo-plugin-management/` extension
- Pantheon: Built-in `LoadOrderManager` with LOOT metadata support

---

### 5. DownloadManager

**Responsibility**: Queue and manage downloads with pause/resume support.

**Location**: `src-tauri/src/services/download_manager.rs`

**API**:

```rust
pub struct DownloadManager;

impl DownloadManager {
    pub fn new(db: Database, app: &AppHandle) -> Self;
    
    pub async fn start_download(
        &self,
        url: &str,
        destination: &Path,
    ) -> Result<String, String>;
    
    pub async fn pause_download(&self, downloadId: &str) -> Result<(), String>;
    
    pub async fn resume_download(&self, downloadId: &str) -> Result<(), String>;
    
    pub async fn cancel_download(&self, downloadId: &str) -> Result<(), String>;
    
    pub async fn get_progress(&self, downloadId: &str) -> Result<DownloadProgress, String>;
    
    pub async fn get_download(&self, downloadId: &str) -> Result<Option<Download>, String>;
    
    pub async fn list_downloads(&self) -> Result<Vec<Download>, String>;
    
    pub async fn list_queue(&self) -> Result<Vec<Download>, String>;
}
```

**Download Strategy Selection**:

```rust
pub enum DownloadStrategy {
    Chunked { chunkSize: u64 },  // Range requests for resume
    Single,                       // No resume support
}

pub async fn probe_server(url: &str) -> Result<DownloadStrategy, String> {
    let client = reqwest::Client::new();
    let response = client.head(url).send().await?;
    
    if response.headers().contains_key("Accept-Ranges") {
        Ok(DownloadStrategy::Chunked { chunkSize: 1024 * 1024 }) // 1MB chunks
    } else {
        Ok(DownloadStrategy::Single)
    }
}
```

**Chunked Download Implementation**:

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn download_chunked(
    url: &str,
    dest: &Path,
    progress: &AppHandle,
) -> Result<(), String> {
    let response = reqwest::get(url).await?;
    let total = response.content_length().unwrap_or(0);
    
    let mut file = File::create(dest).await?;
    let mut downloaded: u64 = 0;
    let client = reqwest::Client::new();
    
    // Download in 1MB chunks
    let chunk_size = 1024 * 1024;
    let mut offset = 0;
    
    while offset < total {
        let end = (offset + chunk_size - 1).min(total - 1);
        let range = format!("bytes={}-{}", offset, end);
        
        let mut response = client
            .get(url)
            .header("Range", range)
            .send()
            .await?;
        
        while let Some(chunk) = response.chunk().await {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            // Emit progress event
            progress.emit("download_progress", DownloadProgress {
                bytesWritten: downloaded,
                bytesTotal: total,
                speed: calculate_speed(downloaded, start_time),
                progressPercent: (downloaded as f64 / total as f64) * 100.0,
                state: DownloadState::Downloading,
            }).map_err(|e| e.to_string())?;
        }
        
        offset = end + 1;
    }
    
    Ok(())
}
```

**Queue Management**:

```rust
pub struct DownloadQueue {
    downloads: Vec<Download>,
    maxConcurrent: u32,
}

impl DownloadQueue {
    pub fn enqueue(&mut self, download: Download) {
        self.downloads.push(download);
        self.process_next();
    }
    
    pub fn process_next(&mut self) {
        let active = self.downloads.iter().filter(|d| d.state == DownloadState::Downloading).count();
        if active >= self.maxConcurrent as usize {
            return;
        }
        
        if let Some(next) = self.downloads.iter_mut().find(|d| d.state == DownloadState::Pending) {
            next.state = DownloadState::Downloading;
            // Start download task
        }
    }
}
```

**Key Interactions**:
- `Database`: Store download records
- `ModInstaller`: Pass completed downloads for installation
- `RepositoryApiClient`: Resolve download URLs

**Vortex Comparison**:
- Vortex: `src/main/src/downloading/` with queue management
- Pantheon: Rust async with tokio tasks

---

### 6. ExtensionSystem

**Responsibility**: Load and manage extensions for game support, mod types, and installers.

**Location**: `src-tauri/src/extensions/mod.rs`

**API**:

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&self, ctx: &mut ExtensionContext) -> Result<(), String>;
}

pub trait GameExtension: Extension {
    fn detect(&self) -> Option<GameInfo>;
    fn get_mod_paths(&self, installPath: &Path) -> HashMap<String, PathBuf>;
    fn list_plugins(&self, gamePath: &Path) -> Result<Vec<PluginInfo>, String>;
    fn get_launcher_args(&self, game: &Game) -> Vec<String>;
}

pub trait ModTypeExtension: Extension {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
    fn test(&self, archive: &Path) -> bool;
    fn install(&self, archive: &Path, dest: &Path) -> Result<Mod, String>;
}

pub struct ExtensionManager {
    extensions: HashMap<String, Box<dyn Extension>>,
    gameHandlers: HashMap<String, Box<dyn GameExtension>>,
    modTypeHandlers: Vec<Box<dyn ModTypeExtension>>,
    installerHandlers: Vec<Box<dyn InstallerExtension>>,
}

impl ExtensionManager {
    pub fn new() -> Self;
    
    pub fn load_extensions(&self, path: &Path) -> Result<(), String>;
    
    pub fn register_game(&mut self, game: GameRegistration);
    
    pub fn register_mod_type(&mut self, modType: ModTypeRegistration);
    
    pub fn register_installer(&mut self, installer: InstallerRegistration);
    
    pub fn get_game_handler(&self, gameId: &str) -> Option<&dyn GameExtension>;
    
    pub fn find_installer(&self, archive: &Path) -> Option<&dyn InstallerExtension>;
    
    pub fn detect_mod_type(&self, archive: &Path) -> ModType;
}
```

**Extension Manifest (game.json)**:

```json
{
  "id": "game-skyrim",
  "name": "The Elder Scrolls V: Skyrim",
  "version": "1.0.0",
  "runtime": {
    "requires": ["modtype-bepinex"]
  },
  "detection": {
    "registry": {
      "key": "HKEY_LOCAL_MACHINE\\Software\\Wow6432Node\\Bethesda Softworks\\skyrim",
      "value": "Installed Path"
    },
    "steamAppId": 72850,
    "requiredFiles": ["TESV.exe", "Skyrim.exe"]
  },
  "modPaths": {
    "": "Data",
    "bsa": "Data",
    "skse": "Data/skse"
  },
  "plugins": {
    "extensions": [".esp", ".esm", ".esl"],
    "archiveExtensions": [".bsa", ".ba2"],
    "sortingMetadata": "loot.yaml"
  },
  "tools": [
    {
      "id": "TES5Edit",
      "name": "TES5Edit",
      "executable": "TES5Edit.exe",
      "requiredFiles": ["TES5Edit.exe"]
    },
    {
      "id": "skse",
      "name": "Skyrim Script Extender",
      "executable": "skse_loader.exe",
      "relative": true,
      "defaultPrimary": true
    }
  ]
}
```

**Extension Registration Flow**:

```
1. Scan extensions/ directory
       │
       ▼
2. Load each extension's game.json manifest
       │
       ▼
3. Check dependencies (runtime.requires)
       │
       ▼
4. Initialize extension (call init())
       │
       ▼
5. Extension registers via context
       ├──► register_game()
       ├──► register_mod_type()
       └──► register_installer()
       │
       ▼
6. ExtensionManager stores registrations
       │
       ▼
7. Games/Mods matched to handlers at runtime
```

**Built-in Extensions**:

```rust
// game-generic - basic game support
// modtype-simple - default mod type handler
// installer-default - basic archive extraction

pub fn register_builtin_extensions(ctx: &mut ExtensionContext) {
    // Register generic game handler
    ctx.register_game(GameRegistration {
        id: "generic",
        name: "Generic Game",
        supportedModTypes: vec![ModType::Simple],
        ..Default::default()
    });
    
    // Register simple mod type
    ctx.register_mod_type(ModTypeRegistration {
        modType: ModType::Simple,
        priority: 0,
        test: |_| true,  // Always matches as fallback
    });
}
```

**Key Interactions**:
- `GameDetector`: Match detected games to extensions
- `ModInstaller`: Get installer handlers
- `Database`: Store extension registry

**Vortex Comparison**:
- Vortex: Extensions as npm packages with registration API
- Pantheon: Rust traits with JSON manifests, compiled type safety

---

### 7. SecurityValidator

**Responsibility**: Validate mod archives for security threats and integrity.

**Location**: `src-tauri/src/services/security_validator.rs`

**API**:

```rust
pub struct SecurityValidator;

impl SecurityValidator {
    pub fn new() -> Self;
    
    pub async fn validate_mod(
        &self,
        archivePath: &Path,
    ) -> Result<ValidationResult, String>;
    
    pub async fn scan_files(
        &self,
        modId: &str,
    ) -> Result<ValidationResult, String>;
    
    pub async fn check_malware(
        &self,
        filePath: &Path,
    ) -> Result<bool, String>;
    
    pub fn compute_hash(&self, filePath: &Path) -> Result<String, String>;
    
    pub fn verify_signature(
        &self,
        filePath: &Path,
        expectedHash: &str,
    ) -> Result<bool, String>;
}
```

**Validation Checks**:

1. **File Size Limits**: Warn on unusual file sizes (>1GB)
2. **Path Traversal**: Check for `../` in archive entries
3. **Executable Detection**: Scan for `.exe`, `.dll`, `.bat`, `.ps1`
4. **Known Malware Patterns**: YARA-style pattern matching
5. **Hash Verification**: SHA256 against known good hashes
6. **Required Files**: Some mods require specific files to be present

**Validation Rules by Mod Type**:

```rust
pub struct ValidationRules {
    pub maxFileSize: u64,          // 1GB default
    pub allowedExtensions: Vec<String>,
    pub blockedExtensions: Vec<String>,
    pub requireManifest: bool,
    pub check Executables: bool,
    pub maxPathDepth: usize,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            maxFileSize: 1024 * 1024 * 1024,  // 1GB
            allowedExtensions: vec![
                ".esp", ".esm", ".esl", ".bsa", ".ba2",
                ".zip", ".7z", ".rar",
                ".dds", ".nif", ".hkx", ".fbx",
                ".txt", ".ini", ".cfg",
            ],
            blockedExtensions: vec![
                ".exe", ".dll", ".bat", ".ps1", ".vbs",
                ".js", ".vbs", ".scr",
            ],
            requireManifest: false,
            checkExecutables: true,
            maxPathDepth: 10,
        }
    }
}
```

**Threat Detection**:

```rust
pub enum ThreatType {
    Malware,
    PathTraversal,
    OversizedFile,
    UnsignedPlugin,
    MissingDependency,
    SuspiciousScript,
}

pub struct ThreatInfo {
    pub threatType: ThreatType,
    pub filePath: PathBuf,
    pub severity: Severity,
    pub description: String,
    pub remediation: String,
}
```

**Key Interactions**:
- `ModInstaller`: Pre-install validation
- `DownloadManager`: Post-download validation
- `Database`: Store validation results

**Vortex Comparison**:
- Vortex: Validation tests in extension system
- Pantheon: Dedicated `SecurityValidator` service

---

### 8. ProfileManager

**Responsibility**: Manage multiple mod configurations per game.

**Location**: `src-tauri/src/services/profile_manager.rs`

**API**:

```rust
pub struct ProfileManager;

impl ProfileManager {
    pub fn new(db: Database) -> Self;
    
    pub fn get_profiles(&self, gameId: &str) -> Result<Vec<Profile>, String>;
    
    pub fn get_profile(&self, profileId: &str) -> Result<Option<Profile>, String>;
    
    pub fn create_profile(
        &self,
        gameId: &str,
        name: &str,
        copyFrom: Option<&str>,
    ) -> Result<Profile, String>;
    
    pub fn delete_profile(&self, profileId: &str) -> Result<(), String>;
    
    pub fn rename_profile(&self, profileId: &str, newName: &str) -> Result<(), String>;
    
    pub fn switch_profile(&self, profileId: &str) -> Result<(), String>;
    
    pub fn export_profile(&self, profileId: &str, path: &Path) -> Result<(), String>;
    
    pub fn import_profile(&self, gameId: &str, path: &Path) -> Result<Profile, String>;
    
    pub fn update_mod_state(
        &self,
        profileId: &str,
        modId: &str,
        enabled: bool,
    ) -> Result<(), String>;
}
```

**Profile Data Structure**:

```rust
pub struct Profile {
    pub id: String,
    pub gameId: String,
    pub name: String,
    pub modState: HashMap<String, ModStateEntry>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

pub struct ModStateEntry {
    pub enabled: bool,
    pub customFileOverrides: Vec<String>,  // For VFS redirect
}
```

**Profile Switch Flow**:

```
1. User selects profile
       │
       ▼
2. Save current profile state
       │
       ▼
3. Deploy mods according to new profile's modState
       │
       ├──► Enable/disable mods based on profile
       │
       ▼
4. Update load order for profile
       │
       ▼
5. Write plugins.txt for new state
       │
       ▼
6. Emit profile_switched event
```

**Profile Export Format**:

```json
{
  "version": 1,
  "name": "Vanilla Plus",
  "gameId": "skyrim",
  "exportedAt": "2024-01-15T10:30:00Z",
  "modState": {
    "mod-id-1": { "enabled": true, "customFileOverrides": [] },
    "mod-id-2": { "enabled": false, "customFileOverrides": [] }
  },
  "loadOrder": [
    "update.esm",
    "dawnguard.esm",
    "dragonborn.esm"
  ]
}
```

**Key Interactions**:
- `DeployManager`: Redeploy on profile switch
- `LoadOrderManager`: Per-profile load order
- `Database`: Store profile metadata

**Vortex Comparison**:
- Vortex: Profile system with Redux state
- Pantheon: SQLite-backed profiles with deploy integration

---

### 9. GameLauncher

**Responsibility**: Launch games with proper mod loaders and script extenders.

**Location**: `src-tauri/src/services/game_launcher.rs`

**API**:

```rust
pub struct GameLauncher;

impl GameLauncher {
    pub fn new(db: Database) -> Self;
    
    pub async fn launch_game(
        &self,
        gameId: &str,
        profileId: Option<&str>,
    ) -> Result<u32, String>;  // Returns process ID
    
    pub async fn detect_loaders(
        &self,
        gameId: &str,
    ) -> Result<Vec<LoaderInfo>, String>;
    
    pub async fn ensure_loaders_installed(
        &self,
        gameId: &str,
    ) -> Result<(), String>;
    
    pub fn get_launcher_args(
        &self,
        gameId: &str,
        profileId: Option<&str>,
    ) -> Result<Vec<String>, String>;
    
    pub fn is_game_running(&self, gameId: &str) -> bool;
    
    pub fn kill_game(&self, gameId: &str) -> Result<(), String>;
}
```

**Launcher Args Construction**:

```rust
pub fn get_launcher_args(
    game: &Game,
    profile: Option<&Profile>,
    loaders: &[LoaderInfo],
) -> Vec<String> {
    let mut args = Vec::new();
    
    // Add script extender loader
    if let Some(skse) = loaders.iter().find(|l| l.loaderType == "skse") {
        args.push("--skse");
        if let Some(skseArgs) = &skse.launcherArgs {
            args.extend(skseArgs.iter().cloned());
        }
    }
    
    // Add profile-specific args
    if let Some(profile) = profile {
        if let Some(profileArgs) = get_profile_launcher_args(profile) {
            args.extend(profileArgs);
        }
    }
    
    args
}
```

**Script Extender Detection**:

```rust
pub enum ScriptExtender {
    SKSE,      // Skyrim
    SKSE64,    // Skyrim Special Edition
    F4SE,      // Fallout 4
    NVSE,      // New Vegas
    OBSE,      // Oblivion
    MWSE,      // Morrowind
}

pub struct LoaderInfo {
    pub loaderType: String,
    pub executable: PathBuf,
    pub version: Option<String>,
    pub installed: bool,
    pub launcherArgs: Option<Vec<String>>,
}

pub fn detect_script_extender(gamePath: &Path) -> Option<LoaderInfo> {
    let loaders = [
        ("skse_loader.exe", "skse", ScriptExtender::SKSE),
        ("skse64_loader.exe", "skse64", ScriptExtender::SKSE64),
        ("f4se_loader.exe", "f4se", ScriptExtender::F4SE),
    ];
    
    for (exe, id, _) in loaders.iter() {
        let loaderPath = gamePath.join(exe);
        if loaderPath.exists() {
            return Some(LoaderInfo {
                loaderType: id.to_string(),
                executable: loaderPath,
                version: detect_loader_version(&loaderPath),
                installed: true,
                launcherArgs: None,
            });
        }
    }
    
    None
}
```

**Game Process Monitoring**:

```rust
pub struct GameMonitor {
    runningGames: HashMap<String, u32>,  // gameId -> processId
}

impl GameMonitor {
    pub fn on_game_launched(&mut self, gameId: String, processId: u32) {
        self.runningGames.insert(gameId, processId);
    }
    
    pub fn on_game_exited(&mut self, gameId: &str, exitCode: i32) {
        self.runningGames.remove(gameId);
        // Emit game_exited event with exit code
    }
}
```

**Key Interactions**:
- `DeployManager`: Ensure mods deployed before launch
- `ProfileManager`: Use profile launch args
- `ModInstaller`: Install loader files if needed

**Vortex Comparison**:
- Vortex: `titlebar-launcher/` extension with tools
- Pantheon: Built-in launcher with loader detection

---

### 10. BackupManager

**Responsibility**: Create and restore backups of game files, saves, and configurations.

**Location**: `src-tauri/src/services/backup_manager.rs`

**API**:

```rust
pub struct BackupManager;

impl BackupManager {
    pub fn new(db: Database) -> Self;
    
    pub async fn create_backup(
        &self,
        gameId: &str,
        backupType: BackupType,
    ) -> Result<Backup, String>;
    
    pub async fn restore_backup(&self, backupId: &str) -> Result<(), String>;
    
    pub async fn delete_backup(&self, backupId: &str) -> Result<(), String>;
    
    pub fn list_backups(&self, gameId: &str) -> Result<Vec<Backup>, String>;
    
    pub fn get_backup_info(&self, backupId: &str) -> Result<Option<Backup>, String>;
    
    pub async fn verify_backup(&self, backupId: &str) -> Result<bool, String>;
}
```

**Backup Types**:

```rust
pub enum BackupType {
    Full,      // Complete game folder
    Saves,     // Save games only
    Config,    // Configuration files
    Mods,      // Deployed mods state
}
```

**Backup Creation Flow**:

```
1. Determine backup source path
       │
       ▼
2. Create timestamped backup directory
       │
       ▼
3. Copy files with progress tracking
       │
       ▼
4. Calculate checksum for verification
       │
       ▼
5. Store backup metadata in database
       │
       ▼
6. Emit backup_created event
```

**Restore Flow**:

```
1. Verify backup integrity
       │
       ▼
2. Create rollback point (backup current state)
       │
       ▼
3. Clear target directory
       │
       ▼
4. Extract backup files
       │
       ▼
5. Verify restored files
       │
       ▼
6. Emit backup_restored event
```

**Key Interactions**:
- `DeployManager`: Backup before deployment changes
- `GameLauncher`: Save game state before launch
- `Database`: Store backup metadata

**Vortex Comparison**:
- Vortex: `gamebryo-savegame-management/` extension
- Pantheon: Unified backup system for all data types

---

### 11. DependencyResolver

**Responsibility**: Resolve mod dependencies and detect conflicts.

**Location**: `src-tauri/src/services/dependency_resolver.rs`

**API**:

```rust
pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new(db: Database) -> Self;
    
    pub fn resolve_dependencies(
        &self,
        modId: &str,
    ) -> Result<Vec<String>, String>;  // Returns required mod IDs
    
    pub fn check_conflicts(
        &self,
        modId: &str,
    ) -> Result<Vec<Conflict>, String>;
    
    pub fn get_mod_graph(
        &self,
        gameId: &str,
    ) -> Result<DependencyGraph, String>;
    
    pub fn find_missing_dependencies(
        &self,
        modId: &str,
    ) -> Result<Vec<String>, String>;
    
    pub fn suggest_installation_order(
        &self,
        gameId: &str,
        modIds: Vec<String>,
    ) -> Result<Vec<String>, String>;
}
```

**Dependency Graph**:

```rust
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub modId: String,
    pub name: String,
    pub priority: i32,
}

pub struct GraphEdge {
    pub from: String,  // modId
    pub to: String,    // modId
    pub edgeType: EdgeType,
}

pub enum EdgeType {
    Requires,      // Hard dependency
    Recommends,    // Soft dependency
    Conflicts,     // Incompatible
    Provides,      // Alternative to
}
```

**Topological Sort for Installation Order**:

```rust
pub fn installation_order(
    graph: &DependencyGraph,
    selectedMods: &[String],
) -> Result<Vec<String>, String> {
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    
    fn visit(
        modId: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(modId) {
            return Ok(());
        }
        
        visited.insert(modId.to_string());
        
        // Visit all dependencies first
        for edge in &graph.edges {
            if edge.from == modId && edge.edgeType == EdgeType::Requires {
                visit(&edge.to, graph, visited, result)?;
            }
        }
        
        result.push(modId.to_string());
        Ok(())
    }
    
    for modId in selectedMods {
        visit(modId, graph, &mut visited, &mut result)?;
    }
    
    Ok(result)
}
```

**Key Interactions**:
- `ModInstaller`: Pre-install dependency check
- `RepositoryApiClient`: Fetch dependency info
- `LoadOrderManager`: Dependency-based ordering

**Vortex Comparison**:
- Vortex: `mod-dependency-manager/` extension
- Pantheon: Built-in resolver with graph algorithms

---

### 12. UpdateChecker

**Responsibility**: Check for mod updates from repository.

**Location**: `src-tauri/src/services/update_checker.rs`

**API**:

```rust
pub struct UpdateChecker;

impl UpdateChecker {
    pub fn new(db: Database, repoClient: RepositoryApiClient) -> Self;
    
    pub async fn check_for_updates(
        &self,
        gameId: &str,
    ) -> Result<Vec<ModUpdateInfo>, String>;
    
    pub async fn check_mod_update(
        &self,
        modId: &str,
    ) -> Result<Option<ModUpdateInfo>, String>;
    
    pub async fn pin_version(
        &self,
        modId: &str,
        version: &str,
    ) -> Result<(), String>;
    
    pub async fn unpin_version(
        &self,
        modId: &str,
    ) -> Result<(), String>;
}
```

**Update Info Structure**:

```rust
pub struct ModUpdateInfo {
    pub modId: String,
    pub currentVersion: String,
    pub newVersion: String,
    pub downloadUrl: String,
    pub changelog: Option<String>,
    pub priority: UpdatePriority,
}

pub enum UpdatePriority {
    Critical,   // Security update
    Important,  // Bug fix
    Optional,   // New feature
}
```

**Key Interactions**:
- `RepositoryApiClient`: Get version info
- `DownloadManager`: Download updates
- `ModInstaller`: Install updates

**Vortex Comparison**:
- Vortex: Nexus API integration for update checking
- Pantheon: Own repository API for updates

---

### 13. RepositoryApiClient

**Responsibility**: Interface with Pantheon mod repository.

**Location**: `src-tauri/src/services/repository_api_client.rs`

**API**:

```rust
pub struct RepositoryApiClient {
    baseUrl: String,
    client: reqwest::Client,
    auth: Option<RepositoryAuth>,
}

impl RepositoryApiClient {
    pub fn new(baseUrl: &str) -> Self;
    
    pub async fn search_mods(
        &self,
        query: &str,
        gameId: Option<&str>,
        page: u32,
        pageSize: u32,
    ) -> Result<SearchResults, String>;
    
    pub async fn get_mod_info(
        &self,
        modId: &str,
    ) -> Result<RepositoryModInfo, String>;
    
    pub async fn download_mod(
        &self,
        modId: &str,
        fileId: Option<&str>,
    ) -> Result<String, String>;  // Returns download URL
    
    pub async fn get_file_info(
        &self,
        modId: &str,
        fileId: &str,
    ) -> Result<FileInfo, String>;
    
    pub async fn authenticate(
        &self,
        credentials: AuthCredentials,
    ) -> Result<RepositoryAuth, String>;
    
    pub async fn refresh_auth(&self) -> Result<RepositoryAuth, String>;
}
```

**Repository Data Structures**:

```rust
pub struct SearchResults {
    pub mods: Vec<RepositoryModInfo>,
    pub totalCount: u32,
    pub page: u32,
    pub pageSize: u32,
}

pub struct RepositoryModInfo {
    pub modId: String,
    pub name: String,
    pub summary: Option<String>,
    pub category: String,
    pub author: String,
    pub version: String,
    pub uploadedAt: String,
    pub downloads: u32,
    pub fileId: String,
    pub gameId: String,
    pub tags: Vec<String>,
    pub files: Vec<FileInfo>,
}

pub struct FileInfo {
    pub fileId: String,
    pub version: String,
    pub size: u64,
    pub category: String,
    pub uploadedAt: String,
    pub md5: Option<String>,
}
```

**Key Interactions**:
- `DownloadManager`: Get download URLs
- `UpdateChecker`: Version checking
- `DependencyResolver`: Fetch dependency info

**Vortex Comparison**:
- Vortex: Nexus API client
- Pantheon: Own repository API

---

## Frontend Modules (Solid.js)

### App Layer

**Location**: `src/app/`

```
src/app/
├── App.tsx              # Root component
├── index.tsx            # Entry point
├── providers/
│   ├── ThemeProvider.tsx
│   ├── ToastProvider.tsx
│   └── QueryProvider.tsx
└── router/
    └── index.tsx        # App router
```

**App.tsx**:

```typescript
import { Component, JSX } from 'solid-js';
import { Router } from '@solidjs/router';
import { ThemeProvider } from './providers/ThemeProvider';
import { ToastProvider } from './providers/ToastProvider';

interface AppProps {
  children?: JSX.Element;
}

export const App: Component<AppProps> = (props) => {
  return (
    <Router>
      <ThemeProvider>
        <ToastProvider>
          {props.children}
        </ToastProvider>
      </ThemeProvider>
    </Router>
  );
};
```

---

### Pages Layer

**Location**: `src/pages/`

```
src/pages/
├── dashboard/
│   └── index.tsx        # Dashboard page
├── games/
│   └── index.tsx        # Games list page
├── game-detail/
│   └── index.tsx        # Game detail + mods page
├── downloads/
│   └── index.tsx        # Download queue page
├── settings/
│   └── index.tsx        # Settings page
└── mod-browser/
    └── index.tsx        # Repository browser page
```

**Dashboard Page Pattern**:

```typescript
import { Component } from 'solid-js';
import { GameCard } from '@/widgets/GameCard';
import { useGameStore } from '@/entities/game';

export const DashboardPage: Component = () => {
  const { games, isLoading } = useGameStore();
  
  return (
    <div class="p-6 space-y-6">
      <h1 class="text-2xl font-bold">Dashboard</h1>
      
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <For each={games()} fallback={<EmptyState />}>
          {(game) => <GameCard game={game} />}
        </For>
      </div>
    </div>
  );
};
```

---

### Widgets Layer

**Location**: `src/widgets/`

```
src/widgets/
├── GameCard/
│   ├── index.tsx
│   └── GameCard.tsx
├── ModList/
│   ├── index.tsx
│   └── ModList.tsx
├── LoadOrderEditor/
│   ├── index.tsx
│   └── LoadOrderEditor.tsx
├── DownloadQueue/
│   ├── index.tsx
│   └── DownloadQueue.tsx
└── ConflictResolver/
    ├── index.tsx
    └── ConflictResolver.tsx
```

**GameCard Component**:

```typescript
import { Component, splitProps } from 'solid-js';
import type { Game } from '@/entities/game/model/game';

interface GameCardProps {
  game: Game;
  onSelect: (gameId: string) => void;
  class?: string;
}

export const GameCard: Component<GameCardProps> = (props) => {
  const [local, others] = splitProps(props, ['game', 'onSelect', 'class']);
  
  return (
    <div 
      class={`card ${local.class ?? ''}`}
      onClick={() => local.onSelect(local.game.id)}
    >
      <img 
        src={local.game.details.logo ?? 'default-game.png'} 
        alt={local.game.name}
        class="w-full h-32 object-cover rounded-t"
      />
      <div class="p-4">
        <h3 class="font-semibold">{local.game.name}</h3>
        <p class="text-sm text-gray-500">
          {local.game.launcher}
        </p>
        <div class="mt-2 flex gap-2">
          <Badge>{local.game.supportedModTypes.length} mod types</Badge>
        </div>
      </div>
    </div>
  );
};
```

---

### Features Layer

**Location**: `src/features/`

```
src/features/
├── install-mod/
│   ├── index.ts
│   ├── ui/
│   │   └── InstallModButton.tsx
│   └── model/
│       └── installMod.ts
├── toggle-mod/
│   ├── index.ts
│   ├── ui/
│   │   └── ToggleMod.tsx
│   └── model/
│       └── toggleMod.ts
├── download-mod/
│   ├── index.ts
│   ├── ui/
│   │   └── DownloadModButton.tsx
│   └── model/
│       └── downloadMod.ts
└── resolve-conflict/
    ├── index.ts
    ├── ui/
    │   └── ConflictDialog.tsx
    └── model/
        └── resolveConflict.ts
```

**installMod Feature**:

```typescript
// model/installMod.ts
import { createSignal } from 'solid-js';
import { invoke } from '@tauri-apps/api';
import type { Mod } from '@/entities/game';

export interface InstallModParams {
  gameId: string;
  archivePath: string;
}

export const installMod = async (params: InstallModParams): Promise<Mod> => {
  return invoke<Mod>('install_mod', params);
};

// ui/InstallModButton.tsx
import { Component, createSignal } from 'solid-js';
import { Button } from '@/shared/ui/Button';
import { installMod } from '../model/installMod';

interface InstallModButtonProps {
  gameId: string;
  archivePath: string;
  onInstalled: (mod: Mod) => void;
}

export const InstallModButton: Component<InstallModButtonProps> = (props) => {
  const [isLoading, setIsLoading] = createSignal(false);
  
  const handleInstall = async () => {
    setIsLoading(true);
    try {
      const mod = await installMod({
        gameId: props.gameId,
        archivePath: props.archivePath,
      });
      props.onInstalled(mod);
    } catch (error) {
      // Handle error
    } finally {
      setIsLoading(false);
    }
  };
  
  return (
    <Button 
      onClick={handleInstall}
      isLoading={isLoading()}
    >
      Install Mod
    </Button>
  );
};
```

---

### Entities Layer

**Location**: `src/entities/`

```
src/entities/
├── game/
│   ├── index.ts
│   ├── model/
│   │   ├── game.ts         # Game interface
│   │   └── gameStore.ts    # Solid.js store
│   └── api/
│       └── gameApi.ts      # Tauri invoke wrappers
├── mod/
│   ├── index.ts
│   ├── model/
│   │   ├── mod.ts
│   │   └── modStore.ts
│   └── api/
│       └── modApi.ts
├── deployment/
│   ├── index.ts
│   ├── model/
│   │   ├── deployment.ts
│   │   └── deploymentStore.ts
│   └── api/
│       └── deploymentApi.ts
└── download/
    ├── index.ts
    ├── model/
    │   ├── download.ts
    │   └── downloadStore.ts
    └── api/
        └── downloadApi.ts
```

**gameStore Example**:

```typescript
// model/gameStore.ts
import { createStore } from 'solid-js/store';
import { createResource } from 'solid-js';
import { invoke } from '@tauri-apps/api';
import type { Game } from './game';

interface GameStoreState {
  games: Game[];
  selectedGameId: string | null;
  isLoading: boolean;
  error: string | null;
}

const [state, setState] = createStore<GameStoreState>({
  games: [],
  selectedGameId: null,
  isLoading: false,
  error: null,
});

export const useGameStore = () => {
  const loadGames = async () => {
    setState('isLoading', true);
    setState('error', null);
    try {
      const games = await invoke<Game[]>('get_games');
      setState('games', games);
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isLoading', false);
    }
  };
  
  const detectGames = async () => {
    setState('isLoading', true);
    try {
      const games = await invoke<Game[]>('detect_games');
      setState('games', games);
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isLoading', false);
    }
  };
  
  const selectGame = (gameId: string) => {
    setState('selectedGameId', gameId);
  };
  
  return {
    state,
    loadGames,
    detectGames,
    selectGame,
  };
};
```

---

### Shared Layer

**Location**: `src/shared/`

```
src/shared/
├── ui/
│   ├── Button/
│   │   └── index.tsx
│   ├── Input/
│   │   └── index.tsx
│   ├── Modal/
│   │   └── index.tsx
│   ├── Card/
│   │   └── index.tsx
│   └── Badge/
│       └── index.tsx
├── api/
│   ├── client.ts           # Tauri invoke wrapper
│   └── gameApi.ts
├── lib/
│   ├── formatDate.ts
│   ├── debounce.ts
│   └── formatBytes.ts
└── config/
    ├── routes.ts
    └── constants.ts
```

**Tauri Client Wrapper**:

```typescript
// api/client.ts
import { invoke } from '@tauri-apps/api';
import { listen, emit } from '@tauri-apps/api/event';

export const api = {
  invoke: <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    return invoke<T>(command, args);
  },
  
  on: <T>(
    event: string, 
    handler: (payload: T) => void
  ): Promise<() => void> => {
    return listen<T>(event, (e) => handler(e.payload));
  },
  
  emit: (event: string, payload?: unknown): Promise<void> => {
    return emit(event, payload);
  },
};
```

---

## Implementation Order

### Phase 1: Core MVP
1. Database schema + migrations
2. GameDetector with Steam/GOG detection
3. ModInstaller basic (zip extraction)
4. DeployManager (copy strategy first)
5. Basic UI shell with Solid.js

### Phase 2: Full Features
6. DeployManager symlink/hardlink
7. DownloadManager with queue
8. LoadOrderManager for Bethesda
9. ExtensionSystem skeleton
10. GameLauncher basic

### Phase 3: Extension System
11. Extension system with JSON manifests
12. Bethesda game extension (Skyrim)
13. FOMOD installer
14. SecurityValidator

### Phase 4: Advanced
15. ProfileManager
16. UpdateChecker
17. RepositoryApiClient
18. BackupManager