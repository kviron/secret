use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[cfg(windows)]
use std::os::windows::fs::{symlink_dir, symlink_file};

use crate::db::Database;
use crate::models::{Conflict, DeployStrategy, DeployedFile, DeploymentState, ModFile};

pub struct DeployManager {
    db: Database,
}

impl DeployManager {
    pub fn new(db: Database) -> Self {
        DeployManager { db }
    }

    pub async fn deploy_mod(
        &self,
        mod_id: &str,
        strategy: DeployStrategy,
    ) -> Result<DeploymentState, String> {
        let mod_ = self
            .db
            .find_mod(mod_id)?
            .ok_or_else(|| format!("Mod not found: {}", mod_id))?;

        let game = self
            .db
            .find_game(&mod_.game_id)?
            .ok_or_else(|| format!("Game not found: {}", mod_.game_id))?;

        let files = self.db.get_mod_files(mod_id)?;

        let game_path = game.support_path.clone();
        let staging_path = mod_.install_path.clone();

        let (strategy_used, deployed_files) =
            self.deploy_files(&files, &staging_path, &game_path, &strategy)?;

        let state = DeploymentState {
            mod_id: mod_id.to_string(),
            game_id: mod_.game_id.clone(),
            status: "deployed".to_string(),
            strategy: strategy_used,
            deployed_files,
            deployed_at: Some(Utc::now().to_rfc3339()),
        };

        self.db.upsert_deployment(&state)?;
        self.db.update_mod_enabled(mod_id, true)?;

        Ok(state)
    }

    pub async fn undeploy_mod(&self, mod_id: &str) -> Result<(), String> {
        let deployment = self.db.get_deployment_state(mod_id)?;

        if let Some(state) = deployment {
            let mod_ = self
                .db
                .find_mod(mod_id)?
                .ok_or_else(|| format!("Mod not found: {}", mod_id))?;

            let game = self
                .db
                .find_game(&mod_.game_id)?
                .ok_or_else(|| format!("Game not found: {}", mod_.game_id))?;

            let game_path = game.support_path;

            for deployed_file in &state.deployed_files {
                let target = game_path.join(&deployed_file.target);
                self.remove_link(&target);
            }
        }

        self.db.delete_deployment(mod_id)?;
        self.db.update_mod_enabled(mod_id, false)?;

        Ok(())
    }

    pub async fn enable_mod(
        &self,
        mod_id: &str,
        strategy: DeployStrategy,
    ) -> Result<DeploymentState, String> {
        self.deploy_mod(mod_id, strategy).await
    }

    pub async fn disable_mod(&self, mod_id: &str) -> Result<(), String> {
        self.undeploy_mod(mod_id).await
    }

    pub async fn deploy_all(
        &self,
        game_id: &str,
        strategy: DeployStrategy,
    ) -> Result<Vec<DeploymentState>, String> {
        let mods = self.db.list_mods(game_id)?;
        let mut states = Vec::new();

        for mod_ in mods {
            if mod_.enabled {
                let state = self.deploy_mod(&mod_.id, strategy.clone()).await?;
                states.push(state);
            }
        }

        Ok(states)
    }

    pub async fn check_conflicts(&self, game_id: &str) -> Result<Vec<Conflict>, String> {
        let mods = self.db.list_mods(game_id)?;
        let mut file_owners: HashMap<String, String> = HashMap::new();
        let mut conflicts = Vec::new();

        for mod_ in &mods {
            if !mod_.enabled {
                continue;
            }
            let files = self.db.get_mod_files(&mod_.id)?;
            for file in &files {
                if let Some(existing_mod) = file_owners.get(&file.path) {
                    conflicts.push(Conflict {
                        file_path: file.path.clone(),
                        mod_a: existing_mod.clone(),
                        mod_b: mod_.id.clone(),
                    });
                } else {
                    file_owners.insert(file.path.clone(), mod_.id.clone());
                }
            }
        }

        Ok(conflicts)
    }

    // --- Internal link logic ---

    fn deploy_files(
        &self,
        files: &[ModFile],
        staging_path: &Path,
        game_path: &Path,
        strategy: &DeployStrategy,
    ) -> Result<(String, Vec<DeployedFile>), String> {
        let mut deployed_files = Vec::new();
        let mut last_error: Option<String> = None;

        let fallback_chain: Vec<DeployStrategy> = match strategy {
            DeployStrategy::Auto => vec![
                DeployStrategy::Symlink,
                DeployStrategy::Hardlink,
                DeployStrategy::Copy,
            ],
            s => vec![s.clone()],
        };

        for strat in &fallback_chain {
            deployed_files.clear();
            last_error = None;

            let mut ok = true;
            for file in files {
                let source = staging_path.join(&file.path);
                let target = game_path.join(&file.path);

                if let Some(parent) = target.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        last_error =
                            Some(format!("Failed to create directory {:?}: {}", parent, e));
                        ok = false;
                        break;
                    }
                }

                match self.create_link(&source, &target, strat) {
                    Ok(()) => {
                        deployed_files.push(DeployedFile {
                            source: file.path.clone(),
                            target: file.path.clone(),
                            size: file.size,
                        });
                    }
                    Err(e) => {
                        last_error = Some(e);
                        ok = false;
                        break;
                    }
                }
            }

            if ok {
                return Ok((strat.as_str().to_string(), deployed_files));
            }

            // Rollback what we deployed in this attempt
            for df in &deployed_files {
                let target = game_path.join(&df.target);
                self.remove_link(&target);
            }
            deployed_files.clear();
        }

        Err(last_error.unwrap_or_else(|| "All deployment strategies failed".to_string()))
    }

    fn create_link(
        &self,
        source: &Path,
        target: &Path,
        strategy: &DeployStrategy,
    ) -> Result<(), String> {
        match strategy {
            DeployStrategy::Symlink => self.create_symlink(source, target),
            DeployStrategy::Hardlink => self.create_hardlink(source, target),
            DeployStrategy::Copy => self.copy_file(source, target),
            DeployStrategy::Auto => unreachable!("Auto should be resolved before create_link"),
        }
    }

    fn create_symlink(&self, source: &Path, target: &Path) -> Result<(), String> {
        if target.exists() || target.symlink_metadata().is_ok() {
            self.remove_link(target);
        }

        if source.is_dir() {
            self.symlink_dir_inner(source, target)
        } else {
            self.symlink_file_inner(source, target)
        }
    }

    #[cfg(windows)]
    fn symlink_file_inner(&self, source: &Path, target: &Path) -> Result<(), String> {
        symlink_file(source, target).map_err(|e| format!("Symlink failed for {:?}: {}", target, e))
    }

    #[cfg(not(windows))]
    fn symlink_file_inner(&self, source: &Path, target: &Path) -> Result<(), String> {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("Symlink failed for {:?}: {}", target, e))
    }

    #[cfg(windows)]
    fn symlink_dir_inner(&self, source: &Path, target: &Path) -> Result<(), String> {
        symlink_dir(source, target)
            .map_err(|e| format!("Directory symlink failed for {:?}: {}", target, e))
    }

    #[cfg(not(windows))]
    fn symlink_dir_inner(&self, source: &Path, target: &Path) -> Result<(), String> {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("Directory symlink failed for {:?}: {}", target, e))
    }

    fn create_hardlink(&self, source: &Path, target: &Path) -> Result<(), String> {
        // Hardlinks only work for files, not directories
        if source.is_dir() {
            return Err(format!(
                "Hardlink not supported for directory: {:?}",
                source
            ));
        }

        if target.exists() || target.symlink_metadata().is_ok() {
            self.remove_link(target);
        }

        fs::hard_link(source, target)
            .map_err(|e| format!("Hardlink failed for {:?}: {}", target, e))
    }

    fn copy_file(&self, source: &Path, target: &Path) -> Result<(), String> {
        if target.exists() || target.symlink_metadata().is_ok() {
            self.remove_link(target);
        }

        if source.is_dir() {
            self.copy_dir_recursive(source, target)
        } else {
            fs::copy(source, target)
                .map_err(|e| format!("Copy failed for {:?} -> {:?}: {}", source, target, e))?;
            Ok(())
        }
    }

    fn copy_dir_recursive(&self, source: &Path, target: &Path) -> Result<(), String> {
        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create dir {:?}: {}", target, e))?;

        for entry in
            fs::read_dir(source).map_err(|e| format!("Failed to read dir {:?}: {}", source, e))?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let src = entry.path();
            let dst = target.join(entry.file_name());
            if src.is_dir() {
                self.copy_dir_recursive(&src, &dst)?;
            } else {
                fs::copy(&src, &dst)
                    .map_err(|e| format!("Copy failed {:?} -> {:?}: {}", src, dst, e))?;
            }
        }
        Ok(())
    }

    fn remove_link(&self, target: &Path) {
        // Symlink metadata tells us the link type, not the target type
        if let Ok(meta) = target.symlink_metadata() {
            if meta.is_symlink() {
                if meta.is_dir() {
                    let _ = fs::remove_dir(target);
                } else {
                    let _ = fs::remove_file(target);
                }
                return;
            }
        }

        if target.is_file() {
            let _ = fs::remove_file(target);
        } else if target.is_dir() {
            // Only remove empty dirs; never rm -rf game directories
            let _ = fs::remove_dir(target);
        }
    }
}
