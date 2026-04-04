# Pantheon MVP Implementation Plan

## Overview

This plan breaks the Pantheon MVP into sequential, atomic tasks that can be implemented by an AI agent one at a time. Each task is self-contained with clear inputs and outputs.

**MVP Goal:** User can detect a game, install a mod from a local archive, enable/disable it, and deploy it to the game folder.

**Tech Stack:** Tauri 2.x, Solid.js, Rust, SQLite, Feature-Sliced Design v2.1

**UI Libraries:**
- **Ark UI** (`@ark-ui/solid`) — Headless UI components (Switch, Dialog, etc.)
- **Panda CSS** — Styling with design tokens

**Design Guidelines:**
- Dark theme with modern gaming aesthetic
- Use Ark UI headless components for all interactive elements
- Follow the visual style from design references (see screenshots)
- Sidebar navigation layout with game library and mod management
- Gradient accents, smooth animations, and polished transitions

---

## Phase 0: Project Scaffolding

### Task 0.1 — Initialize Tauri 2.x Project
**What:** Create Tauri 2.x project with Solid.js template
**Commands:**
```bash
npm create tauri-app@latest pantheon -- --template solidjs-ts
cd pantheon
cargo fetch
npm install
```
**Verify:** `cargo tauri dev` opens a window with "Tauri + Solid" text
**Output:** Working Tauri 2.x + Solid.js project structure

### Task 0.2 — Configure Rust Dependencies
**What:** Add required crates to `src-tauri/Cargo.toml`
**Dependencies:**
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
zip = "0.6"
thiserror = "1"
tokio = { version = "1", features = ["full"] }
tauri = { version = "2", features = ["shell-open"] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
```
**Verify:** `cargo fetch` succeeds

### Task 0.3 — Configure Frontend Dependencies
**What:** Add required npm packages
**Dependencies:**
```bash
npm install @solidjs/router @tauri-apps/api@2
```
**Verify:** `npm install` succeeds

### Task 0.4 — Set Up FSD Directory Structure
**What:** Create Feature-Sliced Design folder structure
**Directories:**
```
src/
├── app/
│   ├── providers/
│   └── router/
├── pages/
│   ├── dashboard/
│   └── game-detail/
├── widgets/
├── features/
├── entities/
│   ├── game/
│   └── mod/
└── shared/
    ├── ui/
    ├── api/
    ├── lib/
    └── config/
```
**Verify:** All directories exist

---

## Phase 1: Database Layer

### Task 1.1 — Create Migration System
**Files:** `src-tauri/src/db/mod.rs`, `src-tauri/src/db/migrations/001_initial_schema.sql`
**What:**
- SQL migration for core tables: `games`, `mods`, `modFiles`, `deployment`
- Migration runner that applies pending migrations on app start
- Only MVP tables (skip profiles, downloads, loadOrder, plugins, etc.)

**SQL Schema (simplified for MVP):**
```sql
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    installPath TEXT NOT NULL,
    supportPath TEXT NOT NULL,
    launcher TEXT NOT NULL,
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE mods (
    id TEXT PRIMARY KEY,
    gameId TEXT NOT NULL REFERENCES games(id),
    name TEXT NOT NULL,
    version TEXT,
    modType TEXT NOT NULL DEFAULT 'simple',
    installPath TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    installTime TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(gameId, name)
);

CREATE TABLE modFiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    UNIQUE(modId, path)
);

CREATE TABLE deployment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL UNIQUE REFERENCES mods(id) ON DELETE CASCADE,
    gameId TEXT NOT NULL REFERENCES games(id),
    status TEXT NOT NULL DEFAULT 'pending',
    strategy TEXT NOT NULL DEFAULT 'copy',
    deployedFiles TEXT DEFAULT '[]',
    deployedAt TEXT
);

CREATE TABLE schemaMigrations (
    version INTEGER PRIMARY KEY,
    appliedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Verify:** Migration runs on app start, tables are created
**Output:** `Database` struct with `new()`, `migrate()`, and connection methods

### Task 1.2 — Implement Database Queries
**Files:** `src-tauri/src/db/queries.rs`
**What:** CRUD operations for MVP tables
**Functions:**
```rust
// Games
fn insert_or_update_game(&self, game: &Game) -> Result<()>
fn list_games(&self) -> Result<Vec<Game>>
fn find_game(&self, id: &str) -> Result<Option<Game>>
fn delete_game(&self, id: &str) -> Result<()>

// Mods
fn insert_mod(&self, mod: &Mod) -> Result<()>
fn list_mods(&self, game_id: &str) -> Result<Vec<Mod>>
fn find_mod(&self, id: &str) -> Result<Option<Mod>>
fn update_mod_enabled(&self, id: &str, enabled: bool) -> Result<()>
fn delete_mod(&self, id: &str) -> Result<()>

// Mod Files
fn insert_mod_file(&self, file: &ModFile) -> Result<()>
fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>>

// Deployment
fn upsert_deployment(&self, state: &DeploymentState) -> Result<()>
fn get_deployment_state(&self, mod_id: &str) -> Result<Option<DeploymentState>>
fn list_deployments(&self, game_id: &str) -> Result<Vec<DeploymentState>>
```
**Verify:** Unit tests for each query with in-memory SQLite
**Output:** Complete database query layer

---

## Phase 2: Rust Backend Services

### Task 2.1 — Define Rust Data Models
**Files:** `src-tauri/src/models.rs`
**What:** Core structs matching MODELS.md (MVP subset)
**Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub install_path: PathBuf,
    pub support_path: PathBuf,
    pub launcher: String, // "steam", "gog", "manual"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub version: Option<String>,
    pub mod_type: String, // "simple" for MVP
    pub install_path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub id: i64,
    pub mod_id: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub mod_id: String,
    pub game_id: String,
    pub status: String, // "pending", "deployed", "failed"
    pub strategy: String, // "copy" for MVP
    pub deployed_files: Vec<DeployedFile>,
    pub deployed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedFile {
    pub source: String,
    pub target: String,
    pub size: u64,
}
```
**Verify:** `cargo check` compiles
**Output:** All MVP types defined with serde

### Task 2.2 — Implement GameDetector (Steam only)
**Files:** `src-tauri/src/services/game_detector.rs`
**What:** Detect Skyrim/Fallout4 via Steam library
**Logic:**
1. Read Steam path from registry (`HKCU\Software\Valve\Steam\SteamPath`)
2. Parse `steamapps/libraryfolders.vdf` for library paths
3. Scan each library for known game executables
4. Return `Vec<Game>` for found games

**MVP Games to detect:**
```rust
const KNOWN_GAMES: &[(&str, &str, &str)] = &[
    ("skyrim", "Skyrim", "TESV.exe"),
    ("skyrimse", "Skyrim Special Edition", "SkyrimSE.exe"),
    ("fallout4", "Fallout 4", "Fallout4.exe"),
];
```
**Simplified approach for MVP:** If registry scan is complex, provide a "manual game registration" fallback where user picks the game folder via dialog.
**Verify:** Returns games when Steam is installed
**Output:** `GameDetector` struct with `detect_games()` method

### Task 2.3 — Implement ModInstaller (zip only)
**Files:** `src-tauri/src/services/mod_installer.rs`
**What:** Extract zip archives to staging directory
**Logic:**
1. Generate UUID for mod
2. Create staging path: `staging/mods/{uuid}/`
3. Extract zip archive to staging path using `zip` crate
4. Parse metadata if `mod.json` exists in archive root
5. Create `Mod` record in database
6. Create `ModFile` records for each extracted file

**Code structure:**
```rust
pub struct ModInstaller {
    staging_path: PathBuf,
}

impl ModInstaller {
    pub fn new(staging_path: PathBuf) -> Self;
    pub async fn install(&self, game_id: &str, archive_path: &Path) -> Result<Mod, String>;
    pub async fn uninstall(&self, mod_id: &str) -> Result<(), String>;
}
```
**Verify:** Extracts a test zip to staging, creates DB records
**Output:** `ModInstaller` service

### Task 2.4 — Implement DeployManager (copy strategy)
**Files:** `src-tauri/src/services/deploy_manager.rs`
**What:** Copy mod files from staging to game folder
**Logic:**
1. Get mod and its files from database
2. For each file: `staging/mods/{modId}/{path}` → `{gameSupportPath}/{path}`
3. Create parent directories as needed
4. Use `std::fs::copy` (simplest strategy for MVP)
5. Update deployment state in database
6. Update mod `enabled` flag

**Code structure:**
```rust
pub struct DeployManager {
    db: Database,
}

impl DeployManager {
    pub fn new(db: Database) -> Self;
    pub async fn deploy_mod(&self, mod_id: &str) -> Result<DeploymentState, String>;
    pub async fn undeploy_mod(&self, mod_id: &str) -> Result<(), String>;
    pub async fn enable_mod(&self, mod_id: &str) -> Result<(), String>;
    pub async fn disable_mod(&self, mod_id: &str) -> Result<(), String>;
}
```

**For `enable_mod`:**
- Set mod.enabled = true in DB
- Call `deploy_mod`

**For `disable_mod`:**
- Set mod.enabled = false in DB
- Call `undeploy_mod` (remove copied files from game folder)

**Verify:** Files are copied to game folder, deployment state updated
**Output:** `DeployManager` service

---

## Phase 3: Tauri Commands

### Task 3.1 — Game Commands
**Files:** `src-tauri/src/commands/games.rs`
**Commands:**
```rust
#[tauri::command]
pub async fn get_games() -> Result<Vec<Game>, String>

#[tauri::command]
pub async fn detect_games() -> Result<Vec<Game>, String>

#[tauri::command]
pub async fn register_game(game: Game) -> Result<Game, String>

#[tauri::command]
pub async fn unregister_game(game_id: String) -> Result<(), String>
```
**Verify:** Commands are registered in `main.rs`, callable from frontend
**Output:** Game command handlers

### Task 3.2 — Mod Commands
**Files:** `src-tauri/src/commands/mods.rs`
**Commands:**
```rust
#[tauri::command]
pub async fn install_mod(game_id: String, archive_path: String) -> Result<Mod, String>

#[tauri::command]
pub async fn uninstall_mod(mod_id: String) -> Result<(), String>

#[tauri::command]
pub async fn get_mods(game_id: String) -> Result<Vec<Mod>, String>

#[tauri::command]
pub async fn set_mod_enabled(mod_id: String, enabled: bool) -> Result<(), String>
```
**Verify:** Commands work end-to-end with test archive
**Output:** Mod command handlers

### Task 3.3 — Deployment Commands
**Files:** `src-tauri/src/commands/deploy.rs`
**Commands:**
```rust
#[tauri::command]
pub async fn deploy_mod(mod_id: String) -> Result<DeploymentState, String>

#[tauri::command]
pub async fn undeploy_mod(mod_id: String) -> Result<(), String>

#[tauri::command]
pub async fn deploy_all(game_id: String) -> Result<Vec<DeploymentState>, String>
```
**Verify:** Deployment works via command invocation
**Output:** Deploy command handlers

### Task 3.4 — Register Commands in main.rs
**Files:** `src-tauri/src/main.rs`
**What:** Wire up all commands and initialize app state
**Structure:**
```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database and run migrations
            let db = Database::new(app.path())?;
            db.migrate()?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Games
            get_games,
            detect_games,
            register_game,
            unregister_game,
            // Mods
            install_mod,
            uninstall_mod,
            get_mods,
            set_mod_enabled,
            // Deploy
            deploy_mod,
            undeploy_mod,
            deploy_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
**Verify:** `cargo tauri dev` compiles and runs without errors
**Output:** Complete Rust backend

---

## Phase 4: Frontend — Shared Layer

### Task 4.1 — TypeScript Type Definitions
**Files:** `src/shared/types.ts`
**What:** TypeScript interfaces matching Rust models
```typescript
export interface Game {
  id: string;
  name: string;
  installPath: string;
  supportPath: string;
  launcher: string;
}

export interface Mod {
  id: string;
  gameId: string;
  name: string;
  version: string | null;
  modType: string;
  installPath: string;
  enabled: boolean;
}

export interface DeploymentState {
  modId: string;
  gameId: string;
  status: string;
  strategy: string;
  deployedFiles: DeployedFile[];
  deployedAt: string | null;
}

export interface DeployedFile {
  source: string;
  target: string;
  size: number;
}
```
**Verify:** TypeScript compiles without errors
**Output:** Shared type definitions

### Task 4.2 — API Client
**Files:** `src/shared/api/client.ts`
**What:** Wrapper for Tauri invoke
```typescript
import { invoke } from '@tauri-apps/api/core';

export const api = {
  invoke: <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    return invoke<T>(command, args);
  },
};
```
**Verify:** Can invoke a command
**Output:** API client utility

### Task 4.3 — UI Components (Button, Card)
**Files:** `src/shared/ui/Button.tsx`, `src/shared/ui/Card.tsx`
**What:** Basic reusable components
```tsx
// Button.tsx
import { Component, JSX, splitProps } from 'solid-js';

interface ButtonProps {
  onClick?: () => void;
  disabled?: boolean;
  isLoading?: boolean;
  variant?: 'primary' | 'secondary' | 'danger';
  children: JSX.Element;
  class?: string;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'variant', 'isLoading', 'children']);
  return (
    <button
      class={`btn btn-${local.variant ?? 'primary'} ${local.isLoading ? 'btn-loading' : ''} ${local.class ?? ''}`}
      disabled={local.isLoading || props.disabled}
      {...others}
    >
      {local.children}
    </button>
  );
};
```
**Verify:** Components render correctly
**Output:** Basic UI kit

---

## Phase 5: Frontend — Entities Layer

### Task 5.1 — Game Entity (Store + API)
**Files:**
- `src/entities/game/model/game.ts` (types, re-export from shared)
- `src/entities/game/api/gameApi.ts` (Tauri wrappers)
- `src/entities/game/model/gameStore.ts` (Solid.js store)
- `src/entities/game/index.ts` (public API)

**gameApi.ts:**
```typescript
import { api } from '@/shared/api/client';
import type { Game } from '@/shared/types';

export const gameApi = {
  getGames: () => api.invoke<Game[]>('get_games'),
  detectGames: () => api.invoke<Game[]>('detect_games'),
  registerGame: (game: Game) => api.invoke<Game>('register_game', { game }),
  unregisterGame: (gameId: string) => api.invoke<void>('unregister_game', { gameId }),
};
```

**gameStore.ts:**
```typescript
import { createStore } from 'solid-js/store';
import type { Game } from '@/shared/types';
import { gameApi } from '../api/gameApi';

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
      const games = await gameApi.getGames();
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
      const games = await gameApi.detectGames();
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

  return { state, loadGames, detectGames, selectGame };
};
```

**Verify:** Store loads games on mount
**Output:** Game entity with store and API

### Task 5.2 — Mod Entity (Store + API)
**Files:** Same structure as game entity
**modApi.ts:**
```typescript
import { api } from '@/shared/api/client';
import type { Mod, DeploymentState } from '@/shared/types';

export const modApi = {
  getMods: (gameId: string) => api.invoke<Mod[]>('get_mods', { gameId }),
  installMod: (gameId: string, archivePath: string) =>
    api.invoke<Mod>('install_mod', { gameId, archivePath }),
  uninstallMod: (modId: string) => api.invoke<void>('uninstall_mod', { modId }),
  setModEnabled: (modId: string, enabled: boolean) =>
    api.invoke<void>('set_mod_enabled', { modId, enabled }),
  deployMod: (modId: string) => api.invoke<DeploymentState>('deploy_mod', { modId }),
  undeployMod: (modId: string) => api.invoke<void>('undeploy_mod', { modId }),
  deployAll: (gameId: string) => api.invoke<DeploymentState[]>('deploy_all', { gameId }),
};
```

**modStore.ts:** Similar pattern with `mods: Mod[]`, `selectedGameId`, loading states
**Verify:** Store manages mod list
**Output:** Mod entity with store and API

---

## Phase 6: Frontend — Features Layer

### Task 6.1 — Detect Games Feature
**Files:**
- `src/features/detect-games/ui/DetectGamesButton.tsx`
- `src/features/detect-games/model/detectGames.ts`
- `src/features/detect-games/index.ts`

**What:** Button that triggers game detection and updates store
**Verify:** Clicking button detects games and shows them
**Output:** Detect games feature

### Task 6.2 — Install Mod Feature
**Files:**
- `src/features/install-mod/ui/InstallModButton.tsx`
- `src/features/install-mod/model/installMod.ts`
- `src/features/install-mod/index.ts`

**What:** Button + file dialog to select archive and install mod
**Uses:** `@tauri-apps/plugin-dialog` for file picker
**Verify:** Selecting a zip installs the mod
**Output:** Install mod feature

### Task 6.3 — Toggle Mod Feature
**Files:**
- `src/features/toggle-mod/ui/ToggleMod.tsx`
- `src/features/toggle-mod/model/toggleMod.ts`
- `src/features/toggle-mod/index.ts`

**What:** Toggle switch to enable/disable a mod (triggers deploy/undeploy)
**Verify:** Toggling changes mod state and deploys/undeploys
**Output:** Toggle mod feature

---

## Phase 7: Frontend — Pages Layer

### Task 7.1 — Dashboard Page
**Files:** `src/pages/dashboard/index.tsx`
**What:** Shows list of detected games with "Detect Games" button
**Layout:**
- Header with app title
- "Detect Games" button
- Grid of game cards
- Empty state when no games

**Verify:** Page renders, detects games, shows game cards
**Output:** Dashboard page

### Task 7.2 — Game Detail Page
**Files:** `src/pages/game-detail/index.tsx`
**What:** Shows mods for selected game with install/toggle controls
**Layout:**
- Game name header
- "Install Mod" button
- List of mods with toggle switches
- Deploy status indicators

**Verify:** Page shows mods, can install and toggle
**Output:** Game detail page

---

## Phase 8: Frontend — App Layer

### Task 8.1 — Router Setup
**Files:** `src/app/router/index.tsx`
**What:** Solid.js router with two routes
**Routes:**
- `/` → Dashboard page
- `/game/:id` → Game detail page

**Verify:** Navigation works between pages
**Output:** Working router

### Task 8.2 — App Root Component
**Files:** `src/app/App.tsx`, `src/app/index.tsx`
**What:** Root component with router and providers
**Verify:** App renders and routes correctly
**Output:** Complete app shell

### Task 8.3 — Basic Styling
**Files:** `src/styles.css` or inline Panda CSS config
**What:** Minimal CSS for layout, buttons, cards
**Style approach:** Use plain CSS or Panda CSS (whichever is simpler for MVP)
**Verify:** UI looks presentable
**Output:** Basic styling

---

## Phase 9: Integration & Testing

### Task 9.1 — End-to-End Flow Test
**What:** Manual test of complete MVP flow:
1. Open app → Dashboard shows
2. Click "Detect Games" → Skyrim appears (or manually register)
3. Click game → Game detail page opens
4. Click "Install Mod" → Select zip → Mod appears in list
5. Toggle mod ON → Files deployed to game folder
6. Toggle mod OFF → Files removed from game folder
7. Uninstall mod → Mod removed from list

**Verify:** All steps complete without errors
**Output:** Verified MVP

### Task 9.2 — Error Handling
**What:** Add error handling for:
- Failed game detection → show error toast
- Invalid archive → show error message
- Deployment failure → show error, rollback state
- Missing game folder → prompt user

**Verify:** Errors are caught and displayed to user
**Output:** Robust error handling

### Task 9.3 — Build Verification
**What:** Ensure both dev and production builds work
**Commands:**
```bash
cargo tauri dev      # Development
cargo tauri build    # Production
npm run typecheck    # TypeScript check
```
**Verify:** All commands succeed
**Output:** Production-ready MVP

---

## Execution Order Summary

```
Phase 0: Project Scaffolding (0.1 → 0.2 → 0.3 → 0.4)
Phase 1: Database Layer (1.1 → 1.2)
Phase 2: Rust Services (2.1 → 2.2 → 2.3 → 2.4)
Phase 3: Tauri Commands (3.1 → 3.2 → 3.3 → 3.4)
Phase 4: Frontend Shared (4.1 → 4.2 → 4.3)
Phase 5: Frontend Entities (5.1 → 5.2)
Phase 6: Frontend Features (6.1 → 6.2 → 6.3)
Phase 7: Frontend Pages (7.1 → 7.2)
Phase 8: App Layer (8.1 → 8.2 → 8.3)
Phase 9: Testing (9.1 → 9.2 → 9.3)
```

**Total tasks:** 28 atomic tasks
**Estimated time per task:** 5-15 minutes for AI agent
**Total estimated time:** 3-7 hours

---

## Post-MVP Roadmap

After MVP is verified, implement in this order:

1. **Symlink/Hardlink deployment** (replace copy strategy)
2. **Download manager** (basic HTTP download with progress)
3. **Load order management** (Bethesda plugins.txt)
4. **Profile system** (multiple mod configurations)
5. **Extension system** (game-specific handlers)
6. **FOMOD installer** (interactive mod installation)
7. **Security validator** (archive scanning)
8. **Repository integration** (mod search/download)
9. **Backup system** (game/save backups)
10. **Settings page** (deployment strategy, staging path)
