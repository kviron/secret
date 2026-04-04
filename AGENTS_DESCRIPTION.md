# Pantheon - Mod Manager

## Concept

**Pantheon** — десктопное приложение для управления модами в играх. Кроссплатформенный мод-менеджер с поддержкой сотен игр, гибкой системой расширений и фазовой установкой модов.

**Цель:** Создать современную, быструю и расширяемую альтернативу Pantheon с меньшим потреблением ресурсов.

---

## Tech Stack

| Layer | Technology | Reason |
|-------|------------|--------|
| Desktop Framework | Tauri 2.x | Small size (~10MB), Rust backend, native performance |
| UI Framework | Solid.js | Fine-grained reactivity, no VDOM, fast |
| Language (UI) | TypeScript | Type safety |
| Styling | Panda CSS | CSS-in-JS with theme modes, design tokens, dark/light themes |
| Design System | ui-design-system skill | Three-tier token system (OKLCH colors) |
| Language (Backend) | Rust | Memory safety, speed, native sys operations |
| Database | SQLite (rusqlite) | Simple, embedded, cross-platform |
| File Operations | Rust std + crates | zip, 7z, symlinks |

### Frontend Styling (Panda CSS)

- **Panda CSS** for CSS-in-JS with compile-time generation
- **Theme modes**: Light/Dark themes via CSS variables
- **Design tokens**: Follow `ui-design-system` skill guidelines (3-tier: primitives → semantics → components)
- **Color space**: OKLCH for perceptual uniformity
- **CSS preset**: `@pandaprism/core` with custom theme overrides

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                     Solid.js Frontend                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │   Views/    │  │  Components │  │   Stores/       │  │
│  │   Pages     │  │             │  │   Signals       │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└────────────────────────┬─────────────────────────────────┘
                         │ invoke() / events
┌────────────────────────▼─────────────────────────────────┐
│                     Tauri Core (Rust)                     │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Command Handlers                       │  │
│  │  games/  mods/  install/  deploy/  settings/        │  │
│  └─────────────────────────────────────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │  Game    │ │   Mod    │ │  Deploy  │ │    DB      │  │
│  │ Detector │ │ Installer│ │  Manager │ │  Manager   │  │
│  └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
│  ┌──────────────────────────────────────────────────────┐│
│  │              Extension System (Plugins)              ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

---

## Modules

Detailed documentation for each module:

### Core Modules
- [Game Detector](docs/modules/game-detector.md) — Обнаружение установленных игр
- [Mod Installer](docs/modules/mod-installer.md) — Установка модов
- [Deploy Manager](docs/modules/deploy-manager.md) — Развёртывание модов
- [Download Manager](docs/modules/download-manager.md) — Загрузки с паузой/возобновлением
- [Database Manager](docs/modules/database-manager.md) — SQLite persistence
- [Extension System](docs/modules/extension-system.md) — Плагинная система
- [Load Order Manager](docs/modules/load-order-manager.md) — Порядок загрузки плагинов
- [UI Structure](docs/modules/ui-structure.md) — Solid.js + FSD v2.1 architecture

### Added Modules (Post-Audit)
- [Profile Manager](docs/modules/profile-manager.md) — Управление профилями модов
- [Game Launcher](docs/modules/game-launcher.md) — Запуск игр с мод-лоадерами
- [Security & Validation](docs/modules/security-validation.md) — Безопасность и валидация модов
- [Mod Repository API](docs/modules/mod-repository-api.md) — Интеграция с репозиторием модов
- [Update Checker](docs/modules/update-checker.md) — Проверка обновлений модов
- [Backup & Restore](docs/modules/backup-restore.md) — Бэкапы и восстановление
- [Dependency Resolution](docs/modules/dependency-resolution.md) — Разрешение зависимостей

### Architecture Documents
- [Architecture Overview](docs/ARCHITECTURE.md) — System design, data flow, IPC channels
- [Module Connections](docs/MODULE_CONNECTIONS.md) — How modules interact
- [Vortex Audit](docs/VORTEX_AUDIT.md) — Feature comparison with Vortex
- [Cross-Platform Support](docs/CROSS_PLATFORM.md) — Linux/SteamOS architecture

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-4)

**Goal:** Minimal working app that can detect games and install mods.

- [ ] Tauri + Solid.js setup
- [ ] Basic UI shell (layout, routing)
- [ ] SQLite database setup
- [ ] Game detector (Steam only)
- [ ] Basic mod installer (zip extraction)
- [ ] Simple deploy (copy files)

### Phase 2: Core Features (Weeks 5-8)

**Goal:** Full mod management with enable/disable, safety, and game launching.

- [ ] Enable/Disable mods
- [ ] Deployment with symlinks
- [ ] File conflict detection
- [ ] Mod metadata display
- [ ] Basic load order (drag-drop)
- [ ] **Game Launcher** (mod loaders, script extenders)
- [ ] **Security & Validation** (file scanning, save protection)
- [ ] **Backup & Restore** (game files, saves, configs)

### Phase 3: Extension System (Weeks 9-12)

**Goal:** Plugin architecture for game support.

- [ ] Extension loading system
- [ ] Game extension template
- [ ] Skyrim extension (full)
- [ ] Fallout 4 extension
- [ ] FOMOD installer support

### Phase 4: Advanced Features (Weeks 13-16)

**Goal:** Profiles, dependency resolution, repository integration, auto-sorting.

- [ ] **Profile Manager** (multiple configurations per game)
- [ ] **Dependency Resolution** (graph-based conflict detection)
- [ ] **Mod Repository API** (browse, download, manage mods)
- [ ] **Update Checker** (mod version tracking, notifications)
- [ ] Load order auto-sort (LOOT integration)
- [ ] Download manager
- [ ] Settings persistence

### Phase 5: Polish & Release (Weeks 17-20)

**Goal:** Bug fixes, performance, release.

- [ ] Performance optimization
- [ ] Error handling refinement
- [ ] Cross-platform (Windows primary, Linux later)
- [ ] Documentation
- [ ] Release builds

---

## Key Dependencies (Rust)

```toml
# Cargo.toml - Tauri app
[dependencies]
tauri = { version = "2", features = ["devtools"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
zip = "0.6"
sevenz-rust = "0.5"
winreg = "0.52"  # Windows registry
log = "0.4"
env_logger = "0.11"
thiserror = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## Frontend File Structure (FSD)

```
pantheon/src/
├── app/                     # App initialization
│   ├── App.tsx
│   ├── index.tsx
│   ├── providers/
│   │   └── ToastProvider.tsx
│   └── router/
│       └── index.tsx
├── pages/                   # Route-level composition
│   ├── dashboard/
│   │   ├── index.ts
│   │   └── ui/
│   │       ├── Dashboard.tsx
│   │       ├── RecentMods.tsx
│   │       └── QuickActions.tsx
│   ├── games/
│   │   ├── index.ts
│   │   └── ui/
│   │       ├── GamesList.tsx
│   │       └── GameCard.tsx
│   ├── game-detail/
│   │   ├── index.ts
│   │   ├── model/
│   │   │   └── store.ts
│   │   └── ui/
│   │       ├── GameDetail.tsx
│   │       ├── ModList.tsx
│   │       ├── ModItem.tsx
│   │       └── LoadOrderTab.tsx
│   ├── settings/
│   │   ├── index.ts
│   │   └── ui/
│   │       ├── Settings.tsx
│   │       └── ThemeSettings.tsx
│   └── downloads/
│       ├── index.ts
│       └── ui/
│           └── Downloads.tsx
├── widgets/                 # Reusable composite UI blocks
│   ├── GameCard/
│   │   ├── index.ts
│   │   └── ui/
│   │       └── GameCard.tsx
│   ├── ModList/
│   │   ├── index.ts
│   │   └── ui/
│   │       ├── ModList.tsx
│   │       └── ModItem.tsx
│   ├── LoadOrderEditor/
│   │   ├── index.ts
│   │   └── ui/
│   │       └── LoadOrderEditor.tsx
│   └── DownloadQueue/
│       ├── index.ts
│       └── ui/
│           └── DownloadQueue.tsx
├── features/                # Reusable user interactions
│   ├── install-mod/
│   │   ├── index.ts
│   │   └── ui/
│   │       ├── InstallModal.tsx
│   │       └── InstallWizard.tsx
│   ├── toggle-mod/
│   │   ├── index.ts
│   │   └── ui/
│   │       └── ToggleMod.tsx
│   ├── resolve-conflict/
│   │   ├── index.ts
│   │   └── ui/
│   │       └── ConflictDialog.tsx
│   └── game-detection/
│       ├── index.ts
│       └── lib/
│           └── detect-games.ts
├── entities/                # Business domain models
│   ├── game/
│   │   ├── index.ts
│   │   ├── model/
│   │   │   └── game.ts
│   │   └── api/
│   │       └── games.ts
│   ├── mod/
│   │   ├── index.ts
│   │   ├── model/
│   │   │   └── mod.ts
│   │   └── api/
│   │       └── mods.ts
│   └── deployment/
│       ├── index.ts
│       ├── model/
│       │   └── deployment.ts
│       └── api/
│           └── deploy.ts
└── shared/                  # Infrastructure
    ├── ui/
    │   ├── Button/
    │   ├── Input/
    │   ├── Modal/
    │   ├── Card/
    │   ├── Spinner/
    │   └── Toast/
    ├── api/
    │   ├── client.ts
    │   ├── games.ts
    │   ├── mods.ts
    │   └── deploy.ts
    ├── lib/
    │   ├── format-date.ts
    │   ├── debounce.ts
    │   └── classnames.ts
    └── config/
        └── routes.ts
```

---

## Configuration

### tauri.conf.json

```json
{
    "productName": "Pantheon",
    "version": "0.1.0",
    "identifier": "com.pantheon.modmanager",
    "build": {
        "devtools": true
    },
    "app": {
        "windows": [
            {
                "title": "Pantheon",
                "width": 1200,
                "height": 800,
                "minWidth": 900,
                "minHeight": 600,
                "resizable": true,
                "fullscreen": false
            }
        ]
    }
}
```

---

## Error Handling

### Error Types

```rust
enum PantheonError {
    GameNotFound(String),
    ModAlreadyInstalled(String),
    DeploymentFailed(String),
    ConflictDetected(Vec<Conflict>),
    ExtensionError(String),
    DatabaseError(String),
    IoError(String),
}
```

### Frontend Error Display

```typescript
// Global error boundary
const [error, setError] = createSignal<Error | null>(null);

// Show toast on error
createEffect(() => {
    if (error()) {
        showToast({
            type: 'error',
            message: error().message,
            duration: 5000
        });
    }
});
```

---

## Testing Strategy

| Level | Tool | Coverage |
|-------|------|----------|
| Unit (Rust) | `cargo test` | Core logic, installers |
| Integration (Rust) | `cargo test` + fixtures | DB, file ops |
| Unit (JS) | Vitest | Stores, components |
| E2E | Playwright | Full user flows |

---

## Performance Considerations

1. **Large mod lists** - Virtual scrolling (`solid-virtual`)
2. **File operations** - Async/await, parallel extraction
3. **Database** - Prepared statements, indexes
4. **UI updates** - Fine-grained reactivity (Solid advantage)
5. **Startup time** - Lazy extension loading

---

## Future Enhancements

- [ ] Mod Repository API integration
- [ ] Mod download browser
- [ ] Mod profiles (different mod configurations)
- [ ] Cloud sync (settings, load orders)
- [ ] Community features (share collections)
- [ ] Linux/macOS support
