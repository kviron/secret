use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::ZipArchive;

use crate::db::Database;
use crate::models::{Mod, ModFile};

pub struct ModInstaller {
    staging_path: PathBuf,
}

impl ModInstaller {
    pub fn new(staging_path: PathBuf) -> Self {
        ModInstaller { staging_path }
    }

    pub async fn install(
        &self,
        db: &Database,
        game_id: &str,
        archive_path: &Path,
    ) -> Result<Mod, String> {
        let mod_id = Uuid::new_v4().to_string();
        let staging_path = self.staging_path.join("mods").join(&mod_id);

        fs::create_dir_all(&staging_path).map_err(|e| e.to_string())?;

        self.extract_archive(archive_path, &staging_path)?;

        let name = self
            .extract_mod_name(&staging_path)
            .unwrap_or_else(|| "Unknown Mod".to_string());

        let version = None;
        let mod_type = "simple".to_string();

        let mod_ = Mod {
            id: mod_id.clone(),
            game_id: game_id.to_string(),
            name: name.clone(),
            version,
            mod_type,
            install_path: staging_path.clone(),
            enabled: false,
        };

        db.insert_mod(&mod_)?;

        let files = self.collect_files(&staging_path, &staging_path)?;
        for (relative_path, size) in files {
            let file = ModFile {
                id: 0,
                mod_id: mod_id.clone(),
                path: relative_path,
                size,
            };
            db.insert_mod_file(&file)?;
        }

        Ok(mod_)
    }

    pub async fn uninstall(&self, db: &Database, mod_id: &str) -> Result<(), String> {
        let mod_ = db
            .find_mod(mod_id)?
            .ok_or_else(|| format!("Mod not found: {}", mod_id))?;

        if mod_.install_path.exists() {
            fs::remove_dir_all(&mod_.install_path)
                .map_err(|e| format!("Failed to remove staging folder: {}", e))?;
        }

        db.delete_mod(mod_id)?;

        Ok(())
    }

    fn extract_archive(&self, archive_path: &Path, dest: &Path) -> Result<(), String> {
        let file =
            File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid zip file: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {}", e))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
                let mut outfile =
                    File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
        }

        Ok(())
    }

    fn extract_mod_name(&self, staging_path: &Path) -> Option<String> {
        let mod_json = staging_path.join("mod.json");
        if mod_json.exists() {
            if let Ok(content) = fs::read_to_string(&mod_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                        return Some(name.to_string());
                    }
                }
            }
        }

        staging_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
    }

    fn collect_files(&self, base: &Path, current: &Path) -> Result<Vec<(String, u64)>, String> {
        let mut files = Vec::new();

        for entry in fs::read_dir(current).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                files.extend(self.collect_files(base, &path)?);
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(base)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push((relative, size));
            }
        }

        Ok(files)
    }
}
