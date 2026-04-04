# Feature Audit

## Purpose

This document maps common mod manager features to Pantheon modules, identifying gaps and implementation priorities.

---

## Core Modules Comparison

### Mod Management

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Mod Installation | ✅ Planned | `mod-installer` | Archive extraction, FOMOD support |
| Mod Staging | ✅ Planned | `mod-installer` + `deploy-manager` | VFS with staging area |
| Mod Enable/Disable | ✅ Planned | `deploy-manager` | Symlink/hardlink management |
| Mod Categories | ✅ Planned | `database` | Category per mod |
| Mod Version Tracking | ✅ Planned | `database` | Version field |
| Mod Metadata | ✅ Planned | `database` | JSON metadata |
| Mod Conflicts | ✅ Planned | `deploy-manager` | Conflict detection |
| Mod Dependencies | ✅ Planned | `dependency-resolution` | Graph-based resolution |
| Mod Sources | ✅ Planned | `mod-repository-api` | Pantheon repository, external sources |

### Download Management

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Download Queue | ✅ Planned | `download-manager` | Queue with concurrency |
| Pause/Resume | ✅ Planned | `download-manager` | Chunked downloads |
| URL Resolution | ✅ Planned | `mod-repository-api` | Pantheon repository CDN |
| File Validation | ✅ Planned | `security-validation` | SHA256 hash check |
| Download Progress | ✅ Planned | `download-manager` | Progress events |

### Game Detection

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Steam Detection | ✅ Planned | `game-detector` | Registry + API |
| GOG Detection | ✅ Planned | `game-detector` | Registry + Galaxy |
| Epic Detection | ✅ Planned | `game-detector` | Manifest files |
| Xbox Detection | ⚠️ Future | `game-detector` | Windows registry |
| Manual Path | ✅ Planned | `game-detector` | User-specified |

### Deployment

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Symlink Deploy | ✅ Planned | `deploy-manager` | Default on Windows |
| Hardlink Deploy | ✅ Planned | `deploy-manager` | Fallback |
| Copy Deploy | ✅ Planned | `deploy-manager` | FS fallback |
| Merge Strategy | ⚠️ Future | `deploy-manager` | Manual conflict resolve |
| BSA Invalidation | ⚠️ Future | `deploy-manager` | Bethesda games |

### Load Order

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Plugin List | ✅ Planned | `load-order-manager` | ESP/ESM/ESL |
| Load Order Persist | ✅ Planned | `database` | load_order table |
| Plugin Enable/Disable | ✅ Planned | `load-order-manager` | plugins.txt |
| Auto-sort (LOOT) | ⚠️ Future | `load-order-manager` | LOOT metadata |
| Groups | ⚠️ Future | `load-order-manager` | LOOT groups |
| Userlist | ⚠️ Future | `load-order-manager` | Managed plugins |

### Extension System

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Game Extensions | ✅ Planned | `extension-system` | Trait-based |
| Mod Type Extensions | ✅ Planned | `extension-system` | Handler traits |
| Installer Extensions | ✅ Planned | `extension-system` | Custom installers |
| Settings Registration | ⚠️ Partial | `extension-system` | Via pages |
| UI Registration | ✅ Planned | `extension-system` | Pages/widgets |

### State Management

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Persistent State | ✅ Planned | `database` | SQLite |
| Session State | ✅ Planned | Solid.js stores | In-memory |
| Reducers | ✅ Planned | Solid.js stores | Native pattern |
| State Hydration | ✅ Planned | `database` | Load on startup |
| State Diff Sync | ⚠️ Future | Tauri IPC | Event-based |

### UI/Views

| Feature | Status | Pantheon Module | Notes |
|---------|--------|-----------------|-------|
| Dashboard | ✅ Planned | `pages/dashboard` | Game overview |
| Game List | ✅ Planned | `pages/games` | All games |
| Game Detail | ✅ Planned | `pages/game-detail` | Mod list, LO |
| Settings | ✅ Planned | `pages/settings` | Preferences |
| Downloads | ✅ Planned | `pages/downloads` | Queue |
| Mod Browser | ✅ Planned | `pages/mod-browser` | Repository browser |
| Mod List Table | ✅ Planned | `widgets/ModList` | Virtual scroll |
| Load Order Editor | ✅ Planned | `widgets/LoadOrderEditor` | Drag-drop |

---

## Priority Implementation Order

### Phase 1: Core (MVP)

1. **Game Detection** (`game-detector`)
   - Steam registry scan
   - Basic game model
   - Database storage

2. **Mod Installation** (`mod-installer`)
   - Archive extraction (zip)
   - Staging area
   - Database records

3. **Basic Deployment** (`deploy-manager`)
   - Copy deployment (fallback)
   - Mod enable/disable
   - Database state

4. **UI Shell** (`app/`, `pages/`)
   - Solid.js + Panda CSS setup
   - Routing
   - Basic components

### Phase 2: Full Features

5. **Symlink Deployment**
   - Windows symlink support
   - Conflict detection
   - Hardlink fallback

6. **Download Manager** (`download-manager`)
   - Queue management
   - Pause/resume
   - Progress UI

7. **Load Order** (`load-order-manager`)
   - Plugin list
   - Drag-drop reorder
   - Persistence

8. **Extension System** (`extension-system`)
   - Game extensions
   - Mod type handlers
   - Installer registry

### Phase 3: Advanced

9. **FOMOD Support**
   - XML parsing
   - Option UI
   - Conditional install

10. **LOOT Integration**
    - Metadata loading
    - Auto-sort
    - Groups

11. **BSA/BA2 Support**
    - Archive extraction
    - Archive invalidation

---

## Feature Gaps

Features from existing mod managers that Pantheon should implement:

| Feature | Description | Priority |
|---------|-------------|----------|
| Profiles | Multiple mod configurations per game | High |
| Mod Repository | Official mod hosting with CDN | High |
| Update Checker | Mod version monitoring | Medium |
| Collections | Share mod configurations | Low (post-MVP) |
| Cloud Sync | Sync across devices | Low |

---

## New Modules Added (Post-Audit)

Based on best practices analysis of existing mod managers (Vortex, Mod Organizer 2) and modern application architecture, the following modules have been added to the documentation:

### Profile Manager
- [Profile Manager](modules/profile-manager.md) — Multiple mod configurations per game
- **Why**: Essential for users with different mod setups per playthrough
- **Status**: ✅ Documented, planned for Phase 4

### Game Launcher
- [Game Launcher](modules/game-launcher.md) — Launch games with mod loaders, script extenders
- **Why**: Critical for modded games — must handle SKSE, F4SE, proxy DLLs, launch args
- **Status**: ✅ Documented, planned for Phase 2

### Security & Validation
- [Security & Validation](modules/security-validation.md) — Malware scanning, file validation, save protection
- **Why**: Third-party mods are untrusted input; security-first approach is essential
- **Status**: ✅ Documented, planned for Phase 2

### Mod Repository API
- [Mod Repository API](modules/mod-repository-api.md) — Browse, download, manage mods from Pantheon repository
- **Why**: Official mod source with CDN, search, and update tracking
- **Status**: ✅ Documented, planned for Phase 4

### Update Checker
- [Update Checker](modules/update-checker.md) — Automatic mod update detection
- **Why**: Users need to know when mods have updates; version pinning support
- **Status**: ✅ Documented, planned for Phase 4

### Backup & Restore
- [Backup & Restore](modules/backup-restore.md) — Game files, saves, config backups
- **Why**: Essential safety net; prevents data loss from bad mod installations
- **Status**: ✅ Documented, planned for Phase 2

### Dependency Resolution
- [Dependency Resolution](modules/dependency-resolution.md) — Dependency graph, conflict detection, resolution
- **Why**: Complex mod setups require automated dependency management
- **Status**: ✅ Documented, planned for Phase 4

### Cross-Platform Support
- [Cross-Platform](CROSS_PLATFORM.md) — Linux/SteamOS support architecture
- **Why**: SteamOS growing; community expects Linux support
- **Status**: ✅ Documented, planned for Phase 3

---

## Updated Implementation Phases

### Phase 1: Core (MVP)
1. Game Detection
2. Mod Installation (basic)
3. Basic Deployment (copy)
4. UI Shell

### Phase 2: Core Features — Extended
5. Symlink Deployment
6. **Game Launcher** (mod loaders, script extenders)
7. **Security & Validation** (file scanning, save protection)
8. **Backup & Restore** (game files, saves)
9. Download Manager
10. Load Order (drag-drop)

### Phase 3: Extension System + Cross-Platform
11. Extension loading system
12. Game extension template
13. Skyrim extension
14. FOMOD support
15. **Cross-Platform** (Linux/SteamOS)

### Phase 4: Advanced Features — Extended
16. **Profile Manager** (multiple configurations)
17. **Dependency Resolution** (graph, conflict detection)
18. **Mod Repository API** (browse, download, updates)
19. **Update Checker** (mod version tracking)
20. LOOT Integration
21. Mod collections

---

## Summary

**Implementation Status:**
- ✅ Planned: ~80% of core mod manager features
- ⚠️ Future: ~15% of features (post-MVP)
- ❌ Not planned: ~5% (privacy/scope reasons)

**Key Differences from existing managers:**
- Solid.js instead of React (performance)
- SQLite instead of LevelDB+DuckDB (simplicity)
- Rust traits instead of JS extensions (type safety)
- Panda CSS instead of LESS (modern tooling)
- Own repository instead of third-party APIs (control, reliability)
- Security-first approach (malware scanning, save protection)
