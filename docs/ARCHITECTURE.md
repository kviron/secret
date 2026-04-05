# Architecture Overview

## System Design

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Pantheon Architecture                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────┐          ┌─────────────────────────────┐   │
│  │     Rust Backend        │          │    Solid.js Frontend        │   │
│  │     (Tauri 2.x)         │          │                             │   │
│  │                         │  IPC     │  ┌───────────────────────┐ │   │
│  │  ┌─────────────────┐   │◄────────►│  │   Solid.js Stores     │ │   │
│  │  │  Application    │   │  invoke()│  │   (Reactive State)    │ │   │
│  │  │  Lifecycle      │   │  events  │  └───────────────────────┘ │   │
│  │  └─────────────────┘   │          │  ┌───────────────────────┐ │   │
│  │  ┌─────────────────┐   │          │  │   Views               │ │   │
│  │  │  Command        │   │          │  │   (Solid Components)   │   │
│  │  │  Handlers       │   │          │  │   (FSD: pages/widgets) │   │
│  │  └────────┬────────┘   │          │  └───────────────────────┘ │   │
│  │           │             │          │  ┌───────────────────────┐ │   │
│  │  ┌────────┴────────┐   │          │  │   Features            │ │   │
│  │  │  Game Detector  │   │          │  │   (FSD: features/)     │   │
│  │  │  Mod Installer  │   │          │  └───────────────────────┘ │   │
│  │  │  Deploy Mgr    │   │          │  ┌───────────────────────┐ │   │
│  │  │  Download Mgr  │   │          │  │   Entities            │ │   │
│  │  │  Load Order Mgr│   │          │  │   (FSD: entities/)     │   │
│  │  │  Extension Mgr │   │          │  └───────────────────────┘ │   │
│  │  └────────┬────────┘   │          │                            │   │
│  │           │             │          │  ┌───────────────────────┐ │   │
│  │  ┌────────┴────────┐   │          │  │   Shared              │ │   │
│  │  │  SQLite (r2d2)  │   │          │  │   (FSD: shared/)       │   │
│  │  │  (Persistence)  │   │          │  │   UI kit, API, utils   │   │
│  │  └─────────────────┘   │          │  └───────────────────────┘ │   │
│  │  ┌─────────────────┐   │          │                            │   │
│  │  │  Extensions     │   │          │                            │   │
│  │  │  (Rust traits)  │   │          │                            │   │
│  │  └─────────────────┘   │          │                            │   │
│  └─────────────────────────┘          └─────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Shared Types (TypeScript + Rust)               │   │
│  │   Tauri Commands • Data Models • IPC Events • Error Types         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## UI: Games Library (dashboard)

The **Games Library** route (`src/pages/dashboard/index.tsx`) lists detected games from the SQLite-backed store. Game cards show:

- **Cover image**: For Steam titles, the default art URL is built from `details.steamAppId` using Steam’s standard **header** asset (`header.jpg`, aspect ratio **460:215**). Optional `details.logo` overrides with any `https` URL. Implementation: `src/shared/lib/steam-art.ts`, styles in `src/index.css` (`.game-card-header`, `.game-card-art`).
- **State**: `entities/game` store (`loadGames`, detection events from `features/detect-games`).

Serialization for `invoke('get_games')` and related commands must expose camelCase JSON so the UI can read `supportedModTypes`, `modSupport`, etc. — see MODELS.md.

---

## Module Relationships

```
                    ┌──────────────────────────────────────────┐
                    │              Application                  │
                    │         (Tauri Entry / Lifecycle)         │
                    └────────────────────┬─────────────────────┘
                                         │
              ┌──────────────────────────┼──────────────────────────┐
              │                          │                          │
              ▼                          ▼                          ▼
    ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
    │    Extension     │      │   SQLite        │      │   Settings      │
    │    System        │      │   (r2d2 pool)   │      │   Manager       │
    └────────┬────────┘      └────────┬────────┘      └────────┬────────┘
             │                        │                        │
             │                        │                        │
             ▼                        ▼                        ▼
    ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
    │   Game Detector  │      │   Mod Installer │      │   Deploy        │
    │                  │      │                 │      │   Manager       │
    └────────┬────────┘      └────────┬────────┘      └────────┬────────┘
             │                        │                        │
             │                        │                        │
             ▼                        ▼                        ▼
    ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
    │   Load Order     │      │   Download      │      │   UI Layer      │
    │   Manager        │      │   Manager       │      │   (Solid.js)    │
    └─────────────────┘      └─────────────────┘      └─────────────────┘
```

---

## Data Flow Diagrams

### State Persistence Flow

```
User Action
     │
     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Solid.js Frontend                              │
│                                                                    │
│  ┌──────────┐    Signal/Store    ┌──────────────────────────┐   │
│  │ Component│──────────────────►│   Solid.js Store         │   │
│  └──────────┘                   │  (Reactive State)         │   │
│                                 └──────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                                 │
                          invoke() IPC
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Rust Backend                              │
│  ┌──────────────┐    ┌──────────────────┐    ┌──────────────┐  │
│  │ Tauri        │───►│  SQLite Write    │───►│  Prepared    │  │
│  │ Command      │    │  (r2d2 pool)     │    │  Statements  │  │
│  └──────────────┘    └──────────────────┘    └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                 │
                          emit() event
                          (state update)
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Solid.js Frontend                              │
│  ┌──────────────────────────┐    ┌──────────────────────────┐  │
│  │   listen() event          │───►│    Store Update           │  │
│  │   handler                 │    │    (Reactive)            │  │
│  └──────────────────────────┘    └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Download Flow

```
download(resource, dest)
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Download Manager (Rust)                     │
│                                                              │
│  ┌──────────────┐                                           │
│  │   Resolver   │──► Resolves resource to URL               │
│  └──────┬───────┘                                           │
│         ▼                                                    │
│  ┌──────────────┐                                           │
│  │  Prober      │──► HEAD request to check Range support     │
│  └──────┬───────┘                                           │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐    │
│  │              Download Strategy                        │    │
│  │                                                       │    │
│  │  ┌─────────────────┐    ┌─────────────────────────┐  │    │
│  │  │  Chunked        │    │  Single Stream          │  │    │
│  │  │  (Range GET)    │    │  (No Range support)     │  │    │
│  │  └────────┬────────┘    └───────────┬─────────────┘  │    │
│  │           │                         │                 │    │
│  │           ▼                         ▼                 │    │
│  │  ┌─────────────────┐    ┌─────────────────────────┐  │    │
│  │  │ Tokio Tasks     │    │ reqwest stream          │  │    │
│  │  │ (Parallel chunks│    │ (tokio::io::copy)       │  │    │
│  │  └────────┬────────┘    └─────────────────────────┘  │    │
│  └───────────┼──────────────────────────────────────────┘    │
│              │                                               │
│              ▼                                               │
│  ┌──────────────────┐                                        │
│  │ Progress events  │──► emit() ──► UI Updates                │
│  └──────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘
               │
               ▼
     ┌─────────────────┐
     │   Downloaded    │
     │   File          │
     └─────────────────┘
```

### Extension Registration Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Extension Load Sequence                       │
│                                                                  │
│  1. Scan extensions/ directory                                   │
│         │                                                        │
│         ▼                                                        │
│  2. Load extension manifest (game.json)                          │
│         │                                                        │
│         ▼                                                        │
│  3. Check dependencies (runtime.requires)                        │
│         │                                                        │
│         ▼                                                        │
│  4. Initialize extension (call init())                           │
│         │                                                        │
│         ▼                                                        │
│  5. Extension registers via context                              │
│         │                                                        │
│         ├──► register_game()                                     │
│         ├──► register_mod_type()                                 │
│         └──► register_installer()                                │
│         │                                                        │
│         ▼                                                        │
│  6. ExtensionManager stores registrations                        │
│         │                                                        │
│         ▼                                                        │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              Post-Registration                              │  │
│  │                                                               │  │
│  │  ┌────────────────┐    ┌────────────────┐                   │  │
│  │  │ Games Listed   │    │ Installers     │                   │  │
│  │  │ in Registry    │    │ Ready          │                   │  │
│  │  └────────────────┘    └────────────────┘                   │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## State Structure

```typescript
// Solid.js Store structure (FSD: entities/)

interface AppState {
  // Persistent (SQLite-backed)
  games: Game[];
  mods: Record<string, Mod[]>;  // keyed by game_id
  deployment: Record<string, DeploymentState>;  // keyed by mod_id
  loadOrder: Record<string, LoadOrderEntry[]>;  // keyed by game_id
  downloads: Download[];
  settings: AppSettings;
  profiles: Record<string, Profile[]>;  // keyed by game_id
  backups: Record<string, Backup[]>;    // keyed by game_id
  
  // Session (ephemeral, in-memory)
  selectedGameId: string | null;
  activeProfileId: string | null;
  ui: {
    isLoading: boolean;
    notifications: Notification[];
    dialog: DialogState | null;
  };
  
  // Security
  validationResults: Record<string, ValidationResult>;  // keyed by mod_id
  
  // Updates
  availableUpdates: ModUpdateInfo[];
  
  // Repository
  repoAuth: RepositoryAuth | null;
}

interface AppSettings {
  theme: 'light' | 'dark';
  deploymentStrategy: 'symlink' | 'hardlink' | 'copy';
  downloadConcurrency: number;
  stagingPath: string;
}
```

---

## Tauri Commands

| Command | Direction | Purpose |
|---------|------------|---------|
| `get_games` | UI → Rust | Get all games |
| `detect_games` | UI → Rust | Scan for games |
| `install_mod` | UI → Rust | Install a mod |
| `uninstall_mod` | UI → Rust | Remove a mod |
| `enable_mod` | UI → Rust | Enable a mod |
| `disable_mod` | UI → Rust | Disable a mod |
| `deploy_mod` | UI → Rust | Deploy mod to game |
| `undeploy_mod` | UI → Rust | Remove mod from game |
| `get_load_order` | UI → Rust | Get plugin load order |
| `set_load_order` | UI → Rust | Set plugin load order |
| `start_download` | UI → Rust | Start a download |
| `pause_download` | UI → Rust | Pause a download |
| `resume_download` | UI → Rust | Resume a download |
| `get_download_progress` | UI → Rust | Get download progress |
| `launch_game` | UI → Rust | Launch game with mod loader |
| `validate_mod` | UI → Rust | Validate mod archive security |
| `check_for_updates` | UI → Rust | Check mod updates |
| `create_profile` | UI → Rust | Create mod profile |
| `switch_profile` | UI → Rust | Switch active profile |
| `create_backup` | UI → Rust | Create game/save backup |
| `restore_backup` | UI → Rust | Restore from backup |
| `resolve_dependencies` | UI → Rust | Resolve mod dependencies |
| `repo_search_mods` | UI → Rust | Search mod repository |
| `repo_download_mod` | UI → Rust | Download from repository |

## Tauri Events

| Event | Direction | Purpose |
|-------|------------|---------|
| `download_progress` | Rust → UI | Download progress updates |
| `download_completed` | Rust → UI | Download finished |
| `download_failed` | Rust → UI | Download error |
| `mod_installed` | Rust → UI | Mod installation complete |
| `deploy_completed` | Rust → UI | Deployment finished |
| `conflict_detected` | Rust → UI | File conflict found |
| `game_launched` | Rust → UI | Game process started |
| `game_exited` | Rust → UI | Game process ended |
| `game_crashed` | Rust → UI | Game crashed |
| `validation_complete` | Rust → UI | Mod validation finished |
| `malware_detected` | Rust → UI | Potential malware found |
| `updates_available` | Rust → UI | Mod updates found |
| `profile_switched` | Rust → UI | Profile changed |
| `backup_created` | Rust → UI | Backup completed |
| `backup_restored` | Rust → UI | Restore completed |
| `repo_auth_complete` | Rust → UI | Repository OAuth completed |

---

## Key Module Responsibilities

### Rust Backend Modules

| Module | Responsibility | Key APIs |
|--------|----------------|----------|
| `Application` | Lifecycle, window, config | `run()`, `quit()` |
| `DownloadManager` | Queue management, pause/resume | `download()`, `pause()`, `cancel()` |
| `Database` | SQLite persistence (r2d2 pool) | `find_game()`, `insert_mod()` |
| `ExtensionManager` | Extension loading/registration | `load_extensions()`, `register_game()` |
| `GameDetector` | Game discovery (Steam, GOG, Epic) | `detect_games()` |
| `ModInstaller` | Archive extraction, staging | `install()`, `uninstall()` |
| `DeployManager` | Symlink/hardlink deployment | `deploy()`, `undeploy()` |
| `LoadOrderManager` | Plugin ordering, LOOT | `get_load_order()`, `auto_sort()` |
| `ProfileManager` | Profile switching, export/import | `create_profile()`, `switch_profile()` |
| `GameLauncher` | Game launching with loaders | `launch_game()`, `detect_loaders()` |
| `SecurityValidator` | Mod validation, malware scanning | `validate_mod()`, `scan_files()` |
| `UpdateChecker` | Mod update detection | `check_for_updates()`, `pin_version()` |
| `BackupManager` | Game/save backup & restore | `create_backup()`, `restore_backup()` |
| `DependencyResolver` | Dependency graph, conflict resolution | `resolve_dependencies()`, `build_graph()` |
| `RepositoryApiClient` | Pantheon repository integration | `search_mods()`, `download_mod()` |
| `RepositoryApiClient` | Pantheon repository integration | `search_mods()`, `download_mod()` |

### Solid.js Frontend Modules (FSD)

| Layer | Responsibility | Key APIs |
|-------|----------------|----------|
| `app/` | App init, providers, routing | `AppRouter`, `ThemeProvider` |
| `pages/` | Route-level composition | Page components |
| `widgets/` | Reusable composites | `GameCard`, `ModList` |
| `features/` | User interactions | `installMod`, `toggleMod` |
| `entities/` | Business models + stores | `gameStore`, `modStore` |
| `shared/` | UI kit, API client, utils | `Button`, `invoke()`, `formatDate` |

### Extension Categories

| Category | Examples | Purpose |
|----------|----------|---------|
| Game Extensions | `game-skyrim`, `game-fallout4` | Game-specific logic |
| Mod Type Extensions | `modtype-bepinex`, `modtype-fomod` | Mod type handling |
| Installer Extensions | `installer-fomod` | Custom installers |
| Built-in | `game-generic`, `modtype-simple` | Core fallback support |

---

## Extension API (Rust Traits)

Extensions register functionality through Rust traits:

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&self, ctx: &mut ExtensionContext) -> Result<(), String>;
}

pub trait GameExtension: Extension {
    fn detect(&self) -> Option<GameInfo>;
    fn get_mod_paths(&self, install_path: &Path) -> HashMap<String, PathBuf>;
    fn list_plugins(&self, game_path: &Path) -> Result<Vec<PluginInfo>, String>;
}

pub trait ModTypeExtension: Extension {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
    fn test(&self, archive: &Path) -> bool;
    fn install(&self, archive: &Path, dest: &Path) -> Result<Mod, String>;
}
```

---

## Comparison: Pantheon → Pantheon

| Pantheon Module | Pantheon Equivalent | Notes |
|---------------|-------------------|-------|
| Electron Main | Tauri 2.x | Rust backend, smaller bundle |
| Electron Renderer | Solid.js | Fine-grained reactivity, no VDOM |
| Redux Store | Solid.js Stores | Native reactive signals |
| LevelDB + DuckDB | SQLite (rusqlite + r2d2) | Single database, simpler |
| React Components | Solid Components (FSD) | Feature-Sliced Design |
| LESS/CSS | Panda CSS | Compile-time CSS-in-JS |
| JS Extensions | Rust traits + JSON manifests | Type-safe, compiled |
| gamebryo-plugin-management | loadOrderManager | LOOT integration |
| mod_management | modInstaller | FOMOD support |
| download_management | downloadManager | Rust async (reqwest + tokio) |
| Telemetry | Optional | Skip for privacy |

---

## Design Reference

![Pantheon UI Design Reference](https://i.pinimg.com/736x/49/ce/7a/49ce7a22004b8f3f391c3f6e4a06b568.jpg)

**Description:** Dark-themed mod manager UI with sidebar navigation, game cards grid, mod list with toggle switches.

**Key Design Elements:**
- Sidebar: general links (games library, deployments, settings); when a **managed game** is active, a game banner (art + launch) and a second group for that game’s mods, plugins, and saves — see [modules/managed-game-context.md](./modules/managed-game-context.md)
- Card-based game grid with cover art and metadata
- Mod list with status indicators and toggle switches
- Dark color scheme with accent colors for interactive elements
- Clean typography with clear visual hierarchy
- Subtle gradients and smooth transitions for interactive states
