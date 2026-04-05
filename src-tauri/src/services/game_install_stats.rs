//! Размер папки установки и сведения из Steam `appmanifest_*.acf` (как в Vortex: место и build id).

use crate::models::{Game, GameInstallStats};
use crate::services::pe_version;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn parse_acf_field(content: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\"", field);
    for line in content.lines() {
        let line = line.trim();
        if line.contains(&needle) {
            return line.split('"').nth(3).map(String::from);
        }
    }
    None
}

fn parse_acf_u64(content: &str, field: &str) -> Option<u64> {
    parse_acf_field(content, field)?.parse().ok()
}

fn parse_libraryfolders_vdf_paths(content: &str) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if !line.contains("\"path\"") {
            continue;
        }
        if let Some(path) = line.split('"').nth(3) {
            let p = path.replace("\\\\", "\\");
            if !p.is_empty() {
                folders.push(PathBuf::from(p));
            }
        }
    }
    folders
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths {
        let k = p.to_string_lossy().to_lowercase();
        if seen.insert(k) {
            out.push(p);
        }
    }
    out
}

fn push_steam_library_roots(out: &mut Vec<PathBuf>, steam_root: &Path) {
    let steamapps = steam_root.join("steamapps");
    if steamapps.is_dir() {
        out.push(steamapps.clone());
    }
    let lib = steamapps.join("libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&lib) {
        for folder in parse_libraryfolders_vdf_paths(&content) {
            let sp = folder.join("steamapps");
            if sp.is_dir() {
                out.push(sp);
            }
        }
    }
}

/// Каталоги `.../steamapps` (основной Steam + библиотеки из `libraryfolders.vdf`).
fn steam_steamapps_directories() -> Vec<PathBuf> {
    let mut out = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(steam_key) = hkcu.open_subkey("Software\\Valve\\Steam") {
            if let Ok(steam_path) = steam_key.get_value::<String, _>("SteamPath") {
                let steam_root = PathBuf::from(steam_path);
                push_steam_library_roots(&mut out, &steam_root);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home = PathBuf::from(home);
            for root in [home.join(".steam/steam"), home.join(".local/share/Steam")] {
                if root.is_dir() {
                    push_steam_library_roots(&mut out, &root);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let root = PathBuf::from(home).join("Library/Application Support/Steam");
            if root.is_dir() {
                push_steam_library_roots(&mut out, &root);
            }
        }
    }

    dedupe_paths(out)
}

fn find_steam_appmanifest(app_id: u32) -> Option<PathBuf> {
    let name = format!("appmanifest_{}.acf", app_id);
    for steamapps in steam_steamapps_directories() {
        let p = steamapps.join(&name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Суммарный размер файлов под `root`; симлинки не обходим (каталоги-ссылки и файлы-ссылки пропускаем).
pub fn directory_size_skip_symlinks(root: &Path) -> std::io::Result<u64> {
    if !root.exists() {
        return Ok(0);
    }
    let meta = fs::symlink_metadata(root)?;
    if meta.is_symlink() {
        return Ok(0);
    }
    if root.is_file() {
        return Ok(meta.len());
    }
    let mut total = 0u64;
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let read = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let path = entry.path();
            let e_meta = match fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if e_meta.is_symlink() {
                continue;
            }
            if e_meta.is_dir() {
                dirs.push(path);
            } else {
                total += e_meta.len();
            }
        }
    }
    Ok(total)
}

/// Размер с учётом симлинков (обход каталогов-ссылок и размер целевых файлов).
pub fn directory_size_follow_symlinks(root: &Path) -> std::io::Result<u64> {
    let mut visited = HashSet::<PathBuf>::new();
    directory_size_follow_symlinks_inner(root, &mut visited)
}

fn directory_size_follow_symlinks_inner(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> std::io::Result<u64> {
    let sm = fs::symlink_metadata(path)?;

    if sm.is_symlink() {
        match fs::metadata(path) {
            Ok(m) if m.is_file() => return Ok(m.len()),
            Ok(m) if m.is_dir() => {
                let canon = path.canonicalize()?;
                if !visited.insert(canon) {
                    return Ok(0);
                }
                let mut total = 0u64;
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    total += directory_size_follow_symlinks_inner(&entry.path(), visited)?;
                }
                return Ok(total);
            }
            _ => return Ok(0),
        }
    }

    if sm.is_file() {
        return Ok(sm.len());
    }

    if sm.is_dir() {
        let canon = path.canonicalize()?;
        if !visited.insert(canon) {
            return Ok(0);
        }
        let mut total = 0u64;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            total += directory_size_follow_symlinks_inner(&entry.path(), visited)?;
        }
        return Ok(total);
    }

    Ok(0)
}

pub fn compute_game_install_stats(game: &Game) -> Result<GameInstallStats, String> {
    let path = game.install_path.as_path();
    if !path.exists() {
        return Err("Install path does not exist".into());
    }
    if !path.is_dir() {
        return Err("Install path is not a directory".into());
    }

    let disk_usage_bytes = directory_size_follow_symlinks(path).map_err(|e| e.to_string())?;
    let disk_usage_bytes_no_symlinks =
        directory_size_skip_symlinks(path).map_err(|e| e.to_string())?;

    let mut steam_size_on_disk_bytes = None;
    let mut steam_build_id = None;

    if let Some(app_id) = game.details.steam_app_id {
        if let Some(manifest_path) = find_steam_appmanifest(app_id) {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                steam_size_on_disk_bytes = parse_acf_u64(&content, "SizeOnDisk");
                steam_build_id = parse_acf_field(&content, "buildid");
            }
        }
    }

    let mut installed_version_label =
        pe_version::pe_file_version_label(path, &game.details.required_files);
    if installed_version_label.is_none() {
        installed_version_label = steam_build_id
            .as_ref()
            .map(|b| format!("Steam build {}", b));
    }

    Ok(GameInstallStats {
        disk_usage_bytes,
        disk_usage_bytes_no_symlinks,
        steam_size_on_disk_bytes,
        steam_build_id,
        installed_version_label,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_build_and_size() {
        let acf = r#""AppState"
{
	"buildid"		"1234567"
	"SizeOnDisk"		"999888777"
}"#;
        assert_eq!(parse_acf_field(acf, "buildid").as_deref(), Some("1234567"));
        assert_eq!(parse_acf_u64(acf, "SizeOnDisk"), Some(999888777u64));
    }
}
