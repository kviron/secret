# Pantheon Database Schema

## Overview

Complete SQL DDL for SQLite database with all tables, indexes, triggers, and migrations.

## Schema Version

Current schema version: **3** (3 migrations applied)

## Actually Implemented Migrations

Migrations are stored in `src-tauri/src/db/migrations/` as numbered SQL files.

### 001_initial_schema.sql (Implemented)

```sql
PRAGMA foreign_keys = ON;

-- Games table
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    install_path TEXT NOT NULL,
    support_path TEXT NOT NULL,
    launcher TEXT NOT NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_games_launcher ON games(launcher);

-- Mods table
CREATE TABLE IF NOT EXISTS mods (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    version TEXT,
    category TEXT,
    mod_type TEXT NOT NULL DEFAULT 'simple',
    install_path TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    flags TEXT DEFAULT '[]',
    attributes TEXT DEFAULT '{}',
    install_time TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_modified TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    metadata TEXT,
    conflicts TEXT DEFAULT '[]',
    dependencies TEXT DEFAULT '[]',
    UNIQUE(game_id, name)
);

CREATE INDEX IF NOT EXISTS idx_mods_game ON mods(game_id);
CREATE INDEX IF NOT EXISTS idx_mods_type ON mods(mod_type);
CREATE INDEX IF NOT EXISTS idx_mods_enabled ON mods(enabled);

-- Mod files table
CREATE TABLE IF NOT EXISTS mod_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT,
    is_archive INTEGER DEFAULT 0,
    UNIQUE(mod_id, path)
);

CREATE INDEX IF NOT EXISTS idx_mod_files_mod ON mod_files(mod_id);
CREATE INDEX IF NOT EXISTS idx_mod_files_hash ON mod_files(hash);

-- Deployment state table
CREATE TABLE IF NOT EXISTS deployment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id TEXT NOT NULL UNIQUE REFERENCES mods(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    strategy TEXT NOT NULL DEFAULT 'symlink',
    deployed_files TEXT DEFAULT '[]',
    conflicts TEXT DEFAULT '[]',
    deployed_at TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_deployment_game ON deployment(game_id);
CREATE INDEX IF NOT EXISTS idx_deployment_status ON deployment(status);

-- Schema migrations table
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);

-- Triggers
CREATE TRIGGER IF NOT EXISTS games_updated_at AFTER UPDATE ON games
BEGIN
    UPDATE games SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS mods_updated_at AFTER UPDATE ON mods
BEGIN
    UPDATE mods SET last_modified = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS deployment_updated_at AFTER UPDATE ON deployment
BEGIN
    UPDATE deployment SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;
```

**Note:** Column `extension_id`, `supported_mod_types`, `merge_mods`, `details`, `mod_support` are added dynamically via `add_missing_columns()` in `db::migrate()` if absent.

### 002_downloads.sql (Implemented)

```sql
CREATE TABLE IF NOT EXISTS downloads (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    destination TEXT NOT NULL,
    game_id TEXT,
    total_bytes INTEGER DEFAULT 0,
    downloaded_bytes INTEGER DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'pending',
    error TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_downloads_state ON downloads(state);
CREATE INDEX IF NOT EXISTS idx_downloads_game ON downloads(game_id);

CREATE TRIGGER IF NOT EXISTS downloads_updated_at AFTER UPDATE ON downloads
BEGIN
    UPDATE downloads SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;
```

### 003_load_order.sql (Implemented)

```sql
CREATE TABLE IF NOT EXISTS load_order (
    game_id TEXT NOT NULL,
    plugin_name TEXT NOT NULL,
    load_order_index INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    plugin_type TEXT NOT NULL DEFAULT 'esp',
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (game_id, plugin_name)
);

CREATE INDEX IF NOT EXISTS idx_load_order_game ON load_order(game_id);
CREATE INDEX IF NOT EXISTS idx_load_order_index ON load_order(game_id, load_order_index);

CREATE TRIGGER IF NOT EXISTS load_order_updated_at AFTER UPDATE ON load_order
BEGIN
    UPDATE load_order SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE game_id = NEW.game_id AND plugin_name = NEW.plugin_name;
END;
```

## Planned Tables (Future Phases)

### 004_plugins.sql (Phase 3)

```sql
-- Plugins table (for detailed plugin info - Bethesda games)
CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    is_master INTEGER DEFAULT 0,
    is_light INTEGER DEFAULT 0,
    is_medium INTEGER DEFAULT 0,
    is_dummy INTEGER DEFAULT 0,
    author TEXT,
    description TEXT,
    master_list TEXT DEFAULT '[]',
    revision INTEGER DEFAULT 0,
    load_order_index INTEGER,
    hash TEXT,
    size INTEGER,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(game_id, file_path)
);

CREATE INDEX IF NOT EXISTS idx_plugins_game ON plugins(game_id);
CREATE INDEX IF NOT EXISTS idx_plugins_loadOrder ON plugins(load_order_index);
```

### 005_profiles.sql (Phase 4)

```sql
CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    mod_state TEXT DEFAULT '{}',
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(game_id, name)
);

CREATE INDEX IF NOT EXISTS idx_profiles_game ON profiles(game_id);
```

### 006_backups.sql (Phase 4)

```sql
CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    source_path TEXT NOT NULL,
    backup_path TEXT NOT NULL,
    size INTEGER,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_backups_game ON backups(game_id);
```

### 007_extensions.sql (Phase 3)

```sql
CREATE TABLE IF NOT EXISTS extensions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    config TEXT DEFAULT '{}',
    installed_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

### 008_masterlist.sql (Phase 3)

```sql
CREATE TABLE IF NOT EXISTS masterlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    globals TEXT DEFAULT '[]',
    plugins TEXT DEFAULT '[]',
    groups TEXT DEFAULT '[]',
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(game_id)
);

CREATE INDEX IF NOT EXISTS idx_masterlist_game ON masterlist(game_id);
```

## Index Summary (Implemented)

| Table | Index Name | Columns | Purpose |
|-------|------------|---------|---------|
| games | idx_games_launcher | launcher | Find games by launcher |
| mods | idx_mods_game | game_id | Find mods by game |
| mods | idx_mods_type | mod_type | Find mods by type |
| mods | idx_mods_enabled | enabled | Filter enabled mods |
| mod_files | idx_mod_files_mod | mod_id | Find files by mod |
| mod_files | idx_mod_files_hash | hash | Find files by hash |
| deployment | idx_deployment_game | game_id | Find deployments by game |
| deployment | idx_deployment_status | status | Filter by status |
| downloads | idx_downloads_state | state | Filter downloads |
| downloads | idx_downloads_game | game_id | Filter downloads by game |
| load_order | idx_load_order_game | game_id | Find load order by game |
| load_order | idx_load_order_index | (game_id, load_order_index) | Sort by order |

## Planned Index Summary (Future)

| Table | Index Name | Columns | Purpose |
|-------|------------|---------|---------|
| plugins | idx_plugins_game | game_id | Find plugins by game |
| profiles | idx_profiles_game | game_id | Find profiles by game |
| backups | idx_backups_game | game_id | Find backups by game |
| masterlist | idx_masterlist_game | game_id | Find masterlist by game |

## JSON Column Formats

### supportedModTypes (games table)
```json
["simple", "fomod", "scriptExtender", "modPlugin"]
```

### flags (mods table)
```json
["installed", "upgradeable", "missing"]
```

### attributes (mods table)
```json
{
  "author": "Mod Author",
  "website": "https://mod-site.com",
  "fileId": "12345",
  "modId": "67890"
}
```

### metadata (mods table)
```json
{
  "author": "Mod Author",
  "description": "Mod description text",
  "website": "https://mod-site.com",
  "installerVersion": "1.0.0",
  "installationFiles": [
    { "fileType": "replace", "source": "texture.dds", "destination": "Data/textures/mod/texture.dds" }
  ],
  "screenshot": "https://mod-site.com/screenshot.jpg",
  "category": "Textures"
}
```

### conflicts (mods table)
```json
["mod-id-1", "mod-id-2"]
```

### dependencies (mods table)
```json
["mod-id-required-1", "mod-id-required-2"]
```

### deployedFiles (deployment table)
```json
[
  {
    "source": "Data/textures/mod/texture.dds",
    "target": "Data/textures/mod/texture.dds",
    "linkType": "symlink",
    "size": 102400,
    "hash": "sha256hash..."
  }
]
```

### conflicts (deployment table)
```json
[
  {
    "type": "FileConflict",
    "modA": "mod-id-1",
    "modB": "mod-id-2",
    "file": "Data/scripts/mod.esp",
    "sizeA": 1024,
    "sizeB": 2048
  }
]
```

### modState (profiles table)
```json
{
  "mod-id-1": { "enabled": true, "customFileOverrides": [] },
  "mod-id-2": { "enabled": false, "customFileOverrides": ["Data/meshes/mod.nif"] }
}
```

### details (games table)

JSON object persisted as text. **New writes** follow the same shape as the TypeScript `GameDetails` type (**camelCase** keys), matching `serde` output from `src-tauri/src/models.rs`. Older rows may still contain **snake_case** keys (e.g. `steam_app_id`); deserialization uses field aliases so both forms load correctly.

`logo` may be `null`, a relative asset name, or an **absolute `https` URL** used by the Games Library card when set (overrides the default Steam header image).

```json
{
  "steamAppId": 72850,
  "gogId": null,
  "epicAppId": null,
  "logo": null,
  "requiredFiles": ["TESV.exe", "Skyrim.exe"],
  "environment": {}
}
```

### issues (validationResults table)
```json
[
  { "severity": "warning", "code": "UNSIGNED_PLUGIN", "message": "Plugin is not signed", "filePath": "Data/plugins/mod.esp" }
]
```

## Query Patterns

### Get all games with mod counts
```sql
SELECT g.*, COUNT(m.id) as modCount
FROM games g
LEFT JOIN mods m ON m.gameId = g.id
GROUP BY g.id;
```

### Get enabled mods for a game
```sql
SELECT * FROM mods
WHERE gameId = ? AND enabled = 1
ORDER BY name;
```

### Get deployment state with mod info
```sql
SELECT d.*, m.name as modName, m.modType
FROM deployment d
JOIN mods m ON m.id = d.modId
WHERE d.gameId = ?;
```

### Get load order for a game
```sql
SELECT * FROM loadOrder
WHERE gameId = ?
ORDER BY loadOrderIndex;
```

### Find conflicting files
```sql
SELECT mf1.modId as modA, mf2.modId as modB, mf1.path, mf1.size as sizeA, mf2.size as sizeB
FROM modFiles mf1
JOIN modFiles mf2 ON mf1.path = mf2.path AND mf1.modId < mf2.modId
WHERE mf1.modId IN (SELECT id FROM mods WHERE gameId = ?);
```

### Get profile with enabled mods
```sql
SELECT p.*, m.name, m.modType
FROM profiles p
JOIN mods m ON m.id IN (SELECT json_each.value FROM json_each(p.modState) WHERE json_each.value->>'enabled' = 'true')
WHERE p.gameId = ?;
```

## Performance Considerations

1. **Use prepared statements** - All queries should use prepared statements
2. **Index foreign keys** - All foreign key columns have indexes
3. **JSON for flexible data** - Use JSON columns for variable attributes
4. **Batch inserts** - Use transactions for bulk inserts
5. **Avoid SELECT *** - Always specify needed columns
6. **Use LIMIT** - Paginate large result sets

## Maintenance

### Vacuum (reclaim space)
```sql
PRAGMA vacuum;
```

### Analyze (update query planner)
```sql
PRAGMA analyze;
```

### Check integrity
```sql
PRAGMA integrity_check;
```