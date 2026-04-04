# Module: Profile Manager

## Responsibility

Управление профилями модов — возможность создавать, переключать и экспортировать различные конфигурации модов для одной и той же игры. Каждый профиль содержит собственный набор включённых модов, порядок загрузки, настройки и локальные сохранения.

## Зачем нужны профили

| Сценарий | Описание |
|----------|----------|
| Разные стили игры | «Vanilla+» vs «Full Overhaul» для одной игры |
| Тестирование | Изолированный профиль для проверки новых модов |
| Совместные сохранения | Разные профили для разных персонажей/сейвов |
| Стриминг | Отдельный стабильный профиль для стримов |
| Разработка | Профиль с инструментами моддинга |

## Data Model

```rust
struct Profile {
    id: String,                     // UUID
    game_id: String,                // Parent game
    name: String,                   // Display name
    description: Option<String>,    // Optional description
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_default: bool,               // Default profile for game
    settings: ProfileSettings,      // Profile-specific settings
}

struct ProfileSettings {
    load_order: Vec<String>,        // Plugin load order (plugin names)
    enabled_mods: HashSet<String>,  // Set of enabled mod IDs
    disabled_mods: HashSet<String>, // Explicitly disabled mod IDs
    deployment_strategy: DeployStrategy,
    game_launch_args: Vec<String>,  // Custom launch arguments
    local_save_path: Option<PathBuf>, // Per-profile save directory
}

struct ProfileMod {
    profile_id: String,
    mod_id: String,
    enabled: bool,
    load_order: u32,
    pinned_version: Option<String>, // Version pinning
    notes: Option<String>,          // User notes about this mod
}
```

## Profile Storage Structure

```
profiles/
├── {game_id}/
│   ├── profiles.json              # Profile metadata
│   ├── {profile_id}/
│   │   ├── settings.json          # Profile settings
│   │   ├── load_order.txt         # Plugin load order
│   │   ├── modlist.txt            # Enabled/disabled mods
│   │   └── saves/                 # Optional per-profile saves
│   │       └── ...
│   └── {profile_id_2}/
│       └── ...
```

## Profile Operations

```rust
#[tauri::command]
pub async fn create_profile(
    game_id: String,
    name: String,
    copy_from: Option<String>,  // Clone from existing profile
) -> Result<Profile, String>;

#[tauri::command]
pub async fn delete_profile(
    profile_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn switch_profile(
    profile_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn export_profile(
    profile_id: String,
    dest: PathBuf,
    include_modlist: bool,
) -> Result<PathBuf, String>;

#[tauri::command]
pub async fn import_profile(
    game_id: String,
    source: PathBuf,
    name: String,
) -> Result<Profile, String>;

#[tauri::command]
pub async fn get_profiles(
    game_id: String,
) -> Result<Vec<Profile>, String>;

#[tauri::command]
pub async fn set_profile_default(
    profile_id: String,
) -> Result<(), String>;
```

## Profile Switching Flow

```
1. User selects target profile
        │
        ▼
2. Save current profile state
        │
        ├──► Flush pending changes
        ├──► Update deployment state
        └──► Save load order
        │
        ▼
3. Load target profile
        │
        ├──► Read profile settings
        ├──► Load enabled/disabled mods
        └──► Load load order
        │
        ▼
4. Redeploy mods
        │
        ├──► Undeploy current mods
        ├──► Deploy profile mods
        └──► Apply load order
        │
        ▼
5. Update UI
        │
        └──► emit('profile_switched')
```

## Profile Export Format

```json
{
    "format_version": "1.0.0",
    "profile": {
        "name": "Heavy Overhaul",
        "game_id": "skyrimse",
        "description": "300+ mods full overhaul",
        "settings": {
            "deployment_strategy": "symlink",
            "game_launch_args": ["-SKSE:LoadPlugins"]
        }
    },
    "modlist": [
        {
            "mod_id": "uuid-1",
            "name": "SkyUI",
            "version": "5.2.3",
            "enabled": true,
            "load_order": 1,
            "source": "nexus",
            "source_id": "12345"
        }
    ],
    "load_order": [
        "Skyrim.esm",
        "Update.esm",
        "Dawnguard.esm"
    ]
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `database` | Stores profile metadata and settings |
| `deploy-manager` | Redeploys mods on profile switch |
| `load-order-manager` | Per-profile load order storage |
| `game-launcher` | Uses profile launch args |
| `mod-installer` | Installs mods into active profile context |

## Best Practices

1. **Atomic switches** — Profile switch should be all-or-nothing, with rollback on failure
2. **Staging isolation** — Each profile references the same staging area but tracks enabled/disabled independently
3. **Save game awareness** — Warn users when switching profiles with active save games
4. **Profile snapshots** — Allow creating point-in-time snapshots for rollback
5. **Shared mods** — Mod files are shared across profiles; only enabled state differs
6. **Version pinning** — Allow pinning specific mod versions per profile

## Notes

- Profiles do NOT duplicate mod files — only track enabled/disabled state and load order
- Profile switching triggers undeploy → deploy cycle (may take time for large modlists)
- Per-profile save directories prevent save corruption when switching heavily modded profiles
- Export/import enables sharing mod configurations between users (collections lite)
