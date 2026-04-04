use rusqlite::{params, Connection, OptionalExtension, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::models::{DeploymentState, Game, GameLauncher, Mod, ModFile};

const MIGRATION_001: &str = include_str!("migrations/001_initial_schema.sql");

#[derive(Debug, Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
        let db_path = app_data_dir.join("pantheon.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| e.to_string())?;
        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(MIGRATION_001)
            .map_err(|e| format!("Migration failed: {}", e))?;

        Self::add_missing_columns(
            &conn,
            "games",
            &[
                (
                    "extension_id",
                    "ALTER TABLE games ADD COLUMN extension_id TEXT",
                ),
                (
                    "supported_mod_types",
                    "ALTER TABLE games ADD COLUMN supported_mod_types TEXT DEFAULT '[]'",
                ),
                (
                    "merge_mods",
                    "ALTER TABLE games ADD COLUMN merge_mods INTEGER DEFAULT 0",
                ),
                (
                    "details",
                    "ALTER TABLE games ADD COLUMN details TEXT DEFAULT '{}'",
                ),
            ],
        )?;

        Self::add_missing_columns(&conn, "mods", &[
            ("category", "ALTER TABLE mods ADD COLUMN category TEXT"),
            ("flags", "ALTER TABLE mods ADD COLUMN flags TEXT DEFAULT '[]'"),
            ("attributes", "ALTER TABLE mods ADD COLUMN attributes TEXT DEFAULT '{}'"),
            ("last_modified", "ALTER TABLE mods ADD COLUMN last_modified TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))"),
            ("metadata", "ALTER TABLE mods ADD COLUMN metadata TEXT"),
            ("conflicts", "ALTER TABLE mods ADD COLUMN conflicts TEXT DEFAULT '[]'"),
            ("dependencies", "ALTER TABLE mods ADD COLUMN dependencies TEXT DEFAULT '[]'"),
        ])?;

        Self::add_missing_columns(
            &conn,
            "mod_files",
            &[
                ("hash", "ALTER TABLE mod_files ADD COLUMN hash TEXT"),
                (
                    "is_archive",
                    "ALTER TABLE mod_files ADD COLUMN is_archive INTEGER DEFAULT 0",
                ),
            ],
        )?;

        Self::add_missing_columns(&conn, "deployment", &[
            ("conflicts", "ALTER TABLE deployment ADD COLUMN conflicts TEXT DEFAULT '[]'"),
            ("created_at", "ALTER TABLE deployment ADD COLUMN created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))"),
            ("updated_at", "ALTER TABLE deployment ADD COLUMN updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))"),
        ])?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_games_extension ON games(extension_id)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mod_files_hash ON mod_files(hash)",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn add_missing_columns(
        conn: &Connection,
        table: &str,
        columns: &[(&str, &str)],
    ) -> Result<(), String> {
        let existing: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| e.to_string())?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        for (col_name, sql) in columns {
            if !existing.iter().any(|c| c == col_name) {
                conn.execute(sql, []).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    fn parse_launcher(s: &str) -> GameLauncher {
        match s {
            "steam" => GameLauncher::Steam,
            "gog" => GameLauncher::GOG,
            "epic" => GameLauncher::Epic,
            "xbox" => GameLauncher::Xbox,
            "origin" => GameLauncher::Origin,
            "manual" => GameLauncher::Manual,
            _ => GameLauncher::Steam,
        }
    }

    // === Games ===

    pub fn insert_or_update_game(&self, game: &Game) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let launcher_str = game.launcher.as_str();
        let details_json = serde_json::to_string(&game.details).map_err(|e| e.to_string())?;
        let mod_types_json =
            serde_json::to_string(&game.supported_mod_types).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO games 
             (id, name, install_path, support_path, launcher, extension_id, supported_mod_types, merge_mods, details, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                game.id,
                game.name,
                game.install_path.to_string_lossy().to_string(),
                game.support_path.to_string_lossy().to_string(),
                launcher_str,
                game.extension_id,
                mod_types_json,
                if game.merge_mods { 1 } else { 0 },
                details_json,
                game.created_at,
                game.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<Game>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, install_path, support_path, launcher, 
                        extension_id, supported_mod_types, merge_mods, details, created_at, updated_at 
                 FROM games ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let games = stmt
            .query_map([], |row| {
                let details_json: String = row.get(8)?;
                let mod_types_json: String = row.get(6)?;
                let launcher_str: String = row.get(4)?;

                Ok(Game {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    install_path: PathBuf::from(row.get::<_, String>(2)?),
                    support_path: PathBuf::from(row.get::<_, String>(3)?),
                    launcher: Database::parse_launcher(&launcher_str),
                    extension_id: row.get(5)?,
                    supported_mod_types: serde_json::from_str(&mod_types_json).unwrap_or_default(),
                    merge_mods: row.get::<_, i32>(7)? == 1,
                    details: serde_json::from_str(&details_json).unwrap_or_default(),
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;
        games
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn find_game(&self, id: &str) -> Result<Option<Game>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, install_path, support_path, launcher, 
                        extension_id, supported_mod_types, merge_mods, details, created_at, updated_at 
                 FROM games WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let game = stmt
            .query_row(params![id], |row| {
                let details_json: String = row.get(8)?;
                let mod_types_json: String = row.get(6)?;
                let launcher_str: String = row.get(4)?;

                Ok(Game {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    install_path: PathBuf::from(row.get::<_, String>(2)?),
                    support_path: PathBuf::from(row.get::<_, String>(3)?),
                    launcher: Database::parse_launcher(&launcher_str),
                    extension_id: row.get(5)?,
                    supported_mod_types: serde_json::from_str(&mod_types_json).unwrap_or_default(),
                    merge_mods: row.get::<_, i32>(7)? == 1,
                    details: serde_json::from_str(&details_json).unwrap_or_default(),
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(game)
    }

    pub fn delete_game(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM games WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // === Mods ===

    pub fn insert_mod(&self, mod_: &Mod) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO mods (id, game_id, name, version, mod_type, install_path, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                mod_.id,
                mod_.game_id,
                mod_.name,
                mod_.version,
                mod_.mod_type,
                mod_.install_path.to_string_lossy().to_string(),
                if mod_.enabled { 1 } else { 0 },
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_mods(&self, game_id: &str) -> Result<Vec<Mod>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, game_id, name, version, mod_type, install_path, enabled FROM mods WHERE game_id = ?1 ORDER BY name")
            .map_err(|e| e.to_string())?;
        let mods = stmt
            .query_map(params![game_id], |row| {
                Ok(Mod {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    name: row.get(2)?,
                    version: row.get(3)?,
                    mod_type: row.get(4)?,
                    install_path: PathBuf::from(row.get::<_, String>(5)?),
                    enabled: row.get::<_, i32>(6)? == 1,
                })
            })
            .map_err(|e| e.to_string())?;
        mods.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn find_mod(&self, id: &str) -> Result<Option<Mod>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, game_id, name, version, mod_type, install_path, enabled FROM mods WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mod_ = stmt
            .query_row(params![id], |row| {
                Ok(Mod {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    name: row.get(2)?,
                    version: row.get(3)?,
                    mod_type: row.get(4)?,
                    install_path: PathBuf::from(row.get::<_, String>(5)?),
                    enabled: row.get::<_, i32>(6)? == 1,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(mod_)
    }

    pub fn update_mod_enabled(&self, id: &str, enabled: bool) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE mods SET enabled = ?1 WHERE id = ?2",
            params![if enabled { 1 } else { 0 }, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_mod(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM mods WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // === Mod Files ===

    pub fn insert_mod_file(&self, file: &ModFile) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO mod_files (mod_id, path, size) VALUES (?1, ?2, ?3)",
            params![file.mod_id, file.path, file.size as i64],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, mod_id, path, size FROM mod_files WHERE mod_id = ?1 ORDER BY path")
            .map_err(|e| e.to_string())?;
        let files = stmt
            .query_map(params![mod_id], |row| {
                Ok(ModFile {
                    id: row.get(0)?,
                    mod_id: row.get(1)?,
                    path: row.get(2)?,
                    size: row.get::<_, i64>(3)? as u64,
                })
            })
            .map_err(|e| e.to_string())?;
        files
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // === Deployment ===

    pub fn upsert_deployment(&self, state: &DeploymentState) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let deployed_files_json =
            serde_json::to_string(&state.deployed_files).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO deployment (mod_id, game_id, status, strategy, deployed_files, deployed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                state.mod_id,
                state.game_id,
                state.status,
                state.strategy,
                deployed_files_json,
                state.deployed_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_deployment_state(&self, mod_id: &str) -> Result<Option<DeploymentState>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT mod_id, game_id, status, strategy, deployed_files, deployed_at FROM deployment WHERE mod_id = ?1")
            .map_err(|e| e.to_string())?;
        let state = stmt
            .query_row(params![mod_id], |row| {
                let deployed_files_json: String = row.get(4)?;
                let deployed_files: Vec<crate::models::DeployedFile> =
                    serde_json::from_str(&deployed_files_json).unwrap_or_default();
                Ok(DeploymentState {
                    mod_id: row.get(0)?,
                    game_id: row.get(1)?,
                    status: row.get(2)?,
                    strategy: row.get(3)?,
                    deployed_files,
                    deployed_at: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(state)
    }

    pub fn list_deployments(&self, game_id: &str) -> Result<Vec<DeploymentState>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT mod_id, game_id, status, strategy, deployed_files, deployed_at FROM deployment WHERE game_id = ?1")
            .map_err(|e| e.to_string())?;
        let states = stmt
            .query_map(params![game_id], |row| {
                let deployed_files_json: String = row.get(4)?;
                let deployed_files: Vec<crate::models::DeployedFile> =
                    serde_json::from_str(&deployed_files_json).unwrap_or_default();
                Ok(DeploymentState {
                    mod_id: row.get(0)?,
                    game_id: row.get(1)?,
                    status: row.get(2)?,
                    strategy: row.get(3)?,
                    deployed_files,
                    deployed_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        states
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn delete_deployment(&self, mod_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM deployment WHERE mod_id = ?1", params![mod_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
