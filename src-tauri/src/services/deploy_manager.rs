use chrono::Utc;
use std::fs;

use crate::db::Database;
use crate::models::{DeployedFile, DeploymentState};

pub struct DeployManager {
    db: Database,
}

impl DeployManager {
    pub fn new(db: Database) -> Self {
        DeployManager { db }
    }

    pub async fn deploy_mod(&self, mod_id: &str) -> Result<DeploymentState, String> {
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

        let mut deployed_files = Vec::new();

        for file in &files {
            let source = staging_path.join(&file.path);
            let target = game_path.join(&file.path);

            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
            }

            fs::copy(&source, &target)
                .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", source, target, e))?;

            deployed_files.push(DeployedFile {
                source: file.path.clone(),
                target: file.path.clone(),
                size: file.size,
            });
        }

        let state = DeploymentState {
            mod_id: mod_id.to_string(),
            game_id: mod_.game_id.clone(),
            status: "deployed".to_string(),
            strategy: "copy".to_string(),
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
                if target.exists() {
                    fs::remove_file(&target)
                        .map_err(|e| format!("Failed to remove {:?}: {}", target, e))?;
                }
            }
        }

        self.db.delete_deployment(mod_id)?;
        self.db.update_mod_enabled(mod_id, false)?;

        Ok(())
    }

    pub async fn enable_mod(&self, mod_id: &str) -> Result<DeploymentState, String> {
        self.deploy_mod(mod_id).await
    }

    pub async fn disable_mod(&self, mod_id: &str) -> Result<(), String> {
        self.undeploy_mod(mod_id).await
    }

    pub async fn deploy_all(&self, game_id: &str) -> Result<Vec<DeploymentState>, String> {
        let mods = self.db.list_mods(game_id)?;
        let mut states = Vec::new();

        for mod_ in mods {
            if mod_.enabled {
                let state = self.deploy_mod(&mod_.id).await?;
                states.push(state);
            }
        }

        Ok(states)
    }
}
