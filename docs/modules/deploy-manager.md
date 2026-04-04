# Module: Deploy Manager

## Responsibility

Manages deployment of mods from staging area to game folder. Implements virtual filesystem (VFS) for safe file management with symlinks/hardlinks/copy strategies.

## Pantheon Comparison

Pantheon `mod_management` extension handles deployment with:
- **VFS (Virtual File System)**: Staging area with deployed symlinks
- **Deployment Methods**: Symlink (Windows default), Hardlink, Copy, Merge
- **File Conflict Detection**: Prevents conflicts before deployment
- **Archive Invalidation**: BSA timestamp management for Bethesda games

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Deployment Architecture                          │
└─────────────────────────────────────────────────────────────────────────┘

    Mod Installation                    Deployment
    ┌─────────────────┐                ┌─────────────────┐
    │   Staging/      │                │   Game Data/    │
    │   mods/         │   deploy()     │   Data/         │
    │   ├── mod_a/    │ ─────────────► │   ├── mod_a/    │
    │   │   └── files │                │   └── mod_b/    │
    │   └── mod_b/    │                └─────────────────┘
    └─────────────────┘
              │
              │ undeploy()
              ▼
         (removes symlinks)
```

## Deployment Strategies

| Strategy | Description | Use Case | Pantheon Equivalent |
|----------|-------------|----------|------------------|
| Symlink | Symlinks in game folder | **Default on Windows** | `symlink` |
| Hardlink | Hard links to staging | When symlinks unavailable | `hardlink` |
| Copy | Copy files directly | Unsupported FS | `copy` |
| Merge | VFS via folder merge | File conflicts | `merge` (manual) |

## Staging Structure

```
staging/
├── mods/
│   ├── {mod_id}/
│   │   ├── mod.json          # Mod metadata
│   │   ├── files/            # All mod files
│   │   │   ├── Data/
│   │   │   │   ├── textures/
│   │   │   │   └── meshes/
│   │   │   └── plugins/
│   │   │       └── mods.txt  # List of contained mods
│   │   └── staging_info.json # Installation details
│   └── {mod_id_2}/
└── deployed/                 # (Internal) deployed files tracking
    └── {game_id}/
        ├── active/          # Currently deployed mod files
        └── staging_map.json  # Maps source -> target
```

## Deployment Flow

```
1. User enables mod
       │
       ▼
2. Check for file conflicts
       │
       ├──► Conflicts found ──► Show conflict resolution UI
       │                              │
       │                              ▼
       │                     User resolves (merge/overwrite/skip)
       │                              │
       ▼                              │
3. Create symlinks in game folder ◄────────────────┘
       │
       ├──► Hardlink if symlink fails (Windows fallback)
       │
       ▼
4. Update deployment state in database
       │
       ▼
5. Emit event for UI update
       │
       ▼
6. (Optional) BSA archive invalidation (Bethesda games)
```

## Data Model

```rust
struct DeploymentState {
    mod_id: String,
    game_id: String,
    status: DeployStatus,
    strategy: DeployStrategy,
    deployed_files: Vec<DeployedFile>,
    conflicts: Vec<Conflict>,
    deployed_at: DateTime<Utc>,
}

struct DeployedFile {
    source: PathBuf,        // In staging (relative to mod folder)
    target: PathBuf,        // In game folder (relative to game path)
    link_type: LinkType,
    size: u64,
    hash: String,
}

enum DeployStatus {
    Pending,               // Mod enabled, not yet deployed
    Deployed,              // Successfully deployed
    PartiallyDeployed,     // Some files deployed (conflicts resolved partially)
    Failed,                // Deployment failed
    Conflict,              // Has unresolved conflicts
}

enum DeployStrategy {
    Symlink,
    Hardlink,
    Copy,
    Merge,
}

enum LinkType {
    Symlink,
    Hardlink,
    Copy,
    DirectoryJunction,    // For directories on Windows
}

enum Conflict {
    FileConflict {
        mod_a: String,
        mod_b: String,
        file: PathBuf,
        size_a: u64,
        size_b: u64,
    },
    MissingDependency {
        mod_id: String,
        dependency_id: String,
    },
    PluginConflict {
        plugin_a: String,
        plugin_b: String,
        load_order_a: u32,
        load_order_b: u32,
    },
}
```

## Conflict Resolution

### Pantheon Conflict Handling

Pantheon provides several conflict resolution strategies:

1. **File Priority**: Higher-priority mod wins
2. **Manual Merge**: User chooses which file to keep
3. **Load Order**: Later load order wins
4. **Ignore**: Mark as "intentional override"

### Pantheon Resolution Flow

```
Conflict Detected
       │
       ▼
┌─────────────────────────┐
│  Conflict Dialog UI      │
│                          │
│  Mod A vs Mod B          │
│  File: textures/x.nif   │
│                          │
│  [Use A] [Use B] [Merge] │
└───────────┬─────────────┘
            │
            ▼
    Resolution Applied
            │
            ▼
    ┌───────────────────┐
    │ Update deploy state │
    │ Set priority/merge  │
    └───────────────────┘
```

## BSA Archive Invalidation (Bethesda Games)

Bethesda games use BSA/BA2 archives. Pantheon handles this with `gamebryo-archive-invalidation`:

```rust
// When mods are deployed/undeployed, update ArchiveInvalidation:
// 1. Read current ArchiveInvalidationfiles in Skyrim.ini / SkyrimPrefs.ini
// 2. Add/remove mod BSA filenames
// 3. Set "ArchiveInvalidation=true" if any BSAs present
// 4. Optionally backdate BSAs to ensure game uses deployed versions

struct ArchiveInvalidation {
    enabled: bool,
    bsa_files: Vec<String>,  // BSA filenames to invalidate
    last_updated: DateTime<Utc>,
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `mod-installer` | Gets mods from staging for deployment |
| `database` | Stores deployment state and conflicts |
| `game-detector` | Gets game support_path for deployment target |
| `load-order-manager` | Plugin conflicts may affect load order |
| `settings` | User preferences for deployment strategy |

## API

```rust
#[tauri::command]
pub async fn deploy_mod(mod_id: String) -> Result<DeploymentState, String>;

#[tauri::command]
pub async fn undeploy_mod(mod_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn enable_mod(mod_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn disable_mod(mod_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn get_deployment_state(game_id: String) -> Result<Vec<DeploymentState>, String>;

#[tauri::command]
pub async fn resolve_conflicts(
    game_id: String,
    resolutions: HashMap<String, ConflictResolution>,
) -> Result<(), String>;

#[tauri::command]
pub async fn set_deployment_strategy(
    game_id: String,
    strategy: DeployStrategy,
) -> Result<(), String>;
```

## Notes

- Symlinks require admin privileges on Windows (Pantheon uses elevation or creates in user-writable location)
- Hardlinks only work within same filesystem
- Merge strategy creates "merge" folder with all conflicting files
- Bethesda games need special handling for BSAs (Archive Invalidation)
- Some games (Starfield) use different archive formats
