# Pantheon Implementation Flows

## Overview

Step-by-step implementation flows for core user interactions. Each flow shows the exact sequence of operations across frontend and backend.

---

## Flow 1: Game Detection

**Trigger**: User opens app or clicks "Detect Games"

**Flow**:

```
1. UI — `invoke('detect_games')` via game API / feature buttons; listen for detection events.
2. Rust — `commands::games::detect_games` → `GameDetector::detect_games(on_progress, on_error)`:
   - Steam: Windows (HKCU SteamPath + manifests) or Linux (`~/.steam/steam`, `~/.local/share/Steam`)
   - GOG / Epic / Xbox: Windows registry and manifest paths (see docs/modules/game-detector.md)
   - Deduplicate games by `id`
3. SQLite — `insert_or_update_game` for each result; events `game_detected`, `game_detection_completed`
4. UI — store updates; dashboard renders cards
```

**Code pointers**:

- Frontend: `src/entities/game/api/gameApi.ts`, `src/entities/game/model/gameStore.ts`, `src/features/detect-games/`, `src/pages/dashboard/index.tsx`
- Backend: `src-tauri/src/commands/games.rs`, `src-tauri/src/services/game_detector.rs`

---

## Flow: Managed game (library → scope)

**Trigger**: User chooses a game to manage from the dashboard (e.g. **Manage** or opening the game scope).

**Flow**:

```
1. UI sets managedGameId in the game store and persists pantheon.managedGameId in localStorage.
2. Navigate to /game/:id/mods (canonical entry for game scope).
3. Sidebar: top area becomes the game banner (cover art + Play); below “General” (Games, Deployments, Settings); then the game block (Mods, Plugins, Saves). Collapse toggle stays at the bottom of the sidebar.
4. On app start, loadGames() then validate managedGameId; drop if the game no longer exists.
5. remove_game_from_library: if gameId === managedGameId, clear store + localStorage and redirect to /; sidebar header returns to Pantheon branding.
```

**Spec**: [modules/managed-game-context.md](./modules/managed-game-context.md)

**Code pointers**: `src/entities/game/model/gameStore.ts`, `src/pages/game-scope/`, `src/widgets/Sidebar/index.tsx`, `src/app/router/index.tsx`

---

## Flow 2: Mod Installation

**Trigger**: User selects mod archive (zip/7z/rar)

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         MOD INSTALLATION FLOW                           │
└─────────────────────────────────────────────────────────────────────────┘

1. UI Layer (Solid.js)
   │
   ├── User drops archive or clicks "Install Mod"
   │
   └── invoke('install_mod', { gameId, archivePath }) ─────────────────┐
                                                                    │
2. Rust Backend: ModInstaller                                         │
   │                                                                 │
   ├── Detect archive type                                           │
   │   ├── Check magic bytes for .bsa/.ba2                          │
   │   ├── Check for FOMOD manifest (fomod/info.xml)                │
   │   └── Check for BepInEx patterns                               │
   │                                                                 │
   ├── Extract archive                                               │
   │   ├── .zip → zip crate                                         │
   │   ├── .7z → sevenz-rust                                        │
   │   └── Copy to staging/mods/{modId}/                            │
   │                                                                 │
   ├── Parse mod metadata                                            │
   │   ├── Read mod.json if present                                 │
   │   └── Extract info from FOMOD if present                       │
   │                                                                 │
   ├── Validate files                                                │
   │   ├── Check for required files                                  │
   │   ├── Compute SHA256 hashes                                    │
   │   └── Run SecurityValidator                                     │
   │                                                                 │
   ├── Register mod in database                                      │
   │   └── INSERT INTO mods (...)                                   │
   │   └── INSERT INTO modFiles (...)                               │
   │                                                                 │
   └── Return Mod entity ◄────────────────────────────────────────────┘
                                                                    │
3. UI Layer                                                          │
   │                                                                 │
   ├── Receive installed mod                                        │
   ├── Update modStore                                              │
   ├── Show success toast                                           │
   └── Enable "Deploy Mods" button
```

**Code Implementation**:

### Frontend

```typescript
// features/install-mod/model/installMod.ts
export interface InstallModParams {
  gameId: string;
  archivePath: string;
}

export const installMod = async (params: InstallModParams): Promise<Mod> => {
  return invoke<Mod>('install_mod', params);
};

// features/install-mod/ui/InstallModButton.tsx
import { Component, createSignal } from 'solid-js';
import { DropZone } from '@/shared/ui/DropZone';
import { Button } from '@/shared/ui/Button';
import { installMod } from '../model/installMod';

interface InstallModButtonProps {
  gameId: string;
  onInstalled: (mod: Mod) => void;
}

export const InstallModButton: Component<InstallModButtonProps> = (props) => {
  const [isInstalling, setIsInstalling] = createSignal(false);
  const [selectedFile, setSelectedFile] = createSignal<string | null>(null);
  
  const handleDrop = async (filePath: string) => {
    setSelectedFile(filePath);
    setIsInstalling(true);
    
    try {
      const mod = await installMod({
        gameId: props.gameId,
        archivePath: filePath,
      });
      props.onInstalled(mod);
    } catch (error) {
      // Show error toast
    } finally {
      setIsInstalling(false);
    }
  };
  
  return (
    <DropZone onDrop={handleDrop}>
      <Button isLoading={isInstalling()} disabled={!selectedFile()}>
        Install Mod
      </Button>
    </DropZone>
  );
};
```

### Backend

```rust
// src-tauri/src/commands/mods.rs
#[tauri::command]
pub async fn install_mod(
    game_id: String,
    archive_path: String,
) -> Result<Mod, String> {
    let installer = ModInstaller::new();
    let options = InstallOptions {
        staging_path: get_staging_path(),
        game_support_path: get_game_support_path(&game_id)?,
        mod_type: None,
        file_overrides: HashMap::new(),
    };
    
    installer.install(&game_id, Path::new(&archive_path), options).await
}

// src-tauri/src/services/mod_installer.rs
pub struct ModInstaller;

impl ModInstaller {
    pub async fn install(
        &self,
        game_id: &str,
        archive_path: &Path,
        options: InstallOptions,
    ) -> Result<Mod, String> {
        // 1. Detect archive type
        let mod_type = self.detect_mod_type(archive_path)?;
        
        // 2. Create mod ID and staging path
        let mod_id = Uuid::new_v4().to_string();
        let staging_path = options.staging_path.join("mods").join(&mod_id);
        
        // 3. Extract archive
        let extracted_files = self.extract_archive(archive_path, &staging_path)?;
        
        // 4. Parse metadata
        let metadata = self.parse_mod_metadata(&staging_path, &mod_type)?;
        
        // 5. Validate
        let validator = SecurityValidator::new();
        validator.validate_mod(&staging_path).await?;
        
        // 6. Store in database
        let mod = Mod {
            id: mod_id.clone(),
            game_id: game_id.to_string(),
            name: metadata.name.unwrap_or_else(|| "Unknown Mod".to_string()),
            version: metadata.version,
            mod_type,
            install_path: staging_path.clone(),
            enabled: false,
            flags: vec!["installed".to_string()],
            attributes: HashMap::new(),
            install_time: Utc::now(),
            last_modified: Utc::now(),
            metadata: Some(metadata),
            conflicts: vec![],
            dependencies: vec![],
        };
        
        let db = Database::new().map_err(|e| e.to_string())?;
        db.insert_mod(&mod).map_err(|e| e.to_string())?;
        
        // 7. Store file records
        for file_path in &extracted_files {
            let file = ModFile {
                id: 0,
                mod_id: mod.id.clone(),
                path: file_path.strip_prefix(&staging_path)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string(),
                size: std::fs::metadata(file_path)
                    .map_err(|e| e.to_string())?
                    .len(),
                hash: None,
                is_archive: is_archive(file_path),
            };
            db.insert_mod_file(&file).map_err(|e| e.to_string())?;
        }
        
        Ok(mod)
    }
    
    fn detect_mod_type(&self, archive_path: &Path) -> Result<ModType, String> {
        let extension = archive_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "zip" => {
                // Check for FOMOD
                let file = std::fs::File::open(archive_path).map_err(|e| e.to_string())?;
                let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                
                if archive.by_name("fomod/info.xml").is_ok() {
                    return Ok(ModType::Fomod);
                }
                if archive.by_name("fomod/module_config.xml").is_ok() {
                    return Ok(ModType::Fomod);
                }
                
                // Check for BepInEx
                // ... similar checks
                
                Ok(ModType::Simple)
            },
            "7z" => Ok(ModType::Simple),  // sevenz-rust can detect more
            _ => Ok(ModType::Simple),
        }
    }
}
```

---

## Flow 3: Mod Deployment

**Trigger**: User clicks "Deploy Mods" or enables a mod

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         MOD DEPLOYMENT FLOW                             │
└─────────────────────────────────────────────────────────────────────────┘

1. UI Layer (Solid.js)
   │
   ├── User toggles mod or clicks "Deploy All"
   │
   └── invoke('deploy_mod', { modId }) ────────────────────────────────┐
                                                                      │
2. Rust Backend: DeployManager                                         │
   │                                                                      │
   ├── Get mod from database                                            │
   │                                                                      │
   ├── Check for file conflicts                                         │
   │   ├── Get list of all deployed files                              │
   │   ├── Compare with mod's files                                     │
   │   └── If conflicts exist:                                          │
   │       └── Emit conflict_detected event ──────────────────────────┐
   │                                                              │    │
   │                         User resolves in UI ◄─────────────────┘    │
   │                                                              │    │
   │                         invoke('resolve_conflicts', ...) ◄────────┘
   │                                                                      │
   ├── Determine deployment strategy                                    │
   │   ├── Get from settings (symlink/hardlink/copy)                   │
   │   └── Check if symlink possible (admin rights)                     │
   │                                                                      │
   ├── Create symlinks/hardlinks                                        │
   │   ├── For each file in mod:                                        │
   │   │   ├── source = staging/mods/{modId}/files/...                 │
   │   │   ├── target = gameSupportPath/Data/...                        │
   │   │   └── Create link (symlink → hardlink → copy fallback)       │
   │   │                                                                      │
   │   └── For Bethesda games:                                           │
   │       └── Trigger Archive Invalidation update                      │
   │                                                                      │
   ├── Update deployment state                                          │
   │   └── INSERT OR REPLACE INTO deployment (...)                     │
   │                                                                      │
   └── Emit deploy_completed ◄────────────────────────────────────────────┘
                                                                      │
3. UI Layer                                                              │
   │                                                                      │
   ├── listen('deploy_completed')                                        │
   ├── Update deploymentStore                                          │
   └── Show success toast
```

**Code Implementation**:

### Backend

```rust
// src-tauri/src/services/deploy_manager.rs
pub struct DeployManager {
    db: Database,
}

impl DeployManager {
    pub async fn deploy_mod(&self, mod_id: &str) -> Result<DeploymentState, String> {
        let db = self.db.clone();
        
        // 1. Get mod
        let mod = db.find_mod(mod_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Mod not found".to_string())?;
        
        // 2. Check conflicts
        let conflicts = self.check_conflicts(&mod.game_id, mod_id)?;
        if !conflicts.is_empty() {
            return Err(format!("Conflicts detected: {:?}", conflicts));
        }
        
        // 3. Get deployment files
        let files = db.get_mod_files(mod_id)
            .map_err(|e| e.to_string())?;
        
        // 4. Get game support path
        let game = db.find_game(&mod.game_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Game not found".to_string())?;
        
        let game_support_path = game.support_path;
        
        // 5. Deploy files
        let mut deployed_files = Vec::new();
        
        for file in files {
            let source = PathBuf::from(&mod.install_path).join(&file.path);
            let target = game_support_path.join(&file.path);
            
            // Create parent directories
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            
            // Try symlink first
            let link_type = match self.create_symlink(&source, &target) {
                Ok(lt) => lt,
                Err(e) => {
                    // Try hardlink
                    self.create_hardlink(&source, &target)
                        .map_err(|e| format!("Link failed: {}", e))?;
                    LinkType::Hardlink
                }
            };
            
            deployed_files.push(DeployedFile {
                source: file.path.clone(),
                target: file.path,
                link_type,
                size: file.size,
                hash: file.hash.unwrap_or_default(),
            });
        }
        
        // 6. Update deployment state
        let state = DeploymentState {
            mod_id: mod_id.to_string(),
            game_id: mod.game_id.clone(),
            status: DeployStatus::Deployed,
            strategy: DeployStrategy::Symlink,
            deployed_files: deployed_files,
            conflicts: vec![],
            deployed_at: Some(Utc::now()),
        };
        
        db.upsert_deployment(&state)
            .map_err(|e| e.to_string())?;
        
        // 7. Update mod enabled flag
        db.update_mod_enabled(mod_id, true)
            .map_err(|e| e.to_string())?;
        
        Ok(state)
    }
    
    fn create_symlink(&self, source: &Path, target: &Path) -> Result<LinkType, String> {
        use std::os::windows::fs::{symlink_file, symlink_dir};
        
        if source.is_dir() {
            symlink_dir(source, target).map_err(|e| e.to_string())?;
            Ok(LinkType::DirectoryJunction)
        } else {
            symlink_file(source, target).map_err(|e| e.to_string())?;
            Ok(LinkType::Symlink)
        }
    }
    
    fn create_hardlink(&self, source: &Path, target: &Path) -> Result<(), String> {
        std::fs::hard_link(source, target).map_err(|e| e.to_string())
    }
}
```

---

## Flow 4: Load Order Management (Bethesda)

**Trigger**: User views plugins page or deploys mods

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      LOAD ORDER MANAGEMENT FLOW                         │
└─────────────────────────────────────────────────────────────────────────┘

1. Plugin Discovery (on game activation)
   │
   ├── Read loadOrder table from database
   │
   ├── Scan game Data folder for .esp/.esm/.esl files
   │   └── Also check staging/deployed folders
   │
   ├── For each plugin:
   │   ├── Parse header (ESM flag, master list, etc.)
   │   ├── Check if in loadOrder table
   │   └── If not, add to table with default index
   │
   └── Store plugin info in plugins table
   │
   ▼
2. Load Order Display
   │
   ├── User opens Plugins page
   │
   ├── invoke('get_load_order', { gameId })
   │
   ├── Get loadOrder entries from database
   │
   ├── Get plugin info from plugins table
   │
   └── Display sortable list with drag-drop
   │
   ▼
3. Load Order Modification
   │
   ├── User drags plugin to new position
   │
   ├── invoke('set_load_order', { gameId, order: [...] })
   │
   ├── Update loadOrder table with new indices
   │
   ├── Write plugins.txt with enabled plugins
   │
   └── Emit load_order_changed event
   │
   ▼
4. Plugin Enable/Disable
   │
   ├── User toggles plugin
   │
   ├── If enabling:
   │   ├── Move plugin to end of load order
   │   └── Add to plugins.txt
   │
   ├── If disabling:
   │   ├── Rename to .ghost extension
   │   └── Remove from plugins.txt
   │
   └── Update database and emit event
```

**Code Implementation**:

### Backend

```rust
// src-tauri/src/services/load_order_manager.rs
pub struct LoadOrderManager {
    db: Database,
}

impl LoadOrderManager {
    pub fn get_load_order(&self, game_id: &str) -> Result<Vec<LoadOrderEntry>, String> {
        self.db.get_load_order(game_id)
    }
    
    pub fn set_load_order(
        &self,
        game_id: &str,
        entries: Vec<LoadOrderEntry>,
    ) -> Result<(), String> {
        self.db.set_load_order(game_id, &entries)
    }
    
    pub fn add_plugin(&self, game_id: &str, plugin_name: &str) -> Result<(), String> {
        let db = self.db.clone();
        
        // Get max load order index
        let max_index = db.get_max_load_order_index(game_id)?;
        
        let entry = LoadOrderEntry {
            game_id: game_id.to_string(),
            plugin_name: plugin_name.to_string(),
            load_order_index: max_index + 1,
            enabled: true,
            group_name: None,
        };
        
        db.insert_load_order_entry(&entry)
    }
    
    pub fn enable_plugin(&self, game_id: &str, plugin_name: &str) -> Result<(), String> {
        let plugin_path = self.get_plugin_path(game_id, plugin_name)?;
        
        // Check if .ghost exists
        let ghost_path = PathBuf::from(format!("{}.ghost", plugin_path.display()));
        
        if ghost_path.exists() {
            std::fs::rename(&ghost_path, &plugin_path)
                .map_err(|e| e.to_string())?;
        }
        
        self.update_plugins_txt(game_id)
    }
    
    pub fn disable_plugin(&self, game_id: &str, plugin_name: &str) -> Result<(), String> {
        let plugin_path = self.get_plugin_path(game_id, plugin_name)?;
        
        // Rename to .ghost
        let ghost_path = PathBuf::from(format!("{}.ghost", plugin_path.display()));
        
        std::fs::rename(&plugin_path, &ghost_path)
            .map_err(|e| e.to_string())?;
        
        self.update_plugins_txt(game_id)
    }
    
    fn update_plugins_txt(&self, game_id: &str) -> Result<(), String> {
        let game = self.db.find_game(game_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Game not found".to_string())?;
        
        let plugins_path = game.support_path.join("plugins.txt");
        
        // Get enabled plugins
        let entries = self.db.get_load_order(game_id)
            .map_err(|e| e.to_string())?;
        
        let enabled_plugins: Vec<String> = entries
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.plugin_name.clone())
            .collect();
        
        // Write plugins.txt
        let content = enabled_plugins.join("\n");
        std::fs::write(&plugins_path, content)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
```

---

## Flow 5: Download with Resume

**Trigger**: User clicks download from repository browser

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         DOWNLOAD FLOW                                   │
└─────────────────────────────────────────────────────────────────────────┘

1. Start Download
   │
   ├── User clicks download
   │
   ├── invoke('start_download', { url, destination })
   │
   ├── Create download record in database
   │   └── state = 'pending'
   │
   └── Return downloadId ◄──────────────────────────────┐
                                                        │
2. Probe Server (Download Manager)                      │
   │                                                        │
   ├── Send HEAD request                                │
   │                                                        │
   ├── Check Accept-Ranges header                        │
   │   ├── If Accept-Ranges: bytes                       │
   │   │   └── Use chunked download (resume supported)   │
   │   │                                                        │
   │   └── If no Range support                           │
   │       └── Use single-stream download               │
   │                                                        │
   └── Update download state to 'downloading'            │
   │                                                        │
3. Download with Progress ◄──────────────────────────────┘
   │
   ├── Emit 'download_progress' events:
   │   {
   │     downloadId: "...",
   │     bytesWritten: 1024000,
   │     bytesTotal: 10485760,
   │     speed: 102400,
   │     progressPercent: 9.76,
   │     state: "downloading"
   │   }
   │
   ├── If pause requested:
   │   ├── Store current offset in database
   │   └── Update state to 'paused'
   │
   └── On complete:
       ├── Update state to 'completed'
       ├── Store etag if provided
       └── Emit 'download_completed'
   │
   ▼
4. UI Updates (Solid.js)
   │
   ├── listen('download_progress', (progress) => {
   │   // Update download store
   │   // Show progress bar
   │ })
   │
   ├── listen('download_completed', (result) => {
   │   // Remove from queue
   │   // Trigger mod installation
   │   invoke('install_mod', { gameId, archivePath: result.filePath })
   │ })
```

**Code Implementation**:

### Backend

```rust
// src-tauri/src/services/download_manager.rs
pub struct DownloadManager {
    db: Database,
    app: AppHandle,
    client: reqwest::Client,
}

impl DownloadManager {
    pub async fn start_download(
        &self,
        url: &str,
        destination: &Path,
    ) -> Result<String, String> {
        let download_id = Uuid::new_v4().to_string();
        
        // Create download record
        let download = Download {
            id: download_id.clone(),
            file_name: Path::new(url)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            url: url.to_string(),
            destination: destination.to_path_buf(),
            state: DownloadState::Pending,
            bytes_written: 0,
            bytes_total: None,
            etag: None,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
        };
        
        self.db.insert_download(&download)
            .map_err(|e| e.to_string())?;
        
        // Start download task
        let download_id_clone = download_id.clone();
        let url_clone = url.to_string();
        let dest_clone = destination.to_path_buf();
        let app_clone = self.app.clone();
        
        tokio::spawn(async move {
            if let Err(e) = self.download_file(
                &download_id_clone,
                &url_clone,
                &dest_clone,
                &app_clone,
            ).await {
                // Update download with error
                self.db.update_download_error(&download_id_clone, &e)
                    .map_err(|e| log::error!("Failed to update error: {}", e));
                
                // Emit failure event
                app_clone.emit("download_failed", DownloadError {
                    download_id: download_id_clone,
                    error: e,
                }).map_err(|e| log::error!("Failed to emit: {}", e));
            }
        });
        
        Ok(download_id)
    }
    
    async fn download_file(
        &self,
        download_id: &str,
        url: &str,
        destination: &Path,
        app: &AppHandle,
    ) -> Result<(), String> {
        // Probe server for Range support
        let strategy = self.probe_server(url).await?;
        
        let mut total = 0u64;
        let mut downloaded = 0u64;
        
        match strategy {
            DownloadStrategy::Chunked { chunk_size } => {
                // Get total size
                let response = self.client.head(url).send().await
                    .map_err(|e| e.to_string())?;
                total = response.content_length().unwrap_or(0);
                
                // Create file
                let mut file = tokio::fs::File::create(destination).await
                    .map_err(|e| e.to_string())?;
                
                // Download in chunks
                while downloaded < total {
                    let end = (downloaded + chunk_size - 1).min(total - 1);
                    let range = format!("bytes={}-{}", downloaded, end);
                    
                    let mut response = self.client
                        .get(url)
                        .header("Range", range)
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    
                    while let Some(chunk) = response.chunk().await
                        .map_err(|e| e.to_string())?
                    {
                        file.write_all(&chunk).await
                            .map_err(|e| e.to_string())?;
                        downloaded += chunk.len() as u64;
                        
                        // Emit progress
                        app.emit("download_progress", DownloadProgress {
                            download_id: download_id.to_string(),
                            bytes_written: downloaded,
                            bytes_total: total,
                            speed: self.calculate_speed(downloaded),
                            progress_percent: (downloaded as f64 / total as f64) * 100.0,
                            state: DownloadState::Downloading,
                        }).map_err(|e| e.to_string())?;
                    }
                    
                    // Save progress for resume
                    self.db.update_download_progress(download_id, downloaded, total)
                        .map_err(|e| e.to_string())?;
                }
            },
            DownloadStrategy::Single => {
                // Simple download without resume
                let response = self.client.get(url).send().await
                    .map_err(|e| e.to_string())?;
                
                total = response.content_length().unwrap_or(0);
                let mut file = tokio::fs::File::create(destination).await
                    .map_err(|e| e.to_string())?;
                
                let mut stream = response.bytes_stream();
                while let Some(item) = stream.next().await
                    .map_err(|e| e.to_string())?
                {
                    file.write_all(&item).await
                        .map_err(|e| e.to_string())?;
                    downloaded += item.len() as u64;
                    
                    app.emit("download_progress", DownloadProgress {
                        download_id: download_id.to_string(),
                        bytes_written: downloaded,
                        bytes_total: total,
                        speed: self.calculate_speed(downloaded),
                        progress_percent: if total > 0 {
                            (downloaded as f64 / total as f64) * 100.0
                        } else { 0.0 },
                        state: DownloadState::Downloading,
                    }).map_err(|e| e.to_string())?;
                }
            }
        }
        
        // Update database
        self.db.update_download_completed(download_id)
            .map_err(|e| e.to_string())?;
        
        // Emit completion
        app.emit("download_completed", DownloadCompletedEvent {
            download_id: download_id.to_string(),
            file_path: destination.to_path_buf(),
        }).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
```

---

## Flow 6: Profile Switching

**Trigger**: User selects different profile

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        PROFILE SWITCHING FLOW                           │
└─────────────────────────────────────────────────────────────────────────┘

1. User selects profile
   │
   ├── Click on profile in dropdown
   │
   └── invoke('switch_profile', { profileId })
   │
2. Backend: ProfileManager
   │
   ├── Save current profile state
   │   └── modState = current mod enable/disable states
   │
   ├── Load new profile
   │   └── Get profile from database
   │
   ├── Update deployment for new profile
   │   │
   │   ├── For each mod in new profile:
   │   │   └── Deploy or undeploy based on enabled flag
   │   │
   │   └── For each mod not in profile:
   │       └── Ensure mod is undeployed
   │
   ├── Update load order
   │   └── Switch to profile-specific load order
   │
   ├── Write plugins.txt for new state
   │
   └── Emit profile_switched event ◄────────────────────────────────────┐
                                                                       │
3. Frontend updates ◄────────────────────────────────────────────────────┘
   │
   ├── listen('profile_switched', ({ profileId, gameId }) => {
   │   // Update activeProfileId in store
   │   // Refresh mod list
   │   // Refresh load order display
   │ })
   │
   └── Show success toast "Switched to {profileName}"
```

---

## Flow 7: Game Launch with Mods

**Trigger**: User clicks "Launch Game"

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         GAME LAUNCH FLOW                                │
└─────────────────────────────────────────────────────────────────────────┘

1. Pre-launch checks
   │
   ├── Check if any mods need deployment
   │   └── If deployment pending, prompt user
   │
   ├── Verify game files exist
   │
   └── Check for required loaders (SKSE, etc.)
   │
2. Ensure all mods deployed
   │
   ├── invoke('deploy_all', { gameId })
   │
   └── Wait for deployment to complete
   │
3. Detect available loaders
   │
   ├── invoke('detect_loaders', { gameId })
   │
   ├── Check for SKSE/F4SE/NVSE/etc.
   │
   └── Return loader info
   │
4. Build launch arguments
   │
   ├── If script extender found:
   │   └── Use skse_loader.exe or equivalent
   │
   ├── Add profile-specific arguments
   │
   └── Construct full command line
   │
5. Launch process
   │
   ├── std::process::Command::new(launcher_exe)
   │   .args(launcher_args)
   │   .spawn()
   │
   └── Return process ID ◄───────────────────────────────────────────────┐
                                                                        │
6. Monitor game process ◄────────────────────────────────────────────────┘
   │
   ├── Spawn monitoring task
   │   └── Wait for process to exit
   │
   ├── On exit:
   │   ├── emit('game_exited', { gameId, exitCode })
   │   └── Optionally run post-launch tasks (AI backup, etc.)
```

---

## Flow 8: Conflict Resolution

**Trigger**: User tries to enable conflicting mod

**Flow**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      CONFLICT RESOLUTION FLOW                          │
└─────────────────────────────────────────────────────────────────────────┘

1. Detect conflict
   │
   ├── User enables mod with conflicting file
   │
   ├── invoke('deploy_mod', { modId })
   │
   ├── DeployManager detects conflict
   │   └── File already deployed by another mod
   │
   └── Emit 'conflict_detected' event ◄──────────────────────────────────┐
   │                                                                  │
   └── listen('conflict_detected', ({ gameId, conflicts }) => {        │
       // Show conflict dialog                                         │
   })                                                                  │
   │                                                                  │
2. Show resolution UI ◄────────────────────────────────────────────────┘
   │
   ├── Display conflict dialog:
   │   ├── "File texture.dds is in conflict"
   │   ├── "Mod A provides version 1 (1024 KB)"
   │   ├── "Mod B provides version 2 (2048 KB)"
   │   └── Options: [Use A] [Use B] [Merge] [Skip]
   │
   └── User selects resolution
   │
3. Apply resolution
   │
   ├── invoke('resolve_conflicts', { gameId, resolutions: [...] })
   │
   ├── Backend:
   │   ├── Store resolution in database
   │   ├── Apply winning file
   │   └── If merge, create merged file
   │
   ├── Continue deployment
   │
   └── Emit 'conflict_resolved'
   │
4. Complete deployment
   │
   ├── Deployment continues with resolved conflicts
   │
   └── Emit 'deploy_completed' with conflict info
```

---

## Common Error Flows

### Archive Extraction Fails

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ERROR: EXTRACTION FAILED                        │
└─────────────────────────────────────────────────────────────────────────┘

1. Detection
   │
   ├── zip::ZipArchive fails or file corrupted
   │
   └── Return Err("Failed to extract archive: ...")
   │
2. Handling
   │
   ├── Catch error in command handler
   │
   ├── Log error with details
   │
   ├── Emit 'mod_install_failed' event
   │
   └── Return Err to UI
   │
3. UI
   │
   ├── Show error toast with message
   │
   └── Offer retry or cancel
```

### Database Error

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ERROR: DATABASE                                 │
└─────────────────────────────────────────────────────────────────────────┘

1. Detection
   │
   ├── rusqlite error (constraint violation, etc.)
   │
   └── Return Err from database layer
   │
2. Handling
   │
   ├── Map to user-friendly error
   │   ├── "MOD_ALREADY_INSTALLED"
   │   ├── "DATABASE_ERROR"
   │   └── "CONSTRAINT_VIOLATION"
   │
   └── Log with full context
   │
3. Recovery
   │
   ├── Retry with backoff for transient errors
   │
   └── For constraint errors, prompt user
```

### Symlink Permission Denied

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      ERROR: SYMLINK PERMISSION                          │
└─────────────────────────────────────────────────────────────────────────┘

1. Detection
   │
   ├── std::io::Error with error_code = EPERM
   │
   └── On Windows: missing admin or Developer Mode
   │
2. Handling
   │
   ├── Fall back to hardlink
   │   └── If same filesystem
   │
   ├── If hardlink fails
   │   └── Fall back to copy
   │
   ├── Log warning about performance
   │
   └── Continue deployment
   │
3. Settings
   │
   ├── Suggest enabling Developer Mode
   │   └── Open Settings → System → For developers
   │
   └── Or change deployment strategy to copy
```