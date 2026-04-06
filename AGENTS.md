# AGENTS.md - Pantheon Development Guide

## Tech Stack
Tauri 2.x + Solid.js/TypeScript + Rust + SQLite(rusqlite) + Ark UI + Panda CSS | Architecture: FSD v2.1 | Dark gaming theme

## Docs (read as needed)
- `docs/AI-OPTIMIZED.md` — nav index, reading order
- `docs/ARCHITECTURE.md` — layers, modules, data flow
- `docs/MODELS.md` — Rust+TS types, IPC naming
- `docs/DATABASE_SCHEMA.md` — SQL DDL, migrations
- `docs/MODULE_SPECS.md` — module APIs
- `docs/FLOWS.md` — user journeys
- `docs/API_REFERENCE.md` — Tauri commands/events
- `docs/modules/*.md` — per-module deep dives

## Build Commands
```bash
# Rust
cargo tauri dev          # dev mode
cargo tauri build        # production
cargo test               # tests
cargo fmt && cargo clippy -- -D warnings  # format + lint

# Frontend
npm run dev / npm run build / npm test
npm run typecheck / npm run lint
```

## FSD Layers (top-down import only)
```
app/ → pages/ → widgets/ → features/ → entities/ → shared/
```
- Each slice exports via `index.ts`
- `shared` imports from nothing; `pages` imports all below

## Code Style

**TS/Solid.js:** `@/` absolute imports, `type` for aliases/`interface` for shapes, `unknown` not `any`, `undefined` not `null`, `splitProps` for props, `createSignal`/`createStore`/`createMemo`, handlers prefixed `handle`

**Rust:** `snake_case` modules/fns/vars, `PascalCase` structs/enums, `SCREAMING_SNAKE_CASE` consts, `thiserror` + `?` operator, `Result<T, String>` from commands

**Naming:** TS types=`PascalCase`, fns=`camelCase`, files=`camelCase|kebab-case` | Rust see above | DB tables/cols=`snake_case`

## Ark UI Pattern
```tsx
<Switch.Root checked={val()} onCheckedChange={(e) => setVal(e.checked)}>
  <Switch.Control><Switch.Thumb /></Switch.Control>
  <Switch.Label>Label</Switch.Label>
</Switch.Root>
```

## File Structure
```
src/
├── app/        # init, providers, routing
├── pages/      # route composition
├── widgets/    # reusable composites
├── features/   # user interactions
├── entities/   # domain models + api
└── shared/     # ui kit, api client, utils, config
src-tauri/src/
├── commands/   # #[tauri::command] per module
├── services/   # business logic
├── db/         # migrations, queries
└── models.rs   # types
```

## Key Rules
- Commands: `src-tauri/src/commands/`, `#[tauri::command]`, return `Result<T, String>`
- DB: `rusqlite` bundled, migrations in `db/migrations/`
- Tests: Rust `#[cfg(test)]` co-located, Vitest `*.test.ts` next to source
- Git: Conventional Commits (`feat:`, `fix:`, `docs:`)
- **При изменении кода — обновляй соответствующую документацию в `docs/`**
