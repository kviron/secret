# Module: Database Manager

## Responsibility

Persistence of all application data: games, mods, settings, deployment state, user preferences.

## Pantheon Comparison

Pantheon uses a combination of:
- **LevelDB**: Low-level key-value storage for Redux state persistence
- **DuckDB**: SQL database for reactive queries and complex data relationships
- **Custom Persistors**: Extension-specific persistence (plugins.txt, loadorder.txt)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Persistence Architecture                    │
└─────────────────────────────────────────────────────────────┘

    ┌──────────────────────────────────────────────────────┐
    │                  Pantheon Database                     │
    │                                                       │
    │   ┌─────────────────┐    ┌─────────────────────┐    │
    │   │    SQLite       │    │     JSON Files      │    │
    │   │   (rusqlite)    │    │   (Config/Settings)│    │
    │   │                 │    │                     │    │
    │   │  • Games        │    │  • App settings    │    │
    │   │  • Mods         │    │  • Window state   │    │
    │   │  • Deployments  │    │  • User prefs     │    │
    │   │  • Download Q   │    │  • Extension data │    │
    │   │  • Load Order   │    │                     │    │
    │   └─────────────────┘    └─────────────────────┘    │
    │                                                       │
    │   ┌─────────────────────────────────────────────┐   │
    │   │              r2d2 Connection Pool            │   │
    │   │         (Prepared Statements)                │   │
    │   └─────────────────────────────────────────────┘   │
    └──────────────────────────────────────────────────────┘
```

## SQLite Schema

### Games Table

```sql
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    install_path TEXT NOT NULL,
    support_path TEXT NOT NULL,
    launcher TEXT NOT NULL,
    extension_id TEXT,
    supported_mod_types TEXT,  -- JSON array
    merge_mods INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_games_launcher ON games(launcher);
CREATE INDEX idx_games_extension ON games(extension_id);
```

### Mods Table

```sql
CREATE TABLE mods (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id),
    name TEXT NOT NULL,
    version TEXT,
    category TEXT,
    mod_type TEXT NOT NULL,
    install_path TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    flags TEXT DEFAULT '[]',           -- JSON array
    attributes TEXT DEFAULT '{}',       -- JSON object
    install_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT,                      -- JSON object
    conflicts TEXT DEFAULT '[]',         -- JSON array of mod IDs
    dependencies TEXT DEFAULT '[]',      -- JSON array of mod IDs
    UNIQUE(game_id, name)
);

CREATE INDEX idx_mods_game ON mods(game_id);
CREATE INDEX idx_mods_type ON mods(mod_type);
CREATE INDEX idx_mods_enabled ON mods(enabled);
```

### Mod Files Table

```sql
CREATE TABLE mod_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id TEXT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT,
    is_archive INTEGER DEFAULT 0,
    UNIQUE(mod_id, path)
);

CREATE INDEX idx_mod_files_mod ON mod_files(mod_id);
CREATE INDEX idx_mod_files_hash ON mod_files(hash);
```

### Deployment State Table

```sql
CREATE TABLE deployment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id TEXT NOT NULL UNIQUE REFERENCES mods(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL REFERENCES games(id),
    status TEXT NOT NULL,
    strategy TEXT NOT NULL,
    deployed_files TEXT DEFAULT '[]',    -- JSON array
    conflicts TEXT DEFAULT '[]',        -- JSON array
    deployed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_deployment_game ON deployment(game_id);
CREATE INDEX idx_deployment_status ON deployment(status);
```

### Load Order Table

```sql
CREATE TABLE load_order (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    plugin_name TEXT NOT NULL,
    load_order_index INTEGER NOT NULL,
    enabled INTEGER DEFAULT 1,
    group_name TEXT,
    UNIQUE(game_id, plugin_name)
);

CREATE INDEX idx_load_order_game ON load_order(game_id);
CREATE INDEX idx_load_order_index ON load_order(load_order_index);
```

### Download Queue Table

```sql
CREATE TABLE downloads (
    id TEXT PRIMARY KEY,
    file_name TEXT NOT NULL,
    url TEXT NOT NULL,
    destination TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending',
    bytes_written INTEGER DEFAULT 0,
    bytes_total INTEGER,
    etag TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    error TEXT
);

CREATE INDEX idx_downloads_state ON downloads(state);
```

### Settings Table

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Migrations Table

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Data Access Layer

```rust
// Database connection pool
pub struct Database {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, String> { ... }
    
    // Prepared statements for games
    pub fn find_game(&self, id: &str) -> Result<Option<Game>, String> { ... }
    pub fn list_games(&self) -> Result<Vec<Game>, String> { ... }
    pub fn insert_game(&self, game: &Game) -> Result<(), String> { ... }
    pub fn update_game(&self, game: &Game) -> Result<(), String> { ... }
    pub fn delete_game(&self, id: &str) -> Result<(), String> { ... }
    
    // Prepared statements for mods
    pub fn find_mod(&self, id: &str) -> Result<Option<Mod>, String> { ... }
    pub fn list_mods(&self, game_id: &str) -> Result<Vec<Mod>, String> { ... }
    pub fn insert_mod(&self, mod_: &Mod) -> Result<(), String> { ... }
    pub fn update_mod(&self, mod_: &Mod) -> Result<(), String> { ... }
    pub fn delete_mod(&self, id: &str) -> Result<(), String> { ... }
    pub fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, String> { ... }
    pub fn insert_mod_file(&self, file: &ModFile) -> Result<(), String> { ... }
    
    // Prepared statements for deployment
    pub fn get_deployment_state(&self, game_id: &str) -> Result<Vec<DeploymentState>, String> { ... }
    pub fn upsert_deployment(&self, state: &DeploymentState) -> Result<(), String> { ... }
    
    // Prepared statements for load order
    pub fn get_load_order(&self, game_id: &str) -> Result<Vec<LoadOrderEntry>, String> { ... }
    pub fn set_load_order(&self, game_id: &str, entries: &[LoadOrderEntry]) -> Result<(), String> { ... }
}
```

## Pantheon Persistence Comparison

| Pantheon | Pantheon | Notes |
|--------|----------|-------|
| LevelDB | SQLite | SQLite is simpler, single file |
| DuckDB | SQL queries | Reactive queries via r2d2 |
| Redux Persistor | Direct DB writes | Tauri IPC for state sync |
| plugins.txt | load_order table | JSON persistence |
| loadorder.txt | load_order table | Structured storage |

## State Synchronization

```rust
// Tauri IPC for state sync between Rust backend and Solid.js frontend

#[tauri::command]
pub async fn get_full_state() -> Result<AppState, String> { ... }

#[tauri::command]
pub async fn persist_state_diff(diff: StateDiff) -> Result<(), String> { ... }

// State diff format
struct StateDiff {
    path: Vec<String>,  // e.g., ["session", "base", "notifications"]
    operation: DiffOp,
    value: Option<serde_json::Value>,
}

enum DiffOp {
    Set { value: serde_json::Value },
    Remove,
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | Stores discovered games |
| `mod-installer` | Creates mod records, stores mod files |
| `deploy-manager` | Stores deployment state, deployed files |
| `load-order-manager` | Stores plugin load order |
| `settings` | Stores user preferences |

## Notes

- Use prepared statements for all queries (performance)
- Connection pooling via r2d2 for concurrent access
- JSON columns for flexible attributes/metadata
- Migrations tracked in schema_migrations table
- Periodic vacuum to keep DB size manageable
