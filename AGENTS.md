# AGENTS.md - Pantheon Development Guide

## Documentation

Links to all documentation in this repo (paths relative to project root), with short notes on when to use each file.

### Core specifications (`docs/`)

| Document | Purpose |
|----------|---------|
| [docs/AI-OPTIMIZED.md](docs/AI-OPTIMIZED.md) | Master AI doc index, quick start, recommended reading order |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System overview: layers, modules, data flow |
| [docs/MODELS.md](docs/MODELS.md) | Rust + TypeScript types and Tauri JSON / IPC naming rules |
| [docs/DATABASE_SCHEMA.md](docs/DATABASE_SCHEMA.md) | SQL DDL, migrations, SQLite indexes |
| [docs/MODULE_SPECS.md](docs/MODULE_SPECS.md) | Aggregated module specs: responsibilities and public APIs |
| [docs/FLOWS.md](docs/FLOWS.md) | Step-by-step flows and user journeys for implementation |
| [docs/API_REFERENCE.md](docs/API_REFERENCE.md) | Tauri commands and events reference |

### Cross-cutting: integration, platform, roadmap

| Document | Purpose |
|----------|---------|
| [docs/MODULE_CONNECTIONS.md](docs/MODULE_CONNECTIONS.md) | How modules connect: dependencies, data flow, communication |
| [docs/DEEP_LINKS.md](docs/DEEP_LINKS.md) | Custom URL schemes (`pantheon://`), OAuth callbacks, external deep links |
| [docs/FEATURE_AUDIT.md](docs/FEATURE_AUDIT.md) | Feature-to-module mapping, gaps, and priorities vs other mod managers |
| [docs/CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md) | Windows / Linux / macOS support matrix and platform adapters |

### Module deep dives (`docs/modules/`)

| Document | Purpose |
|----------|---------|
| [docs/modules/managed-game-context.md](docs/modules/managed-game-context.md) | “Current game” context: store, routes, sidebar (Vortex-style) |
| [docs/modules/ui-structure.md](docs/modules/ui-structure.md) | Frontend: Solid.js, FSD, routing, Panda CSS, UI structure |
| [docs/modules/game-detector.md](docs/modules/game-detector.md) | Detecting and registering installed games |
| [docs/modules/game-launcher.md](docs/modules/game-launcher.md) | Launching games from the app |
| [docs/modules/mod-installer.md](docs/modules/mod-installer.md) | Installing mods from archives, FOMOD, and related behavior |
| [docs/modules/deploy-manager.md](docs/modules/deploy-manager.md) | Deploy/stage files into the game folder; enable/disable |
| [docs/modules/download-manager.md](docs/modules/download-manager.md) | Download queue and source integration |
| [docs/modules/mod-repository-api.md](docs/modules/mod-repository-api.md) | Remote mod catalog / repository API |
| [docs/modules/dependency-resolution.md](docs/modules/dependency-resolution.md) | Mod dependency graph and resolution |
| [docs/modules/load-order-manager.md](docs/modules/load-order-manager.md) | Plugin / mod load order |
| [docs/modules/profile-manager.md](docs/modules/profile-manager.md) | Per-game mod profiles |
| [docs/modules/database-manager.md](docs/modules/database-manager.md) | SQLite layer and persistence |
| [docs/modules/security-validation.md](docs/modules/security-validation.md) | Path/archive/mod validation and security |
| [docs/modules/backup-restore.md](docs/modules/backup-restore.md) | Backup and restore |
| [docs/modules/extension-system.md](docs/modules/extension-system.md) | Extensions and plugin architecture |
| [docs/modules/update-checker.md](docs/modules/update-checker.md) | Application update checks |

---

## Project Overview

Pantheon is a cross-platform desktop mod manager built with Tauri 2.x, Solid.js, and Rust.

**Tech Stack:**
- Desktop Framework: Tauri 2.x
- UI Framework: Solid.js with TypeScript
- UI Components: Ark UI (@ark-ui/solid) — Headless UI primitives
- Styling: Panda CSS — Design tokens + compile-time CSS
- Backend: Rust
- Database: SQLite (rusqlite)
- Architecture: Feature-Sliced Design v2.1 (FSD)

**Design Guidelines:**
- Dark theme with modern gaming aesthetic
- Use Ark UI headless components (Switch, Dialog, etc.) for all interactive elements
- Sidebar navigation layout
- Gradient accents, smooth animations, polished transitions
- Follow design references for visual style

---

## Build Commands

### Rust (Tauri)

```bash
# Install dependencies
cargo fetch

# Development mode with hot reload
cargo tauri dev

# Build for production
cargo tauri build

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific module
cargo test module_name

# Run tests with specific pattern
cargo test test_name_pattern

# Check code formatting
cargo fmt --check

# Auto-format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Build release binary
cargo tauri build --release
```

### Frontend (Solid.js)

```bash
# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build

# Preview production build
npm run preview

# Type check
npm run typecheck

# Lint
npm run lint

# Lint with auto-fix
npm run lint:fix

# Run tests (Vitest)
npm test

# Run tests in watch mode
npm run test:watch

# Run tests for a specific file
npm test -- src/shared/lib/format-date.test.ts

# Run tests matching pattern
npm test -- --grep "toggle-mod"
```

---

## Code Style Guidelines

### TypeScript / Solid.js

**Imports:**
- Use absolute imports with `@/` prefix for FSD layers
- Order imports: external → internal → relative
- Named exports preferred over default exports for components

```typescript
// Good
import { Button } from '@/shared/ui/Button';
import { installMod } from '@/features/install-mod';
import type { Game } from '@/entities/game';

// Avoid barrel exports from index files when possible
// Direct imports are faster for bundlers
```

**Types:**
- Use `type` for type aliases, `interface` for object shapes
- Avoid `any` - use `unknown` and type guards
- Use `undefined` vs `null` consistently (prefer `undefined`)

```typescript
// Good
type ModId = string;
interface Mod {
    id: ModId;
    name: string;
    enabled: boolean;
}
type ModList = Mod[];

// Avoid
interface Mod extends SomeBase {}
```

**Components (Solid.js):**
- Use `.tsx` extension for components with JSX
- Use `Component` type from solid-js
- Props should be typed with an interface
- Use `splitProps` for destructuring props

```typescript
// Good
import { Component, splitProps, JSX } from 'solid-js';

interface GameCardProps {
    game: Game;
    onSelect: (id: string) => void;
    class?: string;
}

export const GameCard: Component<GameCardProps> = (props) => {
    const [local, others] = splitProps(props, ['game', 'onSelect', 'class']);
    
    return (
        <div class={local.class} onClick={() => local.onSelect(local.game.id)}>
            {local.game.name}
        </div>
    );
};
```

**Ark UI Components:**
- Use Ark UI headless components for all interactive elements
- Import from `@ark-ui/solid/{component}` (e.g., `@ark-ui/solid/switch`)
- Style using CSS attributes and data attributes
- Supported components: Switch, Dialog, Tabs, Popover, etc.

```typescript
// Good - Ark UI Switch
import { Switch } from '@ark-ui/solid/switch';

<Switch.Root checked={isEnabled()} onCheckedChange={(e) => setEnabled(e.checked)}>
  <Switch.Control>
    <Switch.Thumb />
  </Switch.Control>
  <Switch.Label>Enable mod</Switch.Label>
  <Switch.HiddenInput />
</Switch.Root>

// Good - Ark UI Dialog
import { Dialog } from '@ark-ui/solid/dialog';

<Dialog.Root open={isOpen()} onOpenChange={(e) => setOpen(e.open)}>
  <Dialog.Backdrop />
  <Dialog.Positioner>
    <Dialog.Content>
      <Dialog.Title>Confirm</Dialog.Title>
      <Dialog.Description>Are you sure?</Dialog.Description>
      <Dialog.CloseTrigger>Close</Dialog.CloseTrigger>
    </Dialog.Content>
  </Dialog.Positioner>
</Dialog.Root>
```

**Styling (Panda CSS):**
- Use CSS custom properties for theming
- Follow dark theme color palette from `index.css`
- Use design tokens: `--bg-primary`, `--accent-primary`, `--radius-md`, etc.
- Sidebar layout with fixed sidebar + scrollable main content

**Signals and Stores:**
- Use `createSignal` for simple state
- Use `createStore` for complex nested state
- Use `createMemo` for derived state
- Prefix event handlers with `handle`

```typescript
// Good
const [count, setCount] = createSignal(0);
const doubled = createMemo(() => count() * 2);
const handleIncrement = () => setCount(c => c + 1);
```

**Async Operations:**
- Use `createResource` for data fetching
- Handle errors with `ErrorBoundary` component

---

### Rust

**Naming Conventions:**
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions: `snake_case`
- Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Type parameters: `PascalCase`

```rust
// Good
struct GameDetector;
enum ModType { Simple, Fomod }
fn detect_games() -> Result<Vec<Game>, String>;
const MAX_RETRIES: u32 = 3;

// Avoid
struct gameDetector;
enum mod_type { simple, fomod }
fn detectGames() -> Result<Vec<Game>, String>;
```

**Error Handling:**
- Use `thiserror` for custom error types
- Use `?` operator for propagation
- Return `Result<T, String>` from Tauri commands (auto-converted)

```rust
// Good
#[derive(Debug, thiserror::Error)]
pub enum PantheonError {
    #[error("Game not found: {0}")]
    GameNotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[tauri::command]
async fn get_game(id: String) -> Result<Game, String> {
    db.find_game(&id).await.map_err(|e| e.to_string())
}
```

**Async:**
- Use `tokio` with `#[tokio::main]`
- Avoid blocking operations on async runtime

**Modules:**
- One module per file, filename matches module name
- Use `mod.rs` for module declarations

---

## Naming Conventions

### Rust (Backend)
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions: `snake_case`
- Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### TypeScript (Frontend)
- Types/Interfaces: `PascalCase`
- Functions/Variables: `camelCase`
- Files: `camelCase` or `kebab-case`

### Database
- Tables: `snake_case`
- Columns: `snake_case`

---

## Documentation for AI Code Generation

AI-oriented docs live under `docs/`. For the full file list and what each file is for, see **[Documentation](#documentation)** above.

**Reading Order for AI:**
1. [docs/AI-OPTIMIZED.md](docs/AI-OPTIMIZED.md) — navigation and quick start
2. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — system overview
3. [docs/MODELS.md](docs/MODELS.md) — types and data contracts
4. [docs/DATABASE_SCHEMA.md](docs/DATABASE_SCHEMA.md) — database schema and migrations
5. [docs/MODULE_SPECS.md](docs/MODULE_SPECS.md) — modules and APIs
6. [docs/FLOWS.md](docs/FLOWS.md) — implementation flows
7. [docs/API_REFERENCE.md](docs/API_REFERENCE.md) — Tauri commands and events

**Key Points for AI:**
- All types from `MODELS.md` must be implemented exactly
- All SQL schemas from `DATABASE_SCHEMA.md` must be created
- All flows from `FLOWS.md` must be followed
- All commands from `API_REFERENCE.md` must be registered
- Error handling returns `Result<T, String>`

---

## Architecture (FSD v2.1)

### Layer Order (Top-Down)

```
app/       → App initialization, providers, routing (NO business logic)
pages/     → Route-level composition (own state, call features)
widgets/   → Large composite blocks reused across pages
features/  → Reusable user interactions (2+ pages use it)
entities/  → Business domain models (2+ features use it)
shared/    → Infrastructure: UI kit, API client, utils (NO business logic)
```

### Import Rules
- Modules may ONLY import from layers strictly below them
- `pages/dashboard` can import from `shared`, `entities`, `features`
- `features/toggle-mod` can import from `shared`, `entities`
- `shared` cannot import from any other layer

### Public API Pattern

Each slice exports via `index.ts`:
```typescript
// features/toggle-mod/index.ts
export { ToggleMod } from './ui/ToggleMod';
export { installMod } from './model/install-model';
```

---

## File Structure

```
src/
├── app/                    # App initialization
│   ├── App.tsx
│   ├── index.tsx
│   ├── providers/
│   │   └── ToastProvider.tsx
│   └── router/
│       └── index.tsx
├── pages/                  # Route-level (pages/**/index.ts + ui/**)
├── widgets/                # Reusable composites (GameCard, ModList)
├── features/               # User interactions (install-mod, toggle-mod)
├── entities/               # Business models (game, mod, deployment)
│   └── {entity}/
│       ├── index.ts
│       ├── model/         # Types + stores
│       └── api/           # Tauri invoke wrappers
└── shared/                # Infrastructure
    ├── ui/                # Button, Input, Modal, Card, etc.
    ├── api/               # client.ts + entity APIs
    ├── lib/               # formatDate, debounce, etc.
    └── config/            # routes.ts, constants
```

---

## Tauri Commands

- Commands go in `src-tauri/src/commands/`
- One file per module (games.rs, mods.rs, deploy.rs)
- Use `#[tauri::command]` attribute
- Async commands return `Result<T, String>`
- **See `docs/API_REFERENCE.md` for complete command and event reference**

```rust
// src-tauri/src/commands/mods.rs
#[tauri::command]
pub async fn install_mod(
    game_id: String,
    archive_path: String,
) -> Result<Mod, String> {
    ModInstaller::new()
        .install(&game_id, &archive_path)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Database (SQLite)

- Use `rusqlite` with bundled SQLite
- Schema migrations in `src-tauri/src/db/migrations/`
- Connection pool via `r2d2` or single connection for simplicity
- Prepared statements for repeated queries
- **See `docs/DATABASE_SCHEMA.md` for complete SQL DDL and schema**

---

## Testing

### Rust
- Unit tests: co-located in module with `#[cfg(test)]`
- Integration tests: `tests/` directory
- Use `#[tokio::test]` for async tests

### Frontend (Vitest)
- Tests next to source files: `*.test.ts`
- Use `@testing-library/solid` for component tests
- Mock Tauri invoke with `vi.mock()`

---

## Git Conventions

- Commits: Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`)
- Branch names: `feature/`, `fix/`, `refactor/`
- PR description: What and Why (not How)

---

## IDE Setup

- VS Code with `rust-analyzer` and `volar` (Solid.js)
- Set `typescript.preferences.importModuleSpecifier` to `non-relative`
- Enable `editor.formatOnSave`


При редактирование кода пожалуйста записывай в документацию изменение логики или добавляй новую информацию если ты пишешь новый код