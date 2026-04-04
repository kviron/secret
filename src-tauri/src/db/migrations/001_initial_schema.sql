PRAGMA foreign_keys = ON;

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

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);

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