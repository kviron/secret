# Module Connections

## Purpose

This document describes how Pantheon modules interact with each other, showing data flow, dependencies, and communication patterns using our stack (Tauri 2.x + Solid.js + Rust + SQLite + Panda CSS).

---

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Module Dependencies                               │
└─────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────┐
                    │        Application       │
                    │     (Tauri Entry)        │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
              ▼                  ▼                  ▼
    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │   Extension      │  │   SQLite        │  │   Settings      │
    │   System         │  │   (r2d2 pool)   │  │   Manager       │
    │   (Rust traits)  │  │   (rusqlite)    │  │   (JSON)        │
    └────────┬────────┘  └────────┬────────┘  └────────┬────────┘
             │                    │                    │
             │                    │                    │
             ▼                    ▼                    ▼
    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │   Game Detector  │  │   Mod Installer │  │   Deploy        │
    │   (winreg, etc)  │  │   (zip, 7z)     │  │   Manager       │
    │                  │  │                 │  │   (symlinks)    │
    └────────┬────────┘  └────────┬────────┘  └────────┬────────┘
             │                    │                    │
             │                    │                    │
             ▼                    ▼                    ▼
    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │   Load Order     │  │   Download      │  │   UI Layer      │
    │   Manager        │  │   Manager       │  │   (Solid.js)    │
    │   (LOOT)         │  │   (reqwest)     │  │   (Panda CSS)   │
    └─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## Interaction Patterns

### 1. User Installs a Mod

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Mod Installation Flow                                  │
└─────────────────────────────────────────────────────────────────────────┘

User selects mod archive
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 1. UI Layer (Solid.js + Panda CSS)                                  │
│    └──► User drops mod archive or clicks "Install"                  │
│    └──► Shows installation progress (createSignal)                  │
│    └──► invoke('install_mod', { game_id, archive_path })            │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Mod Installer (Rust)                                             │
│    ├──► Detects archive type (zip, 7z, rar, FOMOD)                  │
│    ├──► Extracts to staging directory (zip crate / sevenz-rust)     │
│    ├──► Validates files                                             │
│    └──► Returns ModInfo                                             │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. SQLite (rusqlite + r2d2)                                         │
│    ├──► INSERT INTO mods ...                                        │
│    ├──► INSERT INTO mod_files ...                                   │
│    └──► Returns mod ID                                              │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 4. Deploy Manager (Rust)                                            │
│    ├──► Checks for conflicts with existing mods                     │
│    ├──► Creates symlinks in game folder (std::os::windows::fs)      │
│    └──► INSERT/UPDATE deployment table                              │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 5. UI Layer (Solid.js)                                              │
│    └──► emit('mod_installed') → listen() updates store              │
│    └──► Shows success toast (shared/ui/Toast)                       │
└─────────────────────────────────────────────────────────────────────┘
```

### 2. User Enables/Disables a Mod

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Mod Toggle Flow                                        │
└─────────────────────────────────────────────────────────────────────────┘

User toggles mod
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 1. UI Layer (Solid.js)                                              │
│    └──► ToggleMod feature component                                 │
│    └──► invoke('enable_mod', { mod_id })                            │
│    └──► Shows loading state (createSignal)                          │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Deploy Manager (Rust)                                            │
│    ├──► If enabling:                                               │
│    │   ├──► Check for file conflicts                               │
│    │   ├──► Create symlinks/hardlinks                              │
│    │   └──► UPDATE deployment SET status = 'deployed'               │
│    │                                                               │
│    └──► If disabling:                                              │
│        ├──► Remove symlinks                                        │
│        └──► UPDATE deployment SET status = 'pending'                │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. SQLite                                                           │
│    └──► UPDATE mods SET enabled = ?                                 │
│    └──► UPDATE deployment SET status = ?                            │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 4. UI Layer                                                         │
│    └──► emit('mod_toggled') → store update                          │
│    └──► Show success/error toast                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 3. Game Detection Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Game Detection Flow                                    │
└─────────────────────────────────────────────────────────────────────────┘

User opens app
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 1. Game Detector (Rust)                                             │
│    ├──► Scans Steam registry (winreg crate)                        │
│    ├──► Scans GOG registry                                         │
│    ├──► Scans Epic manifests                                       │
│    └──► Returns Vec<Game>                                           │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Extension System (Rust)                                          │
│    ├──► Matches detected games to extensions                       │
│    ├──► Loads game-specific configuration (JSON manifests)          │
│    └──► Registers game handlers                                     │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. SQLite                                                           │
│    └──► INSERT OR REPLACE INTO games ...                            │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 4. UI Layer (Solid.js)                                              │
│    └──► gameStore.loadGames() → createResource                      │
│    └──► Shows game cards on dashboard (widgets/GameCard)            │
└─────────────────────────────────────────────────────────────────────┘
```

### 4. Download Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Download Flow                                          │
└─────────────────────────────────────────────────────────────────────────┘

User clicks "Download"
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 1. Download Manager (Rust)                                          │
│    ├──► Resolves download URL                                      │
│    ├──► Probes server (reqwest HEAD request)                       │
│    ├──► Chooses strategy (chunked vs single)                       │
│    └──► Starts download with progress tracking                     │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. UI Layer (Solid.js)                                              │
│    └──► listen('download_progress') → update progress signal        │
│    └──► Shows download queue (pages/downloads)                      │
│    └──► Allows pause/resume/cancel                                  │
└─────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. Mod Installer (Rust)                                             │
│    └──► After download completes:                                  │
│        ├──► Detects mod type                                       │
│        ├──► Extracts to staging                                    │
│        └──► Registers in SQLite                                    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Module Communication

### Tauri IPC (Commands + Events)

| Type | Direction | Purpose |
|------|------------|---------|
| `#[tauri::command]` | UI → Rust | Request/response (invoke) |
| `app.emit()` | Rust → UI | One-way events (listen) |

### Tauri Commands

| Command | Input | Output |
|---------|-------|--------|
| `get_games` | - | `Vec<Game>` |
| `detect_games` | - | `Vec<Game>` |
| `install_mod` | `game_id, archive_path` | `Mod` |
| `uninstall_mod` | `mod_id` | `()` |
| `enable_mod` | `mod_id` | `()` |
| `disable_mod` | `mod_id` | `()` |
| `deploy_mod` | `mod_id` | `DeploymentState` |
| `undeploy_mod` | `mod_id` | `()` |
| `get_load_order` | `game_id` | `Vec<Plugin>` |
| `set_load_order` | `game_id, order` | `()` |
| `start_download` | `resource, dest` | `String` (ID) |
| `pause_download` | `id` | `()` |
| `resume_download` | `id` | `()` |
| `get_download_progress` | `id` | `DownloadProgress` |

### Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `download_progress` | `DownloadProgress` | Progress updates |
| `download_completed` | `DownloadResult` | Download finished |
| `download_failed` | `DownloadError` | Download error |
| `mod_installed` | `Mod` | Mod installation complete |
| `deploy_completed` | `DeploymentState` | Deployment finished |
| `conflict_detected` | `Conflict` | File conflict found |

---

## Data Flow

### State Ownership

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    State Ownership                                        │
└─────────────────────────────────────────────────────────────────────────┘

Module                │ Owns
──────────────────────┼────────────────────────────────────────────────────
Game Detector         │ Detected games (Vec<Game>)
Mod Installer         │ Mod staging data
Deploy Manager        │ Deployment state, conflicts
Download Manager      │ Download queue, progress
SQLite                │ All persistent data (games, mods, settings)
Extension System      │ Extension registry
UI Layer (Solid.js)   │ UI state (loading, errors, selections)
```

### Data Sharing

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Data Flow                                              │
└─────────────────────────────────────────────────────────────────────────┘

Game Detector ──► SQLite ──► invoke('get_games') ──► gameStore ──► UI
Mod Installer ──► SQLite ──► invoke('get_mods') ──► modStore ──► UI
Deploy Manager ──► SQLite ──► invoke('get_deployment') ──► deployStore ──► UI
Download Manager ──► SQLite ──► invoke('list_downloads') ──► downloadStore ──► UI
Load Order Manager ──► SQLite ──► invoke('get_load_order') ──► loadOrderStore ──► UI
```

---

## Key Interactions

### Module Interaction Matrix

| Module A | Module B | Interaction Type |
|----------|----------|-----------------|
| Game Detector | SQLite | INSERT discovered games |
| Game Detector | Extension System | Match games to extensions |
| Mod Installer | SQLite | INSERT mod records |
| Mod Installer | Deploy Manager | Pass mod for deployment |
| Mod Installer | Download Manager | Receive downloaded files |
| Mod Installer | Security Validator | Validate archive before install |
| Mod Installer | Dependency Resolver | Check dependencies |
| Deploy Manager | SQLite | UPDATE deployment state |
| Deploy Manager | Load Order Manager | Update plugin list |
| Deploy Manager | Backup Manager | Backup before deployment |
| Download Manager | SQLite | INSERT download records |
| Download Manager | Mod Installer | Pass downloaded files |
| Download Manager | Repository API Client | Resolve download URLs |
| Load Order Manager | SQLite | UPDATE load order |
| Extension System | Game Detector | Provide game detection |
| Extension System | Mod Installer | Provide mod type handlers |
| Profile Manager | Deploy Manager | Redeploy on profile switch |
| Profile Manager | Load Order Manager | Per-profile load order |
| Profile Manager | SQLite | Store profile metadata |
| Game Launcher | Deploy Manager | Ensure mods deployed before launch |
| Game Launcher | Profile Manager | Use profile launch args |
| Game Launcher | Mod Installer | Install loader files |
| Security Validator | Mod Installer | Pre-install validation |
| Security Validator | Download Manager | Post-download validation |
| Security Validator | Game Launcher | Pre-launch compatibility |
| Update Checker | Repository API Client | Check mod versions |
| Update Checker | Download Manager | Download updates |
| Update Checker | Mod Installer | Install updates |
| Backup Manager | Deploy Manager | Backup before changes |
| Backup Manager | Profile Manager | Profile state snapshots |
| Backup Manager | Game Launcher | Save game backups |
| Dependency Resolver | Mod Installer | Pre-install checks |
| Dependency Resolver | Repository API Client | Fetch dependency info |
| Dependency Resolver | Load Order Manager | Dependency-based ordering |
| Nexus API Client | Download Manager | Provide download URLs |
| Nexus API Client | Update Checker | Version check source |
| UI Layer | All Modules | Tauri invoke() + listen() |

---

## Error Propagation

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Error Flow                                             │
└─────────────────────────────────────────────────────────────────────────┘

Rust Error ──► Result<T, String> ──► Tauri Command ──► invoke() catch
                                                                    │
                                                                    ▼
                                                          UI Error State
                                                                    │
                                                                    ▼
                                                          Toast / ErrorBoundary

Error Types (Rust):
├── GameNotFound(String)
├── ModAlreadyInstalled(String)
├── DeploymentFailed(String)
├── ConflictDetected(Vec<Conflict>)
├── DownloadError(DownloadErrorKind)
├── ExtensionError(String)
├── DatabaseError(String)
└── IoError(String)
```

---

## Startup Sequence

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Application Startup                                    │
└─────────────────────────────────────────────────────────────────────────┘

1. Tauri Application (Rust)
   ├──► Initialize logging (tracing)
   ├──► Load configuration (tauri.conf.json)
   └──► Create main window

2. SQLite (rusqlite + r2d2)
   ├──► Open/create database file
   ├──► Run migrations
   └──► Initialize connection pool

3. Extension System (Rust)
   ├──► Scan extensions/ directory
   ├──► Load extension manifests (JSON)
   └──► Initialize extensions (traits)

4. Game Detector (Rust)
   ├──► Load game extensions
   └──► Scan for games (Steam, GOG, Epic)

5. Solid.js Frontend
   ├──► Initialize router (@solidjs/router)
   ├──► Initialize stores (createStore)
   ├──► Load initial data (createResource)
   └──► Render dashboard (Panda CSS)

6. Download Manager (Rust)
   └──► Resume interrupted downloads
```
