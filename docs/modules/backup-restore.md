# Module: Backup & Restore

## Responsibility

Создание, управление и восстановление бэкапов: файлов игры, сохранений, конфигураций и состояния мод-менеджера. Обеспечивает безопасный откат при проблемах после установки модов или обновлений игры.

## Backup Types

| Type | What | When | Size |
|------|------|------|------|
| **Game Files** | Original game files before modding | Before first mod install | 100MB - 5GB |
| **Save Games** | Player save files | Before major mod changes | 1MB - 500MB |
| **Configuration** | INI files, config files | Before mod install | < 10MB |
| **Profile State** | Full profile snapshot | On profile switch | 10MB - 1GB |
| **Mod Staging** | Staged mod files | Before bulk operations | 1GB - 100GB |
| **Full Snapshot** | Everything above | Manual, before major changes | Variable |

## Data Model

```rust
struct Backup {
    id: String,                     // UUID
    game_id: String,
    profile_id: Option<String>,
    name: String,                   // User-friendly name
    description: Option<String>,
    backup_type: BackupType,
    created_at: DateTime<Utc>,
    size_bytes: u64,
    file_count: usize,
    status: BackupStatus,
    location: PathBuf,              // Where backup is stored
    metadata: BackupMetadata,
}

enum BackupType {
    GameFiles,
    SaveGames,
    Configuration,
    ProfileState,
    ModStaging,
    FullSnapshot,
}

enum BackupStatus {
    Creating,
    Completed,
    Failed,
    Restoring,
}

struct BackupMetadata {
    game_version: Option<String>,
    mod_count: Option<usize>,
    profile_name: Option<String>,
    tags: Vec<String>,              // User-defined tags
    auto_created: bool,             // Created automatically vs manually
    checksum: Option<String>,       // SHA256 of backup archive
}

struct BackupSettings {
    auto_backup_enabled: bool,
    auto_backup_triggers: Vec<AutoBackupTrigger>,
    max_backups_per_type: HashMap<BackupType, u32>,
    max_total_size_gb: u64,
    backup_location: Option<PathBuf>, // Custom backup directory
    compression: CompressionLevel,
}

enum AutoBackupTrigger {
    BeforeModInstall,
    BeforeModUninstall,
    BeforeProfileSwitch,
    BeforeGameUpdate,
    BeforeBulkChanges,
    OnAppExit,
}

enum CompressionLevel {
    None,       // Fastest, largest
    Fast,       // Good balance
    Maximum,    // Slowest, smallest
}
```

## Backup Flow

```
1. User triggers backup (or auto-trigger fires)
        │
        ▼
2. Determine backup scope
        │
        ├──► Game files: Copy original files
        ├──► Saves: Copy save directory
        ├──► Config: Copy INI/config files
        └──► Full: All of the above + profile state
        │
        ▼
3. Create backup archive
        │
        ├──► Create timestamped directory
        ├──► Copy files with progress tracking
        ├──► Generate checksum (SHA256)
        ├──► Compress if enabled
        └──► Write metadata.json
        │
        ▼
4. Register backup in database
        │
        └──► INSERT INTO backups ...
        │
        ▼
5. Cleanup old backups (if exceeding limits)
        │
        └──► Delete oldest backups beyond limit
```

## Restore Flow

```
1. User selects backup to restore
        │
        ▼
2. Verify backup integrity
        │
        ├──► Check checksum matches
        ├──► Verify archive is not corrupted
        └──► Check sufficient disk space
        │
        ▼
3. Show restore preview
        │
        └──► List files that will be restored
        │
        ▼
4. Create pre-restore backup (safety net)
        │
        └──► Snapshot current state
        │
        ▼
5. Restore files
        │
        ├──► Extract backup archive
        ├──► Overwrite target files
        └──► Update deployment state
        │
        ▼
6. Verify restored state
        │
        └──► Check file integrity
        │
        ▼
7. Update UI and notify user
```

## Backup Storage Structure

```
backups/
├── {game_id}/
│   ├── 2026-04-01_14-30-00_game_files/
│   │   ├── metadata.json
│   │   ├── checksum.sha256
│   │   └── files/
│   │       └── ... (backed up game files)
│   │
│   ├── 2026-04-01_14-30-00_saves/
│   │   ├── metadata.json
│   │   └── saves/
│   │       └── ... (save files)
│   │
│   └── 2026-04-02_10-00-00_full_snapshot/
│       ├── metadata.json
│       ├── checksum.sha256
│       ├── game_files/
│       ├── saves/
│       ├── configs/
│       └── profile_state.json
```

## Game File Backup Strategy

```rust
// Track which original files were backed up
struct GameFileBackup {
    original_path: PathBuf,
    backup_path: PathBuf,
    file_hash: String,
    file_size: u64,
    backed_up_at: DateTime<Utc>,
}

// Before deploying mods:
// 1. Check if game files are already backed up
// 2. If not, create backup of files that will be affected
// 3. Track backed-up files for restore
// 4. Only backup once per file (idempotent)
```

## Save Game Protection

```rust
struct SaveGameBackup {
    save_path: PathBuf,
    backup_path: PathBuf,
    save_name: String,
    save_date: DateTime<Utc>,
    active_mods: Vec<String>,       // Mods active when backed up
    game_version: String,
}

// Automatic save backup:
// - Before installing/uninstalling mods
// - Before switching profiles
// - Before game updates
// - Periodic (configurable interval)
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `deploy-manager` | Triggers backup before deployment changes |
| `profile-manager` | Backs up profile state on switch |
| `mod-installer` | Triggers backup before bulk install/uninstall |
| `game-launcher` | Can backup saves before launch |
| `database` | Stores backup metadata and registry |

## API

```rust
#[tauri::command]
pub async fn create_backup(
    game_id: String,
    backup_type: BackupType,
    name: Option<String>,
) -> Result<Backup, String>;

#[tauri::command]
pub async fn restore_backup(
    backup_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn list_backups(
    game_id: String,
    backup_type: Option<BackupType>,
) -> Result<Vec<Backup>, String>;

#[tauri::command]
pub async fn delete_backup(
    backup_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn cleanup_old_backups(
    game_id: String,
    settings: BackupSettings,
) -> Result<usize, String>;  // Number deleted

#[tauri::command]
pub async fn get_backup_settings() 
    -> Result<BackupSettings, String>;

#[tauri::command]
pub async fn set_backup_settings(
    settings: BackupSettings,
) -> Result<(), String>;

#[tauri::command]
pub async fn export_backup(
    backup_id: String,
    dest: PathBuf,
) -> Result<PathBuf, String>;

#[tauri::command]
pub async fn import_backup(
    game_id: String,
    source: PathBuf,
) -> Result<Backup, String>;
```

## Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `backup_created` | `Backup` | Backup completed |
| `backup_restored` | `String` (backup_id) | Restore completed |
| `backup_failed` | `BackupError` | Backup/restore failed |
| `auto_backup_triggered` | `Backup` | Automatic backup created |

## Best Practices

1. **Always backup before destructive operations** — Install, uninstall, profile switch
2. **Verify integrity** — Check checksums before and after restore
3. **Safety net backup** — Create a backup before restoring (double safety)
4. **Incremental backups** — Only backup changed files after initial full backup
5. **Size management** — Enforce limits to prevent disk space exhaustion
6. **User control** — Allow custom backup locations and naming
7. **Export/import** — Enable sharing backup files between machines
8. **Progress tracking** — Show progress for large backup operations

## Notes

- Game file backups should be done ONCE — only backup originals, not modded files
- Save game backups should be frequent and automatic
- Full snapshots are expensive — use sparingly, mainly for major changes
- Consider cloud backup integration (future enhancement)
- Backup compression trade-off: speed vs disk space
