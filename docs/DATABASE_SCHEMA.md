# Pantheon Database Schema

## Overview

Complete SQL DDL for SQLite database with all tables, indexes, triggers, and migrations.

## Schema Version

Current schema version: **1**

## Migrations

Migrations are stored in `src-tauri/src/db/migrations/` as numbered SQL files.

### 001_initial_schema.sql

```sql
-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Games table
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    installPath TEXT NOT NULL,
    supportPath TEXT NOT NULL,
    launcher TEXT NOT NULL CHECK (launcher IN ('steam', 'gog', 'epic', 'xbox', 'origin', 'manual')),
    extensionId TEXT,
    supportedModTypes TEXT DEFAULT '[]',
    mergeMods INTEGER DEFAULT 0,
    details TEXT DEFAULT '{}',
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_games_launcher ON games(launcher);
CREATE INDEX idx_games_extension ON games(extensionId);

-- Mods table
CREATE TABLE mods (
    id TEXT PRIMARY KEY,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    version TEXT,
    category TEXT,
    modType TEXT NOT NULL CHECK (modType IN ('simple', 'fomod', 'foomad', 'bsat', 'bepinex', 'dazip', 'enb', 'scriptExtender', 'modPlugin')),
    installPath TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    flags TEXT DEFAULT '[]',
    attributes TEXT DEFAULT '{}',
    installTime TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    lastModified TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    metadata TEXT,
    conflicts TEXT DEFAULT '[]',
    dependencies TEXT DEFAULT '[]',
    UNIQUE(gameId, name)
);

CREATE INDEX idx_mods_game ON mods(gameId);
CREATE INDEX idx_mods_type ON mods(modType);
CREATE INDEX idx_mods_enabled ON mods(enabled);

-- Mod files table
CREATE TABLE modFiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT,
    isArchive INTEGER DEFAULT 0,
    UNIQUE(modId, path)
);

CREATE INDEX idx_modFiles_mod ON modFiles(modId);
CREATE INDEX idx_modFiles_hash ON modFiles(hash);

-- Deployment state table
CREATE TABLE deployment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL UNIQUE REFERENCES mods(id) ON DELETE CASCADE,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'deployed', 'partiallyDeployed', 'failed', 'conflict')),
    strategy TEXT NOT NULL CHECK (strategy IN ('symlink', 'hardlink', 'copy', 'merge')),
    deployedFiles TEXT DEFAULT '[]',
    conflicts TEXT DEFAULT '[]',
    deployedAt TEXT,
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_deployment_game ON deployment(gameId);
CREATE INDEX idx_deployment_status ON deployment(status);

-- Load order table
CREATE TABLE loadOrder (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    pluginName TEXT NOT NULL,
    loadOrderIndex INTEGER NOT NULL,
    enabled INTEGER DEFAULT 1,
    groupName TEXT,
    UNIQUE(gameId, pluginName)
);

CREATE INDEX idx_loadOrder_game ON loadOrder(gameId);
CREATE INDEX idx_loadOrder_index ON loadOrder(loadOrderIndex);

-- Plugins table (for detailed plugin info - Bethesda games)
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    filePath TEXT NOT NULL,
    isMaster INTEGER DEFAULT 0,
    isLight INTEGER DEFAULT 0,
    isMedium INTEGER DEFAULT 0,
    isDummy INTEGER DEFAULT 0,
    author TEXT,
    description TEXT,
    masterList TEXT DEFAULT '[]',
    revision INTEGER DEFAULT 0,
    loadOrderIndex INTEGER,
    hash TEXT,
    size INTEGER,
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(gameId, filePath)
);

CREATE INDEX idx_plugins_game ON plugins(gameId);
CREATE INDEX idx_plugins_loadOrder ON plugins(loadOrderIndex);

-- Plugin groups table
CREATE TABLE pluginGroups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    after TEXT DEFAULT '[]',
    description TEXT,
    UNIQUE(gameId, name)
);

-- Userlist rules table
CREATE TABLE userlistRules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    pluginName TEXT NOT NULL,
    after TEXT DEFAULT '[]',
    require TEXT DEFAULT '[]',
    warn TEXT DEFAULT '[]',
    hide INTEGER DEFAULT 0,
    groupName TEXT,
    UNIQUE(gameId, pluginName)
);

-- Download queue table
CREATE TABLE downloads (
    id TEXT PRIMARY KEY,
    fileName TEXT NOT NULL,
    url TEXT NOT NULL,
    destination TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'downloading', 'paused', 'completed', 'failed')),
    bytesWritten INTEGER DEFAULT 0,
    bytesTotal INTEGER,
    etag TEXT,
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    completedAt TEXT,
    error TEXT
);

CREATE INDEX idx_downloads_state ON downloads(state);

-- Profiles table
CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    modState TEXT DEFAULT '{}',
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(gameId, name)
);

CREATE INDEX idx_profiles_game ON profiles(gameId);

-- Settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Extensions registry table
CREATE TABLE extensions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    config TEXT DEFAULT '{}',
    installedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Backups table
CREATE TABLE backups (
    id TEXT PRIMARY KEY,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('full', 'saves', 'config', 'mods')),
    sourcePath TEXT NOT NULL,
    backupPath TEXT NOT NULL,
    size INTEGER,
    createdAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_backups_game ON backups(gameId);
CREATE INDEX idx_backups_type ON backups(type);

-- Schema migrations table
CREATE TABLE schemaMigrations (
    version INTEGER PRIMARY KEY,
    appliedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Insert initial migration record
INSERT INTO schemaMigrations (version) VALUES (1);

-- Triggers for updatedAt auto-update
CREATE TRIGGER games_updated_at AFTER UPDATE ON games
BEGIN
    UPDATE games SET updatedAt = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER mods_updated_at AFTER UPDATE ON mods
BEGIN
    UPDATE mods SET lastModified = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER deployment_updated_at AFTER UPDATE ON deployment
BEGIN
    UPDATE deployment SET updatedAt = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER plugins_updated_at AFTER UPDATE ON plugins
BEGIN
    UPDATE plugins SET updatedAt = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER profiles_updated_at AFTER UPDATE ON profiles
BEGIN
    UPDATE profiles SET updatedAt = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;
```

### 002_add_validation_tables.sql

```sql
-- Validation results table
CREATE TABLE validationResults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    isValid INTEGER NOT NULL,
    issues TEXT DEFAULT '[]',
    scannedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(modId)
);

CREATE INDEX idx_validationResults_mod ON validationResults(modId);

-- Mod updates table
CREATE TABLE modUpdates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modId TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    currentVersion TEXT NOT NULL,
    newVersion TEXT NOT NULL,
    downloadUrl TEXT,
    notifiedAt TEXT,
    UNIQUE(modId)
);

CREATE INDEX idx_modUpdates_mod ON modUpdates(modId);
```

### 003_add_masterlist_tables.sql

```sql
-- Masterlist table
CREATE TABLE masterlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gameId TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    globals TEXT DEFAULT '[]',
    plugins TEXT DEFAULT '[]',
    groups TEXT DEFAULT '[]',
    updatedAt TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(gameId)
);

CREATE INDEX idx_masterlist_game ON masterlist(gameId);
```

## Index Summary

| Table | Index Name | Columns | Purpose |
|-------|------------|---------|---------|
| games | idx_games_launcher | launcher | Find games by launcher |
| games | idx_games_extension | extensionId | Find games by extension |
| mods | idx_mods_game | gameId | Find mods by game |
| mods | idx_mods_type | modType | Find mods by type |
| mods | idx_mods_enabled | enabled | Filter enabled mods |
| modFiles | idx_modFiles_mod | modId | Find files by mod |
| modFiles | idx_modFiles_hash | hash | Find files by hash |
| deployment | idx_deployment_game | gameId | Find deployments by game |
| deployment | idx_deployment_status | status | Filter by status |
| loadOrder | idx_loadOrder_game | gameId | Find load order by game |
| loadOrder | idx_loadOrder_index | loadOrderIndex | Sort by order |
| plugins | idx_plugins_game | gameId | Find plugins by game |
| plugins | idx_plugins_loadOrder | loadOrderIndex | Sort plugins |
| pluginGroups | (unique) | (gameId, name) | Unique group per game |
| userlistRules | (unique) | (gameId, pluginName) | Unique rule per plugin |
| downloads | idx_downloads_state | state | Filter downloads |
| profiles | idx_profiles_game | gameId | Find profiles by game |
| backups | idx_backups_game | gameId | Find backups by game |
| backups | idx_backups_type | type | Filter by backup type |
| validationResults | idx_validationResults_mod | modId | Find validation by mod |
| modUpdates | idx_modUpdates_mod | modId | Find update by mod |
| masterlist | idx_masterlist_game | gameId | Find masterlist by game |

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
```json
{
  "steamAppId": 72850,
  "gogId": null,
  "epicAppId": null,
  "logo": "gameart.jpg",
  "requiredFiles": ["TESV.exe", "Skyrim.exe"],
  "environment": {
    "SteamAPPId": "72850"
  }
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