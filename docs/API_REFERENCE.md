# Pantheon API Reference

## Overview

Complete reference for Tauri commands (invoke) and events (listen/emit).

### IPC JSON field names

All structured payloads (`Game`, `Mod`, `DetectionProgress`, `GameDetectionError`, `DeploymentState`, …) use **camelCase** property names in JSON when crossing the Tauri boundary, matching TypeScript types in `src/shared/types.ts`. Rust structs keep `snake_case` field names in source; `serde` attributes on `src-tauri/src/models.rs` perform the mapping. Enum values for `GameLauncher` and `ModSupportLevel` are **lowercase** strings (`"steam"`, `"ubisoft"`, `"microsoftstore"`, `"none"`, …).

See **MODELS.md → JSON over Tauri IPC** for the full rules and database JSON compatibility notes.

---

## Implemented Commands (Phase 1-2)

47 commands currently registered in `src-tauri/src/lib.rs`. These are actually compiled and available for `invoke()`.

### Game Commands

#### get_games

```rust
#[tauri::command]
pub async fn get_games() -> Result<Vec<Game>, String>
```

**Returns**: All registered games from database.

#### get_game

```rust
#[tauri::command]
pub async fn get_game(game_id: String) -> Result<Option<Game>, String>
```

**Returns**: Single game by ID, or `null`.

#### get_game_install_stats

```rust
#[tauri::command]
pub async fn get_game_install_stats(game_id: String) -> Result<GameInstallStats, String>
```

**Returns**: Disk usage, Steam metadata, PE version string.

#### detect_games

```rust
#[tauri::command]
pub async fn detect_games() -> Result<Vec<Game>, String>
```

**Returns**: Newly discovered games from registry scanning.

#### scan_custom_path

```rust
#[tauri::command]
pub async fn scan_custom_path(path: String) -> Result<Vec<Game>, String>
```

**Returns**: Games found in user-selected directory.

#### register_game

```rust
#[tauri::command]
pub async fn register_game(game: Game) -> Result<Game, String>
```

#### unregister_game

```rust
#[tauri::command]
pub async fn unregister_game(game_id: String) -> Result<(), String>
```

#### remove_game_from_library

```rust
#[tauri::command]
pub async fn remove_game_from_library(game_id: String) -> Result<RemoveGameResult, String>
```

**Returns**: `{ deletedMods: number }` — count of deleted mods.

### Mod Commands

#### install_mod

```rust
#[tauri::command]
pub async fn install_mod(game_id: String, archive_path: String) -> Result<Mod, String>
```

#### uninstall_mod

```rust
#[tauri::command]
pub async fn uninstall_mod(mod_id: String) -> Result<(), String>
```

#### get_mods

```rust
#[tauri::command]
pub async fn get_mods(game_id: String) -> Result<Vec<Mod>, String>
```

#### set_mod_enabled

```rust
#[tauri::command]
pub async fn set_mod_enabled(mod_id: String, enabled: bool, strategy: Option<String>) -> Result<(), String>
```

**Parameters**:
- `strategy: "auto" | "symlink" | "hardlink" | "copy"` (optional, defaults to "auto")

### Deploy Commands

#### deploy_mod

```rust
#[tauri::command]
pub async fn deploy_mod(mod_id: String, strategy: Option<String>) -> Result<DeploymentState, String>
```

**Parameters**:
- `strategy: "auto" | "symlink" | "hardlink" | "copy"` (optional, defaults to "auto")

#### undeploy_mod

```rust
#[tauri::command]
pub async fn undeploy_mod(mod_id: String) -> Result<(), String>
```

#### deploy_all

```rust
#[tauri::command]
pub async fn deploy_all(game_id: String, strategy: Option<String>) -> Result<Vec<DeploymentState>, String>
```

#### check_conflicts

```rust
#[tauri::command]
pub async fn check_conflicts(game_id: String) -> Result<Vec<Conflict>, String>
```

**Returns**: `{ filePath: string, modA: string, modB: string }[]`

### Download Commands

#### start_download

```rust
#[tauri::command]
pub async fn start_download(url: String, file_name: String, game_id: Option<String>) -> Result<String, String>
```

**Returns**: `download_id` (UUID).

#### pause_download

```rust
#[tauri::command]
pub async fn pause_download(download_id: String) -> Result<(), String>
```

#### resume_download

```rust
#[tauri::command]
pub async fn resume_download(download_id: String) -> Result<(), String>
```

#### cancel_download

```rust
#[tauri::command]
pub async fn cancel_download(download_id: String) -> Result<(), String>
```

#### get_download

```rust
#[tauri::command]
pub async fn get_download(download_id: String) -> Result<Option<Download>, String>
```

#### list_downloads

```rust
#[tauri::command]
pub async fn list_downloads() -> Result<Vec<Download>, String>
```

#### list_download_queue

```rust
#[tauri::command]
pub async fn list_download_queue() -> Result<Vec<Download>, String>
```

**Returns**: Downloads in `pending` / `downloading` / `paused` state.

### Load Order Commands

#### refresh_plugin_list

```rust
#[tauri::command]
pub async fn refresh_plugin_list(game_id: String) -> Result<Vec<PluginInfo>, String>
```

Scans game data directory, merges with DB state.

#### get_load_order

```rust
#[tauri::command]
pub async fn get_load_order(game_id: String) -> Result<Vec<PluginInfo>, String>
```

#### set_plugin_enabled

```rust
#[tauri::command]
pub async fn set_plugin_enabled(game_id: String, plugin_name: String, enabled: bool) -> Result<(), String>
```

#### move_plugin

```rust
#[tauri::command]
pub async fn move_plugin(game_id: String, plugin_name: String, new_index: u32) -> Result<(), String>
```

#### auto_sort_plugins

```rust
#[tauri::command]
pub async fn auto_sort_plugins(game_id: String) -> Result<Vec<PluginInfo>, String>
```

Sorts ESM → ESL → ESP (alphabetical within each group).

#### write_plugins_txt

```rust
#[tauri::command]
pub async fn write_plugins_txt(game_id: String) -> Result<String, String>
```

Writes `%LOCALAPPDATA%/game_id/plugins.txt`. Enabled plugins get `*` prefix.

#### read_plugins_txt

```rust
#[tauri::command]
pub async fn read_plugins_txt(game_id: String) -> Result<Vec<String>, String>
```

#### set_plugin_ghost

```rust
#[tauri::command]
pub async fn set_plugin_ghost(game_id: String, plugin_name: String, ghosted: bool) -> Result<(), String>
```

### Game Launcher Commands

#### launch_game

```rust
#[tauri::command]
pub fn launch_game(game_id: String, loader_id: Option<String>) -> Result<LaunchResult, String>
```

**Returns**: `{ processId: number, loaderUsed: string | null }`

#### detect_game_loaders

```rust
#[tauri::command]
pub fn detect_game_loaders(game_id: String) -> Result<Vec<LoaderInfo>, String>
```

**Returns**: Available loaders (SKSE, F4SE, BepInEx, etc.).

#### is_game_running

```rust
#[tauri::command]
pub fn is_game_running(game_id: String) -> bool
```

#### list_running_games

```rust
#[tauri::command]
pub fn list_running_games() -> Vec<RunningGame>
```

#### get_running_game

```rust
#[tauri::command]
pub fn get_running_game(game_id: String) -> Option<RunningGame>
```

#### kill_game

```rust
#[tauri::command]
pub fn kill_game(game_id: String) -> Result<(), String>
```

### Game Content Commands

#### list_game_plugins

```rust
#[tauri::command]
pub async fn list_game_plugins(game_id: String) -> Result<Vec<String>, String>
```

Returns ESP/ESM/ESL file names from game data directory.

#### list_game_saves

```rust
#[tauri::command]
pub async fn list_game_saves(game_id: String) -> Result<Vec<SaveFileEntry>, String>
```

#### delete_save

```rust
#[tauri::command]
pub async fn delete_save(game_id: String, save_path: String) -> Result<(), String>
```

#### backup_save

```rust
#[tauri::command]
pub async fn backup_save(game_id: String, save_path: String) -> Result<String, String>
```

#### restore_save

```rust
#[tauri::command]
pub async fn restore_save(game_id: String, backup_path: String) -> Result<(), String>
```

#### list_save_backups

```rust
#[tauri::command]
pub async fn list_save_backups(game_id: String) -> Result<Vec<SaveBackupEntry>, String>
```

#### get_saves_dir_path

```rust
#[tauri::command]
pub async fn get_saves_dir_path(game_id: String) -> Result<Option<String>, String>
```

### System Commands

#### open_folder

```rust
#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String>
```

Opens folder in system file explorer.

### Extension Commands

#### list_extensions

```rust
#[tauri::command]
pub fn list_extensions() -> Result<Vec<ExtensionInfo>, String>
```

#### get_extension_info

```rust
#[tauri::command]
pub fn get_extension_info(extension_id: String) -> Result<Option<ExtensionInfo>, String>
```

---

## Planned Commands (Future Phases)

Commands planned for future phases. Not yet implemented.

### Game Commands

#### get_games

```rust
#[tauri::command]
pub async fn get_games() -> Result<Vec<Game>, String>
```

**Returns**: All registered games from database.

**Example**:
```typescript
const games = await invoke<Game[]>('get_games');
```

---

#### get_game_install_stats

```rust
#[tauri::command]
pub async fn get_game_install_stats(game_id: String) -> Result<GameInstallStats, String>
```

**Parameters**:
- `gameId: string` — id игры в каталоге Pantheon.

**Returns**: `GameInstallStats`: два размера папки установки (`diskUsageBytes` — с обходом симлинков; `diskUsageBytesNoSymlinks` — без); на Windows строка `installedVersionLabel` из ресурса версии главного `.exe` (`requiredFiles`), иначе запасной вариант `Steam build …` из `appmanifest_<appId>.acf`; для Steam также `steamSizeOnDiskBytes`, `steamBuildId` (см. [Vortex](https://github.com/Nexus-Mods/Vortex)).

**Example**:
```typescript
const stats = await invoke<GameInstallStats>('get_game_install_stats', { gameId: 'skyrimse' });
```

---

#### detect_games

```rust
#[tauri::command]
pub async fn detect_games() -> Result<Vec<Game>, String>
```

**Returns**: Newly discovered games from registry scanning.

**Example**:
```typescript
const newGames = await invoke<Game[]>('detect_games');
```

---

#### register_game

```rust
#[tauri::command]
pub async fn register_game(game: Game) -> Result<Game, String>
```

**Parameters**:
- `game: Game` - Game object to register

**Returns**: Registered game with ID.

---

#### select_game

```rust
#[tauri::command]
pub async fn select_game(game_id: String) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID to select

**Example**:
```typescript
await invoke('select_game', { gameId: 'skyrim' });
```

---

#### unregister_game

```rust
#[tauri::command]
pub async fn unregister_game(game_id: String) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID to remove

---

### Mod Commands

#### install_mod

```rust
#[tauri::command]
pub async fn install_mod(
    game_id: String,
    archive_path: String,
) -> Result<Mod, String>
```

**Parameters**:
- `gameId: string` - Target game ID
- `archivePath: string` - Path to mod archive

**Returns**: Installed mod object.

**Example**:
```typescript
const mod = await invoke<Mod>('install_mod', {
  gameId: 'skyrim',
  archivePath: 'C:/Downloads/mod.zip'
});
```

---

#### uninstall_mod

```rust
#[tauri::command]
pub async fn uninstall_mod(mod_id: String) -> Result<(), String>
```

**Parameters**:
- `modId: string` - Mod ID to remove

---

#### get_mods

```rust
#[tauri::command]
pub async fn get_mods(game_id: String) -> Result<Vec<Mod>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: All mods for the specified game.

---

#### list_game_plugins

```rust
#[tauri::command]
pub async fn list_game_plugins(game_id: String, state: State<'_, AppState>) -> Result<Vec<String>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Sorted plugin file names (`.esp`, `.esm`, `.esl`) under the game `support_path`.

---

#### list_game_saves

```rust
#[tauri::command]
pub async fn list_game_saves(game_id: String, state: State<'_, AppState>) -> Result<Vec<SaveFileEntry>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Entries `{ name, path }` for save files in the known Windows Documents save folder for supported catalog game IDs; empty if unmapped or missing.

---

#### get_mod

```rust
#[tauri::command]
pub async fn get_mod(mod_id: String) -> Result<Option<Mod>, String>
```

**Parameters**:
- `modId: string` - Mod ID

**Returns**: Mod object or null if not found.

---

#### set_mod_enabled

```rust
#[tauri::command]
pub async fn set_mod_enabled(
    mod_id: String,
    enabled: bool,
) -> Result<(), String>
```

**Parameters**:
- `modId: string` - Mod ID
- `enabled: boolean` - Enable/disable state

---

#### update_mod

```rust
#[tauri::command]
pub async fn update_mod(mod_id: String, updates: ModUpdates) -> Result<Mod, String>
```

**Parameters**:
- `modId: string` - Mod ID
- `updates: ModUpdates` - Fields to update

---

### Deployment Commands

#### deploy_mod

```rust
#[tauri::command]
pub async fn deploy_mod(mod_id: String) -> Result<DeploymentState, String>
```

**Parameters**:
- `modId: string` - Mod ID to deploy

**Returns**: Deployment state with deployed files.

---

#### undeploy_mod

```rust
#[tauri::command]
pub async fn undeploy_mod(mod_id: String) -> Result<(), String>
```

**Parameters**:
- `modId: string` - Mod ID to undeploy

---

#### deploy_all

```rust
#[tauri::command]
pub async fn deploy_all(game_id: String) -> Result<Vec<DeploymentState>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: All deployment states for the game.

---

#### get_deployment_state

```rust
#[tauri::command]
pub async fn get_deployment_state(game_id: String) -> Result<Vec<DeploymentState>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: All deployment states for the game.

---

#### set_deployment_strategy

```rust
#[tauri::command]
pub async fn set_deployment_strategy(
    game_id: String,
    strategy: DeployStrategy,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `strategy: DeployStrategy` - Strategy (symlink, hardlink, copy, merge)

---

#### resolve_conflicts

```rust
#[tauri::command]
pub async fn resolve_conflicts(
    game_id: String,
    resolutions: Vec<ConflictResolution>,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `resolutions: ConflictResolution[]` - Resolution choices

---

### Load Order Commands

#### get_load_order

```rust
#[tauri::command]
pub async fn get_load_order(game_id: String) -> Result<Vec<LoadOrderEntry>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Sorted list of plugins with load order indices.

---

#### set_load_order

```rust
#[tauri::command]
pub async fn set_load_order(
    game_id: String,
    order: Vec<LoadOrderEntry>,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `order: LoadOrderEntry[]` - New load order

---

#### get_plugin_info

```rust
#[tauri::command]
pub async fn get_plugin_info(
    game_id: String,
    plugin_name: String,
) -> Result<PluginInfo, String>
```

**Parameters**:
- `gameId: string` - Game ID
- `pluginName: string` - Plugin file name

**Returns**: Parsed plugin information (masters, flags, etc.).

---

#### set_plugin_enabled

```rust
#[tauri::command]
pub async fn set_plugin_enabled(
    game_id: String,
    plugin_name: String,
    enabled: bool,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `pluginName: string` - Plugin file name
- `enabled: boolean` - Enable/disable

---

#### auto_sort_plugins

```rust
#[tauri::command]
pub async fn auto_sort_plugins(game_id: String) -> Result<Vec<LoadOrderEntry>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Auto-sorted load order based on LOOT metadata.

---

#### set_plugin_ghost

```rust
#[tauri::command]
pub async fn set_plugin_ghost(
    game_id: String,
    plugin_name: String,
    ghosted: bool,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `pluginName: string` - Plugin file name
- `ghosted: boolean` - Add or remove .ghost extension

---

#### convert_plugin_light

```rust
#[tauri::command]
pub async fn convert_plugin_light(
    game_id: String,
    plugin_name: String,
    to_light: bool,
) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID
- `pluginName: string` - Plugin file name
- `toLight: boolean` - Convert to light or regular

---

### Download Commands

#### start_download

```rust
#[tauri::command]
pub async fn start_download(
    url: String,
    destination: String,
) -> Result<String, String>
```

**Parameters**:
- `url: string` - Download URL
- `destination: string` - Destination path

**Returns**: Download ID for tracking.

---

#### pause_download

```rust
#[tauri::command]
pub async fn pause_download(download_id: String) -> Result<(), String>
```

**Parameters**:
- `downloadId: string` - Download ID

---

#### resume_download

```rust
#[tauri::command]
pub async fn resume_download(download_id: String) -> Result<(), String>
```

**Parameters**:
- `downloadId: string` - Download ID

---

#### cancel_download

```rust
#[tauri::command]
pub async fn cancel_download(download_id: String) -> Result<(), String>
```

**Parameters**:
- `downloadId: string` - Download ID

---

#### get_download_progress

```rust
#[tauri::command]
pub async fn get_download_progress(download_id: String) -> Result<DownloadProgress, String>
```

**Parameters**:
- `downloadId: string` - Download ID

**Returns**: Current download progress.

---

#### list_downloads

```rust
#[tauri::command]
pub async fn list_downloads() -> Result<Vec<Download>, String>
```

**Returns**: All downloads (pending, active, completed).

---

### Profile Commands

#### get_profiles

```rust
#[tauri::command]
pub async fn get_profiles(game_id: String) -> Result<Vec<Profile>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: All profiles for the game.

---

#### create_profile

```rust
#[tauri::command]
pub async fn create_profile(
    game_id: String,
    name: String,
    copy_from: Option<String>,
) -> Result<Profile, String>
```

**Parameters**:
- `gameId: string` - Game ID
- `name: string` - Profile name
- `copyFrom?: string` - Optional profile ID to copy from

**Returns**: Created profile.

---

#### switch_profile

```rust
#[tauri::command]
pub async fn switch_profile(profile_id: String) -> Result<(), String>
```

**Parameters**:
- `profileId: string` - Profile ID to activate

---

#### delete_profile

```rust
#[tauri::command]
pub async fn delete_profile(profile_id: String) -> Result<(), String>
```

**Parameters**:
- `profileId: string` - Profile ID to delete

---

#### export_profile

```rust
#[tauri::command]
pub async fn export_profile(
    profile_id: String,
    path: String,
) -> Result<(), String>
```

**Parameters**:
- `profileId: string` - Profile ID
- `path: string` - Export file path

---

#### import_profile

```rust
#[tauri::command]
pub async fn import_profile(
    game_id: String,
    path: String,
) -> Result<Profile, String>
```

**Parameters**:
- `gameId: string` - Target game ID
- `path: string` - Import file path

**Returns**: Imported profile.

---

### Game Launcher Commands

#### launch_game

```rust
#[tauri::command]
pub async fn launch_game(
    game_id: String,
    profile_id: Option<String>,
) -> Result<u32, String>
```

**Parameters**:
- `gameId: string` - Game ID
- `profileId?: string` - Optional profile ID

**Returns**: Process ID of launched game.

---

#### detect_loaders

```rust
#[tauri::command]
pub async fn detect_loaders(game_id: String) -> Result<Vec<LoaderInfo>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Available script extenders and loaders.

---

#### is_game_running

```rust
#[tauri::command]
pub async fn is_game_running(game_id: String) -> Result<bool, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: True if game process is running.

---

#### kill_game

```rust
#[tauri::command]
pub async fn kill_game(game_id: String) -> Result<(), String>
```

**Parameters**:
- `gameId: string` - Game ID

---

### Backup Commands

#### create_backup

```rust
#[tauri::command]
pub async fn create_backup(
    game_id: String,
    backup_type: BackupType,
) -> Result<Backup, String>
```

**Parameters**:
- `gameId: string` - Game ID
- `backupType: BackupType` - Type (full, saves, config, mods)

**Returns**: Created backup info.

---

#### restore_backup

```rust
#[tauri::command]
pub async fn restore_backup(backup_id: String) -> Result<(), String>
```

**Parameters**:
- `backupId: string` - Backup ID

---

#### list_backups

```rust
#[tauri::command]
pub async fn list_backups(game_id: String) -> Result<Vec<Backup>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: All backups for the game.

---

### Settings Commands

#### get_settings

```rust
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String>
```

**Returns**: Current application settings.

---

#### update_settings

```rust
#[tauri::command]
pub async fn update_settings(settings: AppSettings) -> Result<(), String>
```

**Parameters**:
- `settings: AppSettings` - New settings

---

### Validation Commands

#### validate_mod

```rust
#[tauri::command]
pub async fn validate_mod(mod_id: String) -> Result<ValidationResult, String>
```

**Parameters**:
- `modId: string` - Mod ID to validate

**Returns**: Validation result with issues.

---

#### scan_file

```rust
#[tauri::command]
pub async fn scan_file(file_path: String) -> Result<ValidationResult, String>
```

**Parameters**:
- `filePath: string` - File to scan

**Returns**: Scan result.

---

### Repository Commands

#### search_mods

```rust
#[tauri::command]
pub async fn search_mods(
    query: String,
    game_id: Option<String>,
    page: u32,
    page_size: u32,
) -> Result<SearchResults, String>
```

**Parameters**:
- `query: string` - Search query
- `gameId?: string` - Optional game filter
- `page: number` - Page number (0-indexed)
- `pageSize: number` - Results per page

**Returns**: Search results with total count.

---

#### download_mod

```rust
#[tauri::command]
pub async fn download_mod(
    mod_id: String,
    file_id: Option<String>,
) -> Result<String, String>
```

**Parameters**:
- `modId: string` - Mod ID
- `fileId?: string` - Optional specific file ID

**Returns**: Download URL.

---

### Dependency Commands

#### resolve_dependencies

```rust
#[tauri::command]
pub async fn resolve_dependencies(mod_id: String) -> Result<Vec<String>, String>
```

**Parameters**:
- `modId: string` - Mod ID

**Returns**: List of required mod IDs.

---

#### check_conflicts

```rust
#[tauri::command]
pub async fn check_conflicts(mod_id: String) -> Result<Vec<Conflict>, String>
```

**Parameters**:
- `modId: string` - Mod ID

**Returns**: List of conflicts.

---

### Update Commands

#### check_for_updates

```rust
#[tauri::command]
pub async fn check_for_updates(game_id: String) -> Result<Vec<ModUpdateInfo>, String>
```

**Parameters**:
- `gameId: string` - Game ID

**Returns**: Available updates for game's mods.

---

#### pin_version

```rust
#[tauri::command]
pub async fn pin_version(mod_id: String, version: String) -> Result<(), String>
```

**Parameters**:
- `modId: string` - Mod ID
- `version: string` - Version to pin

---

---

## Events

### Download Events

#### download_progress

**Direction**: Rust → UI

**Payload**:
```typescript
interface DownloadProgress {
  downloadId: string;
  bytesWritten: number;
  bytesTotal: number;
  speed: number;
  progressPercent: number;
  state: DownloadState;
}
```

**Example**:
```typescript
listen<DownloadProgress>('download_progress', (progress) => {
  updateProgressBar(progress.progressPercent);
});
```

---

#### download_completed

**Direction**: Rust → UI

**Payload**:
```typescript
interface DownloadCompleted {
  downloadId: string;
  filePath: string;
}
```

---

#### download_failed

**Direction**: Rust → UI

**Payload**:
```typescript
interface DownloadError {
  downloadId: string;
  error: string;
}
```

---

### Mod Events

#### mod_installed

**Direction**: Rust → UI

**Payload**:
```typescript
interface ModInstalled {
  modId: string;
  gameId: string;
  modType: ModType;
}
```

---

#### mod_uninstalled

**Direction**: Rust → UI

**Payload**:
```typescript
interface ModUninstalled {
  modId: string;
  gameId: string;
}
```

---

#### mod_updated

**Direction**: Rust → UI

**Payload**:
```typescript
interface ModUpdated {
  modId: string;
  changes: Partial<Mod>;
}
```

---

### Deployment Events

#### deploy_completed

**Direction**: Rust → UI

**Payload**:
```typescript
interface DeployCompleted {
  modId: string;
  status: DeployStatus;
  deployedFiles: DeployedFile[];
}
```

---

#### deploy_failed

**Direction**: Rust → UI

**Payload**:
```typescript
interface DeployFailed {
  modId: string;
  error: string;
}
```

---

#### conflict_detected

**Direction**: Rust → UI

**Payload**:
```typescript
interface ConflictDetected {
  gameId: string;
  conflicts: Conflict[];
}
```

---

### Load Order Events

#### load_order_changed

**Direction**: Rust → UI

**Payload**:
```typescript
interface LoadOrderChanged {
  gameId: string;
  order: LoadOrderEntry[];
}
```

---

#### plugin_enabled

**Direction**: Rust → UI

**Payload**:
```typescript
interface PluginEnabled {
  gameId: string;
  pluginName: string;
}
```

---

#### plugin_disabled

**Direction**: Rust → UI

**Payload**:
```typescript
interface PluginDisabled {
  gameId: string;
  pluginName: string;
}
```

---

### Game Events

#### game_launched

**Direction**: Rust → UI

**Payload**:
```typescript
interface GameLaunched {
  gameId: string;
  processId: number;
}
```

---

#### game_exited

**Direction**: Rust → UI

**Payload**:
```typescript
interface GameExited {
  gameId: string;
  exitCode: number;
}
```

---

#### game_crashed

**Direction**: Rust → UI

**Payload**:
```typescript
interface GameCrashed {
  gameId: string;
  signal: number;
}
```

---

### Validation Events

#### validation_complete

**Direction**: Rust → UI

**Payload**:
```typescript
interface ValidationComplete {
  modId: string;
  isValid: boolean;
  issues: ValidationIssue[];
}
```

---

#### malware_detected

**Direction**: Rust → UI

**Payload**:
```typescript
interface MalwareDetected {
  modId: string;
  filePath: string;
  threatType: string;
}
```

---

### Profile Events

#### profile_switched

**Direction**: Rust → UI

**Payload**:
```typescript
interface ProfileSwitched {
  profileId: string;
  gameId: string;
}
```

---

#### profile_created

**Direction**: Rust → UI

**Payload**:
```typescript
interface ProfileCreated {
  profile: Profile;
}
```

---

### Backup Events

#### backup_created

**Direction**: Rust → UI

**Payload**:
```typescript
interface BackupCreated {
  backup: Backup;
}
```

---

#### backup_restored

**Direction**: Rust → UI

**Payload**:
```typescript
interface BackupRestored {
  backupId: string;
  gameId: string;
}
```

---

### Update Events

#### updates_available

**Direction**: Rust → UI

**Payload**:
```typescript
interface UpdatesAvailable {
  updates: ModUpdateInfo[];
}
```

---

### Repository Events

#### repo_auth_complete

**Direction**: Rust → UI

**Payload**:
```typescript
interface RepoAuthComplete {
  token: string;
  expiresAt: string;
}
```

---

### Extension Events

#### extension_loaded

**Direction**: Rust → UI

**Payload**:
```typescript
interface ExtensionLoaded {
  extensionId: string;
  name: string;
  version: string;
}
```

---

#### extension_error

**Direction**: Rust → UI

**Payload**:
```typescript
interface ExtensionError {
  extensionId: string;
  error: string;
}
```

---

## TypeScript Event Helpers

```typescript
// src/shared/api/events.ts
import { listen, emit } from '@tauri-apps/api/event';

// Download progress
export const onDownloadProgress = (handler: (progress: DownloadProgress) => void) =>
  listen<DownloadProgress>('download_progress', (e) => handler(e.payload));

// Download completed
export const onDownloadCompleted = (handler: (result: DownloadCompleted) => void) =>
  listen<DownloadCompleted>('download_completed', (e) => handler(e.payload));

// Mod installed
export const onModInstalled = (handler: (mod: ModInstalled) => void) =>
  listen<ModInstalled>('mod_installed', (e) => handler(e.payload));

// Deploy completed
export const onDeployCompleted = (handler: (state: DeployCompleted) => void) =>
  listen<DeployCompleted>('deploy_completed', (e) => handler(e.payload));

// Conflict detected
export const onConflictDetected = (handler: (conflicts: ConflictDetected) => void) =>
  listen<ConflictDetected>('conflict_detected', (e) => handler(e.payload));

// Game launched
export const onGameLaunched = (handler: (info: GameLaunched) => void) =>
  listen<GameLaunched>('game_launched', (e) => handler(e.payload));

// Game exited
export const onGameExited = (handler: (info: GameExited) => void) =>
  listen<GameExited>('game_exited', (e) => handler(e.payload));

// Profile switched
export const onProfileSwitched = (handler: (info: ProfileSwitched) => void) =>
  listen<ProfileSwitched>('profile_switched', (e) => handler(e.payload));

// Validation complete
export const onValidationComplete = (handler: (result: ValidationComplete) => void) =>
  listen<ValidationComplete>('validation_complete', (e) => handler(e.payload));
```

---

## Error Handling

All commands return `Result<T, String>`. Errors are propagated as strings.

```typescript
try {
  const mod = await invoke<Mod>('install_mod', { gameId, archivePath });
} catch (error) {
  // error is a string message
  if (error.includes('CONFLICT_DETECTED')) {
    // Handle conflict
  } else if (error.includes('MOD_NOT_FOUND')) {
    // Handle missing mod
  }
}
```

---

## Batch Operations

For operations that modify multiple items:

```typescript
// Set load order for multiple plugins
await invoke('set_load_order', {
  gameId: 'skyrim',
  order: [
    { gameId: 'skyrim', pluginName: 'update.esm', loadOrderIndex: 0, enabled: true, groupName: null },
    { gameId: 'skyrim', pluginName: 'dawnguard.esm', loadOrderIndex: 1, enabled: true, groupName: null },
    // ...
  ]
});

// Resolve multiple conflicts
await invoke('resolve_conflicts', {
  gameId: 'skyrim',
  resolutions: [
    { conflictId: '1', resolution: 'useModA', winnerModId: 'mod-a', mergedFilePath: null },
    { conflictId: '2', resolution: 'merge', winnerModId: null, mergedFilePath: '/path/to/merged' },
    // ...
  ]
});
```