# Module: Update Checker

## Responsibility

Автоматическая проверка обновлений для установленных модов, лоадеров и самого приложения. Уведомление пользователя о доступных обновлениях с возможностью массовой установки.

## Update Sources

| Source | Method | Frequency |
|--------|--------|-----------|
| Pantheon Repository | API (mod version check) | Every 24h or on demand |
| ModDB | RSS/HTML scraping | Every 24h |
| GitHub Releases | API (for open-source mods) | Every 24h |
| Script Extenders | Official website/version endpoint | Every 6h |
| Pantheon App | GitHub Releases / update server | On startup |

## Data Model

```rust
struct ModUpdateInfo {
    mod_id: String,
    current_version: Option<String>,
    latest_version: String,
    source: UpdateSource,
    source_id: String,              // External ID (Nexus mod ID, etc.)
    release_date: DateTime<Utc>,
    changelog: Option<String>,
    is_major_update: bool,          // Major version bump
    is_security_update: bool,       // Security-related fix
    download_url: Option<String>,
    file_size: Option<u64>,
    compatibility: UpdateCompatibility,
}

enum UpdateSource {
    NexusMods,
    ModDB,
    GitHub,
    Direct,                         // Manual URL check
    AppSelf,                        // Pantheon app update
}

enum UpdateCompatibility {
    Compatible,                     // Works with current setup
    RequiresLoaderUpdate,           // Needs newer loader
    RequiresGameVersion(String),    // Needs specific game version
    Unknown,                        // Compatibility not verified
    Incompatible,                   // Known incompatibility
}

struct UpdateCheckResult {
    checked_count: usize,
    updates_available: Vec<ModUpdateInfo>,
    errors: Vec<UpdateCheckError>,
    check_timestamp: DateTime<Utc>,
}

struct UpdateCheckError {
    mod_id: String,
    error: String,
    source: UpdateSource,
}

struct AppUpdateInfo {
    current_version: String,
    latest_version: String,
    release_date: DateTime<Utc>,
    changelog: String,
    download_url: String,
    is_critical: bool,
}
```

## Update Check Flow

```
1. Trigger (scheduled or manual)
        │
        ▼
2. Build list of installed mods with version info
        │
        └──► Filter: only mods with version + source info
        │
        ▼
3. Group mods by source (Nexus, ModDB, GitHub, etc.)
        │
        ▼
4. Batch API requests per source
        │
        ├──► Nexus: batch version check
        ├──► GitHub: release API calls
        └──► Direct: HEAD requests to update URLs
        │
        ▼
5. Compare versions (semver or custom)
        │
        ├──► current < latest → Update available
        ├──► current == latest → Up to date
        └──► current > latest → Dev/prerelease
        │
        ▼
6. Check compatibility
        │
        ├──► Game version compatibility
        ├──► Loader version requirements
        └──► Dependency chain validation
        │
        ▼
7. Aggregate results and notify user
        │
        └──► emit('updates_available')
```

## Version Comparison

```rust
// Semantic versioning comparison
fn compare_versions(current: &str, latest: &str) -> VersionDiff {
    let current = parse_version(current);
    let latest = parse_version(latest);
    
    match (current, latest) {
        (Some(c), Some(l)) => {
            if l > c {
                if l.major > c.major { VersionDiff::Major }
                else if l.minor > c.minor { VersionDiff::Minor }
                else { VersionDiff::Patch }
            } else {
                VersionDiff::UpToDate
            }
        }
        _ => VersionDiff::Unknown,
    }
}

enum VersionDiff {
    Major,
    Minor,
    Patch,
    UpToDate,
    Unknown,
}
```

## Update Notification UI

```
┌─────────────────────────────────────────────────────────────┐
│  Updates Available (3)                                       │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  ⚠️  SkyUI                            5.2 → 5.3       │  │
│  │      Minor update • Compatible • 2.1 MB               │  │
│  │      [Update] [Skip] [Details]                        │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  🔒  SKSE64                           2.2.1 → 2.2.3   │  │
│  │      Security update • Compatible • 5.4 MB            │  │
│  │      [Update] [Details]                               │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  ℹ️  Unofficial Patch                  4.2.5 → 4.3.0   │  │
│  │      Major update • Requires game 1.6+ • 120 MB       │  │
│  │      [Update] [Skip] [Details]                        │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  [Update All] [Dismiss All] [Check Again]                   │
└─────────────────────────────────────────────────────────────┘
```

## Version Pinning

```rust
struct VersionPin {
    mod_id: String,
    pinned_version: String,
    reason: PinReason,
    created_at: DateTime<Utc>,
}

enum PinReason {
    UserChoice,           // User manually pinned
    Compatibility,        // Incompatible with current setup
    Stability,            // Known issues with newer version
    Dependency,           // Required by another mod at this version
}

// Pinned mods should be excluded from update checks
// or shown with a "pinned" indicator in the UI
```

## Scheduled Checks

```rust
struct UpdateSchedule {
    enabled: bool,
    interval_hours: u32,        // Default: 24
    last_check: Option<DateTime<Utc>>,
    next_check: Option<DateTime<Utc>>,
    notify_on_startup: bool,    // Check on app startup
    auto_download: bool,        // Auto-download (not install)
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `mod-repository-api` | Primary source for mod version checks |
| `download-manager` | Downloads update files |
| `mod-installer` | Installs updates (replaces old version) |
| `database` | Stores update check results and pins |
| `settings` | User preferences for check frequency |

## API

```rust
#[tauri::command]
pub async fn check_for_updates(
    game_id: Option<String>,
) -> Result<UpdateCheckResult, String>;

#[tauri::command]
pub async fn update_mod(
    mod_id: String,
) -> Result<String, String>;  // Download ID

#[tauri::command]
pub async fn update_all_mods(
    game_id: String,
) -> Result<Vec<String>, String>;  // Download IDs

#[tauri::command]
pub async fn pin_mod_version(
    mod_id: String,
    version: String,
    reason: Option<String>,
) -> Result<(), String>;

#[tauri::command]
pub async fn unpin_mod_version(
    mod_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn skip_mod_update(
    mod_id: String,
    version: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn check_app_update() 
    -> Result<Option<AppUpdateInfo>, String>;

#[tauri::command]
pub async fn set_update_schedule(
    schedule: UpdateSchedule,
) -> Result<(), String>;
```

## Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `updates_available` | `UpdateCheckResult` | Updates found |
| `update_downloaded` | `ModUpdateInfo` | Update file ready to install |
| `update_installed` | `String` (mod_id) | Mod updated successfully |
| `update_failed` | `UpdateError` | Update installation failed |

## Best Practices

1. **Respect version pins** — Never update a pinned mod without explicit user action
2. **Batch updates** — Group updates to minimize API calls
3. **Changelog display** — Always show what changed before updating
4. **Rollback support** — Keep previous version available for quick rollback
5. **Dependency awareness** — Check if update breaks dependencies
6. **Offline mode** — Cache update info for offline viewing
7. **Silent checks** — Background checks should not block UI

## Notes

- Not all mods have version numbers — use file upload date as fallback
- Repository API provides file version info for mods uploaded through the platform
- Manual mods (from other sources) cannot be auto-checked without explicit update URL
- Major updates may require re-running FOMOD installation options
- Consider implementing a "changelog diff" for mods with detailed release notes
