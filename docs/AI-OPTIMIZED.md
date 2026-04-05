# Pantheon AI-Optimized Documentation

## Purpose

This documentation is optimized for AI code generation. It provides complete, unambiguous specifications that enable accurate implementation without ambiguity or missing context.

## Documentation Structure

```
docs/
├── AI-OPTIMIZED.md           # This file - Master index
├── MODELS.md                 # Complete type definitions + Tauri JSON (camelCase) rules
├── DATABASE_SCHEMA.md        # Full SQL DDL with migrations + indexes
├── MODULE_SPECS.md           # Complete module specifications
├── FLOWS.md                  # Step-by-step implementation flows
├── API_REFERENCE.md          # Tauri commands and events
├── DEEP_LINKS.md            # URL scheme handling (pantheon://)
└── ARCHITECTURE.md           # System overview (start here)
```

## Quick Start for AI

### 1. Read ARCHITECTURE.md first
- Understand the overall system design
- See module relationships and data flow

### 2. Read MODELS.md
- Get complete type definitions
- Understand all data structures in both Rust and TypeScript
- Read **JSON over Tauri IPC** in MODELS.md before adding fields to `models.rs` / `shared/types.ts` (camelCase on the wire, enums lowercase)

### 3. Read DATABASE_SCHEMA.md
- Get exact SQL DDL
- Understand migrations and indexes

### 4. Read MODULE_SPECS.md
- Get detailed module specifications
- Understand module responsibilities and APIs

### 5. Read FLOWS.md
- Get step-by-step implementation flows
- Understand user interactions and system responses

### 6. Reference API_REFERENCE.md
- Look up specific commands and events during implementation

## Local development (npm)

| Script | Purpose |
|--------|---------|
| `npm run dev` | Vite dev server (frontend only) |
| `npm run tauri` | `tauri dev` — desktop app with backend (see `package.json`) |

---

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Desktop Framework | Tauri | 2.x |
| UI Framework | Solid.js | latest |
| Language (Backend) | Rust | stable |
| Language (Frontend) | TypeScript | 5.x |
| Database | SQLite | bundled via rusqlite |
| Connection Pool | r2d2 | latest |
| State Management | Solid.js Stores | native |
| Styling | Panda CSS | latest |
| Architecture | Feature-Sliced Design | v2.1 |

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
- Indexes: `idx_<table>_<column>`

## Code Generation Guidelines

### Rust Backend
1. All Tauri commands return `Result<T, String>`
2. Use `thiserror` for custom error types
3. Use prepared statements for all queries
4. Implement `Send + Sync` for all shared types

### TypeScript Frontend
1. Use `Component` type from solid-js
2. Use `splitProps` for destructuring props
3. Use `createSignal` for simple state
4. Use `createStore` for complex state
5. Use `createResource` for async data

### Database
1. Always use prepared statements
2. Use transactions for multi-table operations
3. Store JSON as TEXT with proper serialization (e.g. `games.details` should align with `GameDetails` — camelCase for new data; see MODELS.md)
4. Track migrations in schema_migrations table

## Key Patterns

### Tauri Command Pattern
```rust
#[tauri::command]
pub async fn command_name(
    param1: String,
    param2: i32,
) -> Result<ReturnType, String> {
    // Implementation
    Ok(return_value)
}
```

### Event Emission Pattern
```rust
app.emit("event_name", payload)?;
```

### Solid.js Store Pattern
```typescript
const [store, setStore] = createStore<StoreType>({
  // initial state
});

const createMemoDerived = createMemo(() => {
  // derived state computation
});
```

### Feature Module Pattern (FSD)
```
features/
├── feature-name/
│   ├── index.ts              # Public API
│   ├── ui/
│   │   └── FeatureName.tsx   # Component
│   └── model/
│       └── featureModel.ts   # Business logic
```

## Validation Checklist

Before code is considered complete, ensure:

- [ ] All types from MODELS.md are implemented
- [ ] All SQL schemas from DATABASE_SCHEMA.md are created
- [ ] All module APIs from MODULE_SPECS.md are implemented
- [ ] All flows from FLOWS.md are covered
- [ ] All commands from API_REFERENCE.md are registered
- [ ] Error handling returns `Result<T, String>`
- [ ] New/changed Rust models that cross Tauri IPC use serde rules in `models.rs` so the UI receives camelCase JSON
- [ ] Events are emitted for all state changes
- [ ] No `any` types used (use `unknown` with type guards)
- [ ] All async operations use tokio

## Game Support Priority

### Phase 1: Bethesda Games (Creation Kit)
- Skyrim (all versions)
- Fallout 4
- Oblivion
- Fallout New Vegas

These games share:
- ESM/ESP/ESL plugin format
- BSA/BA2 archive format
- Archive invalidation mechanism
- Script extender (SKSE/F4SE)

### Phase 2: Unity Games (BepInEx)
- Valheim
- Palworld
- Subnautica

### Phase 3: Other Games
- Generic mod support
- Cross-platform (Linux/SteamOS)

## Extension Architecture

Extensions are Rust traits that register:
- Game detection handlers
- Mod type handlers
- Custom installers

See MODULE_SPECS.md for extension API details.

## File Location Reference

```
src-tauri/
├── src/
│   ├── main.rs               # Entry point
│   ├── commands/             # Tauri commands
│   │   ├── games.rs
│   │   ├── mods.rs
│   │   ├── deploy.rs
│   │   └── ...
│   ├── db/                   # Database layer
│   │   ├── mod.rs
│   │   ├── migrations/
│   │   └── queries/
│   ├── services/             # Business logic
│   │   ├── mod_installer.rs
│   │   ├── deploy_manager.rs
│   │   └── ...
│   └── extensions/           # Extension system
│       └── mod.rs

src/
├── app/                      # App initialization
├── pages/                    # Route pages
├── widgets/                  # Reusable components
├── features/                 # User interactions
├── entities/                 # Business models + stores
└── shared/                   # UI kit, API, utils
```

## Critical Implementation Notes

### Windows Symlinks
- Symlinks require admin privileges or Developer Mode
- Fallback to hardlinks when symlinks unavailable
- Consider directory junctions for folders

### Bethesda Games
- BSA files must be sorted by name for proper loading
- Archive invalidation requires.ini manipulation
- ESM files load before ESP files
- Light plugins (ESL) have special handling

### Mod Deployment
- Mods stage in `staging/mods/{modId}/`
- Deploy creates symlinks/hardlinks in game folder
- Conflict detection must happen before deployment
- Profile switching triggers re-deployment

### Download Manager
- Use reqwest for HTTP
- Support Range requests for resume
- Track progress via events
- Store download metadata in SQLite

---

For detailed specifications, see:
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System overview
- [MODELS.md](./MODELS.md) - Type definitions
- [DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md) - SQL schemas
- [MODULE_SPECS.md](./MODULE_SPECS.md) - Module details
- [FLOWS.md](./FLOWS.md) - Implementation flows
- [API_REFERENCE.md](./API_REFERENCE.md) - Command reference
- [DEEP_LINKS.md](./DEEP_LINKS.md) - URL scheme handling