use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
#[cfg(windows)]
use winreg::RegKey;

/// Депо Steam без отдельной «игры» (редистрибутивы и т.п.) — не регистрируем как `steam_<id>`.
const STEAM_TOOL_APP_IDS: &[u32] = &[228980];

use crate::models::{
    DetectionProgress, Game, GameDetails, GameDetectionError, GameLauncher, ModSupportLevel,
};

#[cfg(windows)]
use rusqlite::{Connection, OpenFlags};

#[derive(Debug, Clone)]
pub struct GameDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub steam_app_id: u32,
    pub executables: &'static [&'static str],
    pub mod_support: ModSupportLevel,
    pub supported_mod_types: &'static [&'static str],
    pub merge_mods: bool,
    pub support_path_suffix: &'static str,
}

pub const KNOWN_GAMES: &[GameDefinition] = &[
    GameDefinition {
        id: "skyrimse",
        name: "Skyrim Special Edition",
        steam_app_id: 489830,
        executables: &["SkyrimSE.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "enb",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "skyrim",
        name: "The Elder Scrolls V: Skyrim",
        steam_app_id: 72850,
        executables: &["TESV.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "enb",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "skyrimvr",
        name: "Skyrim VR",
        steam_app_id: 611670,
        executables: &["SkyrimVR.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "fallout4",
        name: "Fallout 4",
        steam_app_id: 377160,
        executables: &["Fallout4.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "fallout4vr",
        name: "Fallout 4 VR",
        steam_app_id: 611660,
        executables: &["Fallout4VR.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &["simple", "fomod", "bsat", "modPlugin", "gameSaves"],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "falloutnv",
        name: "Fallout: New Vegas",
        steam_app_id: 22380,
        executables: &["FalloutNV.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "oblivion",
        name: "The Elder Scrolls IV: Oblivion",
        steam_app_id: 22330,
        executables: &["Oblivion.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &[
            "simple",
            "fomod",
            "bsat",
            "scriptExtender",
            "dazip",
            "modPlugin",
            "gameSaves",
        ],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "starfield",
        name: "Starfield",
        steam_app_id: 1716740,
        executables: &["Starfield.exe"],
        mod_support: ModSupportLevel::Full,
        supported_mod_types: &["simple", "fomod", "bsat", "modPlugin", "gameSaves"],
        merge_mods: true,
        support_path_suffix: "Data",
    },
    GameDefinition {
        id: "dragonage",
        name: "Dragon Age: Origins",
        steam_app_id: 47810,
        executables: &["DAOrigins.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "dazip"],
        merge_mods: false,
        support_path_suffix: "packages",
    },
    GameDefinition {
        id: "dragonage2",
        name: "Dragon Age II",
        steam_app_id: 1238040,
        executables: &["DragonAge2.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "dazip"],
        merge_mods: false,
        support_path_suffix: "packages",
    },
    GameDefinition {
        id: "witcher3",
        name: "The Witcher 3: Wild Hunt",
        steam_app_id: 292030,
        executables: &["bin\\x64\\witcher3.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "fomod"],
        merge_mods: false,
        support_path_suffix: "Mods",
    },
    GameDefinition {
        id: "cyberpunk2077",
        name: "Cyberpunk 2077",
        steam_app_id: 1091500,
        executables: &["bin\\x64\\Cyberpunk2077.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "fomod", "bepinex"],
        merge_mods: false,
        support_path_suffix: "archive/pc/mod",
    },
    GameDefinition {
        id: "valheim",
        name: "Valheim",
        steam_app_id: 892970,
        executables: &["valheim.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "riskofrain2",
        name: "Risk of Rain 2",
        steam_app_id: 632360,
        executables: &["Risk of Rain 2.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "deeprockgalactic",
        name: "Deep Rock Galactic",
        steam_app_id: 548430,
        executables: &["FSD.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "hollowknight",
        name: "Hollow Knight",
        steam_app_id: 367520,
        executables: &["hollow_knight.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "subnautica",
        name: "Subnautica",
        steam_app_id: 264710,
        executables: &["Subnautica.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "palworld",
        name: "Palworld",
        steam_app_id: 1623730,
        executables: &["Binaries\\Win64\\PalWorld-Win64-Shipping.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex"],
        merge_mods: false,
        support_path_suffix: "BepInEx",
    },
    GameDefinition {
        id: "gtav",
        name: "Grand Theft Auto V",
        steam_app_id: 271590,
        executables: &["GTA5.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "scriptExtender"],
        merge_mods: false,
        support_path_suffix: "mods",
    },
    GameDefinition {
        id: "reddead2",
        name: "Red Dead Redemption 2",
        steam_app_id: 1174180,
        executables: &["RDR2.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "scriptExtender"],
        merge_mods: false,
        support_path_suffix: "mods",
    },
    GameDefinition {
        id: "monsterhunterworld",
        name: "Monster Hunter: World",
        steam_app_id: 582010,
        executables: &["MonsterHunterWorld.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "nativePC",
    },
    GameDefinition {
        id: "dyinglight2",
        name: "Dying Light 2 Stay Human",
        steam_app_id: 534380,
        executables: &["DyingLightGame_x64_rwdi.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "data",
    },
    GameDefinition {
        id: "stardewvalley",
        name: "Stardew Valley",
        steam_app_id: 413150,
        executables: &["StardewValley.exe", "StardewModdingAPI.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "bepinex", "fomod"],
        merge_mods: false,
        support_path_suffix: "Mods",
    },
    GameDefinition {
        id: "terraria",
        name: "Terraria",
        steam_app_id: 105600,
        executables: &["Terraria.exe", "tModLoader.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "tModLoader",
    },
    GameDefinition {
        id: "darksouls3",
        name: "Dark Souls III",
        steam_app_id: 374320,
        executables: &["DarkSoulsIII.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "Game",
    },
    GameDefinition {
        id: "eldenring",
        name: "Elden Ring",
        steam_app_id: 1245620,
        executables: &["eldenring.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "Game",
    },
    GameDefinition {
        id: "baldursgate3",
        name: "Baldur's Gate 3",
        steam_app_id: 1086940,
        executables: &["bg3.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple", "fomod"],
        merge_mods: false,
        support_path_suffix: "Mods",
    },
    GameDefinition {
        id: "divinityoriginalsin2",
        name: "Divinity: Original Sin 2",
        steam_app_id: 435150,
        executables: &["EoCApp.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "Mods",
    },
    GameDefinition {
        id: "cities2",
        name: "Cities: Skylines II",
        steam_app_id: 949230,
        executables: &["Cities2.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "Mods",
    },
    GameDefinition {
        id: "manorlords",
        name: "Manor Lords",
        steam_app_id: 1363080,
        executables: &["ManorLords-Win64-Shipping.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    // Blizzard / Battle.net games
    GameDefinition {
        id: "overwatch2",
        name: "Overwatch 2",
        steam_app_id: 0,
        executables: &["Overwatch Launcher.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "diablo4",
        name: "Diablo IV",
        steam_app_id: 0,
        executables: &["Diablo IV.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "diablo2r",
        name: "Diablo II: Resurrected",
        steam_app_id: 0,
        executables: &["D2R.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "diablo3",
        name: "Diablo III",
        steam_app_id: 0,
        executables: &["Diablo III.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "wow",
        name: "World of Warcraft",
        steam_app_id: 0,
        executables: &["_retail_\\Wow.exe", "Wow.exe"],
        mod_support: ModSupportLevel::Partial,
        supported_mod_types: &["simple"],
        merge_mods: false,
        support_path_suffix: "_retail_\\Interface\\AddOns",
    },
    GameDefinition {
        id: "starcraft2",
        name: "StarCraft II",
        steam_app_id: 0,
        executables: &["SC2_x64.exe", "SC2.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "hearthstone",
        name: "Hearthstone",
        steam_app_id: 0,
        executables: &["Hearthstone.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
    GameDefinition {
        id: "warcraft3r",
        name: "Warcraft III: Reforged",
        steam_app_id: 0,
        executables: &["_retail_\\x86_64\\Warcraft III.exe"],
        mod_support: ModSupportLevel::None,
        supported_mod_types: &[],
        merge_mods: false,
        support_path_suffix: "",
    },
];

fn dedupe_games_by_id(games: Vec<Game>) -> Vec<Game> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for g in games {
        if seen.insert(g.id.clone()) {
            out.push(g);
        }
    }
    out
}

/// Индекс относительного пути exe → игры каталога (для быстрого поиска без полного перебора).
fn exe_rel_path_index() -> &'static HashMap<String, Vec<&'static GameDefinition>> {
    static CELL: OnceLock<HashMap<String, Vec<&'static GameDefinition>>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut m: HashMap<String, Vec<&'static GameDefinition>> = HashMap::new();
        for gd in KNOWN_GAMES {
            for exe in gd.executables {
                m.entry((*exe).to_string()).or_default().push(gd);
            }
        }
        m
    })
}

/// Множество `GameDefinition.id`, для которых в `install_path` существует хотя бы один известный exe.
fn game_ids_with_listed_exe_at(install_path: &Path) -> HashSet<&'static str> {
    let mut ids = HashSet::new();
    for (rel, defs) in exe_rel_path_index().iter() {
        if install_path.join(rel).exists() {
            for gd in defs {
                ids.insert(gd.id);
            }
        }
    }
    ids
}

#[cfg(target_os = "linux")]
fn heroic_json_collect_install_paths(v: &serde_json::Value, out: &mut Vec<PathBuf>) {
    match v {
        serde_json::Value::String(s) => {
            if s.starts_with('/') && Path::new(s).is_dir() {
                out.push(PathBuf::from(s));
            }
        }
        serde_json::Value::Object(map) => {
            for val in map.values() {
                heroic_json_collect_install_paths(val, out);
            }
        }
        serde_json::Value::Array(a) => {
            for x in a {
                heroic_json_collect_install_paths(x, out);
            }
        }
        _ => {}
    }
}

#[cfg(target_os = "linux")]
fn heroic_guess_launcher(v: &serde_json::Value) -> GameLauncher {
    if let Some(s) = v.get("runner").and_then(|x| x.as_str()) {
        let s = s.to_lowercase();
        if s.contains("gog") {
            return GameLauncher::Gog;
        }
        if s.contains("steam") {
            return GameLauncher::Steam;
        }
    }
    if let Some(s) = v.get("store").and_then(|x| x.as_str()) {
        if s.to_lowercase().contains("gog") {
            return GameLauncher::Gog;
        }
    }
    GameLauncher::Epic
}

#[cfg(target_os = "linux")]
fn lutris_install_path_from_yml(content: &str) -> Option<PathBuf> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("game_path:") {
            let p = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                return Some(pb);
            }
        }
        if let Some(rest) = line.strip_prefix("exe:") {
            let p = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            let pb = PathBuf::from(p);
            if pb.is_file() {
                return pb.parent().map(|x| x.to_path_buf());
            }
            if pb.is_dir() {
                return Some(pb);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn lutris_guess_launcher(content: &str) -> GameLauncher {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("runner:") {
            let r = rest.trim().to_lowercase();
            if r.contains("gog") {
                return GameLauncher::Gog;
            }
            if r.contains("steam") {
                return GameLauncher::Steam;
            }
        }
    }
    GameLauncher::Epic
}

pub struct GameDetector;

impl GameDetector {
    pub fn new() -> Self {
        GameDetector
    }

    pub fn detect_games<F, E>(&self, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static,
    {
        let mut games = Vec::new();

        #[cfg(windows)]
        {
            on_progress(DetectionProgress {
                message: "Scanning Steam library...".into(),
                found: 0,
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_steam_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning GOG library...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_gog_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning GOG Galaxy data...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_gog_galaxy_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Epic Games library...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_epic_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning EA app / Origin installs...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_ea_origin_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Ubisoft Connect library...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_ubisoft_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Battle.net installations...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_battlenet_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Amazon Games library...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_amazon_games_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Xbox / Game Pass exports...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_xbox_windows(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Microsoft Store (Uninstall registry)...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_microsoft_store_windows(&on_progress, &on_error));
        }

        #[cfg(target_os = "linux")]
        {
            on_progress(DetectionProgress {
                message: "Scanning Steam library...".into(),
                found: 0,
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_steam_linux(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Heroic Games Launcher data...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_heroic_linux(&on_progress, &on_error));

            on_progress(DetectionProgress {
                message: "Scanning Lutris games...".into(),
                found: games.len(),
                total: 0,
                current_game: None,
            });
            games.extend(self.detect_lutris_linux(&on_progress, &on_error));
        }

        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            on_progress(DetectionProgress {
                message: "Game detection: Steam scanners run on Windows and Linux only; other launchers are Windows-only in this build.".into(),
                found: 0,
                total: 0,
                current_game: None,
            });
        }

        games = dedupe_games_by_id(games);

        let found_count = games.len();
        on_progress(DetectionProgress {
            message: format!(
                "Scan complete. Found {} game{}.",
                found_count,
                if found_count == 1 { "" } else { "s" }
            ),
            found: found_count,
            total: found_count,
            current_game: None,
        });

        games
    }

    pub fn scan_custom_path<F, E>(&self, path: &Path, on_progress: F, on_error: E) -> Vec<Game>
    where
        F: Fn(DetectionProgress) + Send + 'static,
        E: Fn(GameDetectionError) + Send + 'static,
    {
        let mut games = Vec::new();

        if !path.exists() {
            on_error(GameDetectionError {
                game_id: "custom".into(),
                game_name: "Custom Path".into(),
                error: format!("Path does not exist: {}", path.display()),
                recoverable: false,
            });
            return games;
        }

        if !path.is_dir() {
            on_error(GameDetectionError {
                game_id: "custom".into(),
                game_name: "Custom Path".into(),
                error: "Path is not a directory".into(),
                recoverable: false,
            });
            return games;
        }

        let total = KNOWN_GAMES.len();

        on_progress(DetectionProgress {
            message: format!("Scanning {}...", path.display()),
            found: 0,
            total,
            current_game: None,
        });

        let listed_ids = game_ids_with_listed_exe_at(path);
        for game_def in KNOWN_GAMES {
            if !listed_ids.contains(game_def.id) {
                continue;
            }
            let install_path = path.to_path_buf();
            let support_path = if game_def.support_path_suffix.is_empty() {
                install_path.clone()
            } else {
                install_path.join(game_def.support_path_suffix)
            };
            let required_files: Vec<String> =
                game_def.executables.iter().map(|s| s.to_string()).collect();

            games.push(Game {
                id: game_def.id.to_string(),
                name: game_def.name.to_string(),
                install_path: install_path.clone(),
                support_path,
                install_path_missing: false,
                launcher: GameLauncher::Manual,
                extension_id: None,
                supported_mod_types: game_def
                    .supported_mod_types
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                merge_mods: game_def.merge_mods,
                mod_support: game_def.mod_support.clone(),
                details: GameDetails {
                    steam_app_id: None,
                    gog_id: None,
                    epic_app_id: None,
                    logo: None,
                    required_files,
                    environment: std::collections::HashMap::new(),
                },
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            });

            on_progress(DetectionProgress {
                message: format!("Found {}", game_def.name),
                found: games.len(),
                total,
                current_game: Some(game_def.name.to_string()),
            });
        }

        if games.is_empty() {
            let exes = self.list_exe_files_in_dir(path);
            if exes.len() == 1 {
                let exe = exes[0].clone();
                let stem = Path::new(&exe)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("game");
                let slug: String = stem
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .map(|c| c.to_ascii_lowercase())
                    .collect();
                let id = if slug.is_empty() {
                    "manual_game".to_string()
                } else {
                    format!("manual_{}", slug)
                };
                let name = stem.to_string();
                games.push(Game {
                    id,
                    name,
                    install_path: path.to_path_buf(),
                    support_path: path.to_path_buf(),
                    install_path_missing: false,
                    launcher: GameLauncher::Manual,
                    extension_id: None,
                    supported_mod_types: vec!["simple".to_string(), "fomod".to_string()],
                    merge_mods: true,
                    mod_support: ModSupportLevel::None,
                    details: GameDetails {
                        steam_app_id: None,
                        gog_id: None,
                        epic_app_id: None,
                        logo: None,
                        required_files: vec![exe],
                        environment: std::collections::HashMap::new(),
                    },
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                });
                on_progress(DetectionProgress {
                    message: format!("Found {} (unlisted executable)", games.last().unwrap().name),
                    found: games.len(),
                    total,
                    current_game: games.last().map(|g| g.name.clone()),
                });
            } else {
                on_error(GameDetectionError {
                    game_id: "custom".into(),
                    game_name: "Custom Path".into(),
                    error: if exes.is_empty() {
                        "No supported games and no playable executables in this folder".into()
                    } else {
                        "No supported games found; folder has multiple executables — add the game to the catalog or leave only one .exe in the folder".into()
                    },
                    recoverable: true,
                });
            }
        }

        let found_count = games.len();
        on_progress(DetectionProgress {
            message: format!(
                "Scan complete. Found {} game{}.",
                found_count,
                if found_count == 1 { "" } else { "s" }
            ),
            found: found_count,
            total: found_count,
            current_game: None,
        });

        games
    }

    #[cfg(windows)]
    fn detect_steam_windows<F, E>(&self, on_progress: &F, on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let hkcu = match RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            Ok(key) => key,
            Err(_) => {
                on_error(GameDetectionError {
                    game_id: "steam".into(),
                    game_name: "Steam".into(),
                    error: "Steam not found in registry".into(),
                    recoverable: true,
                });
                return Vec::new();
            }
        };

        let steam_path: String = match hkcu.get_value("SteamPath") {
            Ok(p) => p,
            Err(_) => {
                on_error(GameDetectionError {
                    game_id: "steam".into(),
                    game_name: "Steam".into(),
                    error: "Steam path not found in registry".into(),
                    recoverable: true,
                });
                return Vec::new();
            }
        };

        let steam_path = PathBuf::from(steam_path);
        let library_folders = self.get_steam_library_folders(&steam_path);

        let mut search_paths = vec![steam_path.join("steamapps")];
        search_paths.extend(library_folders.iter().map(|p| p.join("steamapps")));

        self.scan_steam_libraries(search_paths, on_progress, on_error)
    }

    #[cfg(target_os = "linux")]
    fn detect_steam_linux<F, E>(&self, on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut search_paths: Vec<PathBuf> = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            let home = PathBuf::from(&home);
            // Стандартные пути
            for root in [home.join(".steam/steam"), home.join(".local/share/Steam")] {
                let sp = root.join("steamapps");
                if sp.is_dir() {
                    search_paths.push(sp);
                }
            }
            // Flatpak
            let flatpak =
                home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps");
            if flatpak.is_dir() && !search_paths.contains(&flatpak) {
                search_paths.push(flatpak);
            }
            // Snap
            let snap = home.join("snap/steam/common/.local/share/Steam/steamapps");
            if snap.is_dir() && !search_paths.contains(&snap) {
                search_paths.push(snap);
            }

            // Дополнительные библиотеки из libraryfolders.vdf
            for sp in &search_paths {
                let vdf = sp.join("libraryfolders.vdf");
                if vdf.is_file() {
                    if let Ok(content) = fs::read_to_string(&vdf) {
                        for folder in steam_parse::parse_libraryfolders_vdf(&content) {
                            let steamapps = folder.join("steamapps");
                            if steamapps.is_dir() && !search_paths.contains(&steamapps) {
                                search_paths.push(steamapps);
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("$HOME not set; skipping Steam Linux detection");
        }

        let mut seen = HashSet::new();
        search_paths.retain(|p| {
            let k = p.to_string_lossy().to_lowercase();
            seen.insert(k)
        });
        if search_paths.is_empty() {
            return Vec::new();
        }
        self.scan_steam_libraries(search_paths, on_progress, _on_error)
    }

    #[cfg(target_os = "linux")]
    fn detect_heroic_linux<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let Ok(home) = std::env::var("HOME") else {
            return out;
        };
        let mut heroic_dirs = Vec::new();
        let heroic_default = PathBuf::from(&home).join(".config").join("heroic");
        if heroic_default.is_dir() {
            heroic_dirs.push(heroic_default);
        }
        // Heroic Flatpak
        let heroic_flatpak =
            PathBuf::from(&home).join(".var/app/com.heroic.games.launcher/config/heroic");
        if heroic_flatpak.is_dir() && !heroic_dirs.iter().any(|d| d == &heroic_flatpak) {
            heroic_dirs.push(heroic_flatpak);
        }
        if heroic_dirs.is_empty() {
            return out;
        }
        for heroic in heroic_dirs {
            let mut jsons = Vec::new();
            GameDetector::collect_json_files_recursive(&heroic, &mut jsons, 0);
            for jf in jsons {
                let Ok(txt) = fs::read_to_string(&jf) else {
                    continue;
                };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else {
                    continue;
                };
                let launcher = heroic_guess_launcher(&v);
                let mut paths = Vec::new();
                heroic_json_collect_install_paths(&v, &mut paths);
                let mut seen = HashSet::new();
                paths.retain(|p| seen.insert(p.to_string_lossy().to_string()));
                for path in paths {
                    if let Some(mut game) =
                        self.try_match_known_game_in_folder(&path, launcher, None, None)
                    {
                        game.details.environment.insert("heroic".into(), "1".into());
                        out.push(game);
                    }
                }
            }
        }
        out
    }

    #[cfg(target_os = "linux")]
    fn detect_lutris_linux<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let Ok(home) = std::env::var("HOME") else {
            return out;
        };
        let mut games_dirs = Vec::new();
        let lutris_default = PathBuf::from(&home).join(".local/share/lutris/games");
        if lutris_default.is_dir() {
            games_dirs.push(lutris_default);
        }
        // Lutris Flatpak
        let lutris_flatpak =
            PathBuf::from(&home).join(".var/app/net.lutris.Lutris/data/lutris/games");
        if lutris_flatpak.is_dir() && !games_dirs.iter().any(|d| d == &lutris_flatpak) {
            games_dirs.push(lutris_flatpak);
        }
        for games_dir in games_dirs {
            let Ok(rd) = fs::read_dir(&games_dir) else {
                continue;
            };
            for e in rd.filter_map(|x| x.ok()) {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) != Some("yml") {
                    continue;
                }
                let Ok(txt) = fs::read_to_string(&p) else {
                    continue;
                };
                let Some(install_path) = lutris_install_path_from_yml(&txt) else {
                    continue;
                };
                let launcher = lutris_guess_launcher(&txt);
                if let Some(mut game) =
                    self.try_match_known_game_in_folder(&install_path, launcher, None, None)
                {
                    game.details.environment.insert("lutris".into(), "1".into());
                    out.push(game);
                }
            }
        }
        out
    }

    fn scan_steam_libraries<F, E>(
        &self,
        search_paths: Vec<PathBuf>,
        on_progress: &F,
        _on_error: &E,
    ) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut games = Vec::new();

        let mut steam_app_ids: HashMap<String, (u32, PathBuf)> = HashMap::new();

        for search_path in &search_paths {
            if !search_path.exists() {
                continue;
            }

            let entries = match fs::read_dir(search_path) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Cannot read Steam library {}: {err}", search_path.display());
                    continue;
                }
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy().to_string();

                if file_name_str.starts_with("appmanifest_") && file_name_str.ends_with(".acf") {
                    if let Some(app_id_str) = file_name_str
                        .strip_prefix("appmanifest_")
                        .and_then(|s| s.strip_suffix(".acf"))
                    {
                        if let Ok(app_id) = app_id_str.parse::<u32>() {
                            if let Some(install_dir) = self.parse_app_manifest(&entry.path()) {
                                steam_app_ids.insert(install_dir, (app_id, entry.path().clone()));
                            }
                        }
                    }
                }
            }
        }

        on_progress(DetectionProgress {
            message: format!("Found {} installed games in Steam", steam_app_ids.len()),
            found: 0,
            total: steam_app_ids.len(),
            current_game: None,
        });

        let mut found_count = 0;
        for (install_dir, (app_id, manifest_path)) in &steam_app_ids {
            if self.find_game_by_app_id(*app_id).is_none() && STEAM_TOOL_APP_IDS.contains(app_id) {
                continue;
            }
            for search_path in &search_paths {
                let common_path = search_path.join("common").join(install_dir);
                if !common_path.exists() {
                    continue;
                }

                if let Some(game_def) = self.find_game_by_app_id(*app_id) {
                    let listed_ids = game_ids_with_listed_exe_at(&common_path);
                    let matched_listed_exe = listed_ids.contains(game_def.id);

                    let fallback_exe = if !matched_listed_exe {
                        self.fallback_exe_for_known_game(&common_path, game_def)
                    } else {
                        None
                    };

                    if matched_listed_exe || fallback_exe.is_some() {
                        let support_path = if game_def.support_path_suffix.is_empty() {
                            common_path.clone()
                        } else {
                            common_path.join(game_def.support_path_suffix)
                        };

                        let required_files: Vec<String> = if matched_listed_exe {
                            game_def.executables.iter().map(|s| s.to_string()).collect()
                        } else {
                            vec![fallback_exe.expect("checked above")]
                        };

                        games.push(Game {
                            id: game_def.id.to_string(),
                            name: game_def.name.to_string(),
                            install_path: common_path.clone(),
                            support_path,
                            install_path_missing: false,
                            launcher: GameLauncher::Steam,
                            extension_id: None,
                            supported_mod_types: game_def
                                .supported_mod_types
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            merge_mods: game_def.merge_mods,
                            mod_support: game_def.mod_support.clone(),
                            details: GameDetails {
                                steam_app_id: Some(game_def.steam_app_id),
                                gog_id: None,
                                epic_app_id: None,
                                logo: None,
                                required_files,
                                environment: std::collections::HashMap::new(),
                            },
                            created_at: chrono::Utc::now().to_rfc3339(),
                            updated_at: chrono::Utc::now().to_rfc3339(),
                        });

                        found_count += 1;
                        on_progress(DetectionProgress {
                            message: format!("Found {}", game_def.name),
                            found: found_count,
                            total: steam_app_ids.len(),
                            current_game: Some(game_def.name.to_string()),
                        });
                        break;
                    }
                } else {
                    if let Some(first_exe) = self.find_executable_in_dir(&common_path) {
                        let game_name = self.get_game_name_from_manifest(manifest_path);
                        let display_name = game_name.unwrap_or_else(|| {
                            install_dir
                                .split('_')
                                .map(|s| {
                                    let mut chars = s.chars();
                                    match chars.next() {
                                        None => String::new(),
                                        Some(c) => {
                                            let upper =
                                                c.to_uppercase().to_string() + chars.as_str();
                                            upper
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        });

                        games.push(Game {
                            id: format!("steam_{}", app_id),
                            name: display_name,
                            install_path: common_path.clone(),
                            support_path: common_path.clone(),
                            install_path_missing: false,
                            launcher: GameLauncher::Steam,
                            extension_id: None,
                            supported_mod_types: vec![],
                            merge_mods: false,
                            mod_support: ModSupportLevel::None,
                            details: GameDetails {
                                steam_app_id: Some(*app_id),
                                gog_id: None,
                                epic_app_id: None,
                                logo: None,
                                required_files: vec![first_exe],
                                environment: std::collections::HashMap::new(),
                            },
                            created_at: chrono::Utc::now().to_rfc3339(),
                            updated_at: chrono::Utc::now().to_rfc3339(),
                        });

                        found_count += 1;
                        on_progress(DetectionProgress {
                            message: format!(
                                "Found {} (mod support not available)",
                                games.last().unwrap().name
                            ),
                            found: found_count,
                            total: steam_app_ids.len(),
                            current_game: Some(games.last().unwrap().name.clone()),
                        });
                    }
                }
            }
        }

        games
    }

    /// Сопоставить папку установки с [KNOWN_GAMES] (exe в корне / fallback).
    fn try_match_known_game_in_folder(
        &self,
        install_path: &Path,
        launcher: GameLauncher,
        gog_id: Option<String>,
        epic_app_id: Option<String>,
    ) -> Option<Game> {
        if !install_path.is_dir() {
            return None;
        }
        let listed_ids = game_ids_with_listed_exe_at(install_path);
        for game_def in KNOWN_GAMES {
            let matched_listed = listed_ids.contains(game_def.id);
            let fallback_exe = if !matched_listed {
                self.fallback_exe_for_known_game(install_path, game_def)
            } else {
                None
            };
            if !matched_listed && fallback_exe.is_none() {
                continue;
            }
            let required_files: Vec<String> = if matched_listed {
                game_def.executables.iter().map(|s| s.to_string()).collect()
            } else {
                let Some(fe) = fallback_exe else {
                    continue;
                };
                vec![fe]
            };
            let support_path = if game_def.support_path_suffix.is_empty() {
                install_path.to_path_buf()
            } else {
                install_path.join(game_def.support_path_suffix)
            };
            let steam_app_id = if matches!(launcher, GameLauncher::Steam) {
                Some(game_def.steam_app_id)
            } else {
                None
            };
            return Some(Game {
                id: game_def.id.to_string(),
                name: game_def.name.to_string(),
                install_path: install_path.to_path_buf(),
                support_path,
                install_path_missing: false,
                launcher,
                extension_id: None,
                supported_mod_types: game_def
                    .supported_mod_types
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                merge_mods: game_def.merge_mods,
                mod_support: game_def.mod_support.clone(),
                details: GameDetails {
                    steam_app_id,
                    gog_id,
                    epic_app_id,
                    logo: None,
                    required_files,
                    environment: std::collections::HashMap::new(),
                },
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            });
        }
        None
    }

    #[cfg(windows)]
    fn detect_gog_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let hklm = match RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\GOG.com\\Games")
        {
            Ok(k) => k,
            Err(_) => {
                match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\GOG.com\\Games") {
                    Ok(k) => k,
                    Err(_) => return out,
                }
            }
        };

        for key_name in hklm.enum_keys().filter_map(|e| e.ok()) {
            let sub = match hklm.open_subkey(&key_name) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Приоритет: path > exe > InstallDir
            let path_res: Option<String> = sub
                .get_value("path")
                .ok()
                .or_else(|| sub.get_value("PATH").ok());
            let exe_res: Option<String> = sub.get_value("exe").ok().map(|p: String| {
                let pb = PathBuf::from(p.trim().trim_matches('"'));
                pb.parent()
                    .map(|d| d.to_string_lossy().to_string())
                    .unwrap_or_default()
            });
            let dir_res: Option<String> = sub.get_value("InstallDir").ok();
            let install_dir: Option<String> = path_res.or(exe_res).or(dir_res);
            let Some(dir) = install_dir else {
                continue;
            };
            let dir = dir.trim().trim_matches('"');
            if dir.is_empty() {
                continue;
            }
            let install_path = PathBuf::from(dir);
            if let Some(game) = self.try_match_known_game_in_folder(
                &install_path,
                GameLauncher::Gog,
                Some(key_name),
                None,
            ) {
                out.push(game);
            }
        }
        out
    }

    #[cfg(windows)]
    fn detect_epic_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let manifests = PathBuf::from(
            std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into()),
        )
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");
        let Ok(read_dir) = fs::read_dir(&manifests) else {
            return out;
        };
        for entry in read_dir.filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "item" && ext != "manifest" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            let Some(loc) = v.get("InstallLocation").and_then(|x| x.as_str()) else {
                continue;
            };
            let install_path = PathBuf::from(loc.trim());
            let epic_id = v
                .get("CatalogItemId")
                .and_then(|x| x.as_str())
                .map(String::from)
                .or_else(|| {
                    v.get("MainGameAppName")
                        .and_then(|x| x.as_str())
                        .map(String::from)
                });
            if let Some(game) = self.try_match_known_game_in_folder(
                &install_path,
                GameLauncher::Epic,
                None,
                epic_id,
            ) {
                out.push(game);
            }
        }
        out
    }

    #[cfg(windows)]
    fn detect_xbox_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let xbox_root = match RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\XboxGameExport")
        {
            Ok(k) => k,
            Err(_) => return out,
        };
        for key_name in xbox_root.enum_keys().filter_map(|e| e.ok()) {
            let sub = match xbox_root.open_subkey(&key_name) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let path_val: Option<String> = sub
                .get_value("Path")
                .ok()
                .or_else(|| sub.get_value("InstallLocation").ok());
            let Some(p) = path_val else {
                continue;
            };
            let install_path = PathBuf::from(p.trim());
            if let Some(game) =
                self.try_match_known_game_in_folder(&install_path, GameLauncher::Xbox, None, None)
            {
                out.push(game);
            }
        }
        out
    }

    /// EA Desktop / Origin: `HKLM\...\EA Games\<title>\Install Dir`
    /// EA Desktop (new): `HKCU\...\EA Desktop\<title>\InstallLocation`
    #[cfg(windows)]
    fn detect_ea_origin_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        // EA Games (legacy + EA Desktop)
        for root_path in [
            "SOFTWARE\\WOW6432Node\\EA Games",
            "SOFTWARE\\EA Games",
            "SOFTWARE\\Electronic Arts\\EA Games",
            "SOFTWARE\\WOW6432Node\\Electronic Arts\\EA Games",
        ] {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let Ok(ea_root) = hklm.open_subkey(root_path) else {
                continue;
            };
            for key_name in ea_root.enum_keys().filter_map(|e| e.ok()) {
                let Ok(sub) = ea_root.open_subkey(&key_name) else {
                    continue;
                };
                let install_dir: Option<String> = sub
                    .get_value("Install Dir")
                    .ok()
                    .or_else(|| sub.get_value("InstallDir").ok())
                    .or_else(|| sub.get_value("InstallLocation").ok());
                let Some(dir) = install_dir else {
                    continue;
                };
                let install_path = PathBuf::from(dir.trim().trim_matches('"'));
                if install_path.as_os_str().is_empty() {
                    continue;
                }
                if let Some(mut game) = self.try_match_known_game_in_folder(
                    &install_path,
                    GameLauncher::Origin,
                    None,
                    None,
                ) {
                    game.details
                        .environment
                        .insert("ea_registry_key".into(), key_name.clone());
                    out.push(game);
                }
            }
        }

        // EA Desktop: HKCU
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for root_path in [
            "Software\\Electronic Arts\\EA Desktop\\Game Download\\Installs",
            "Software\\Electronic Arts\\EA Desktop\\Installs",
        ] {
            let Ok(ea_root) = hkcu.open_subkey(root_path) else {
                continue;
            };
            for key_name in ea_root.enum_keys().filter_map(|e| e.ok()) {
                let Ok(sub) = ea_root.open_subkey(&key_name) else {
                    continue;
                };
                let install_dir: Option<String> = sub
                    .get_value("InstallLocation")
                    .ok()
                    .or_else(|| sub.get_value("Install Dir").ok())
                    .or_else(|| sub.get_value("InstallDir").ok());
                let Some(dir) = install_dir else {
                    continue;
                };
                let install_path = PathBuf::from(dir.trim().trim_matches('"'));
                if install_path.as_os_str().is_empty() {
                    continue;
                }
                if let Some(mut game) = self.try_match_known_game_in_folder(
                    &install_path,
                    GameLauncher::Origin,
                    None,
                    None,
                ) {
                    game.details
                        .environment
                        .insert("ea_source".into(), "ea_desktop".into());
                    out.push(game);
                }
            }
        }

        out
    }

    /// Ubisoft Connect: `HKLM\...\Ubisoft\Launcher\Installs\<id>\InstallDir`
    #[cfg(windows)]
    fn detect_ubisoft_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let Ok(ubi) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs") else {
            return out;
        };
        for key_name in ubi.enum_keys().filter_map(|e| e.ok()) {
            let Ok(sub) = ubi.open_subkey(&key_name) else {
                continue;
            };
            let install_dir: std::io::Result<String> = sub.get_value("InstallDir");
            let Ok(dir) = install_dir else {
                continue;
            };
            let install_path = PathBuf::from(dir.trim().trim_matches('"'));
            if let Some(mut game) = self.try_match_known_game_in_folder(
                &install_path,
                GameLauncher::Ubisoft,
                None,
                None,
            ) {
                game.details
                    .environment
                    .insert("ubisoft_install_id".into(), key_name.clone());
                out.push(game);
            }
        }
        out
    }

    #[cfg(windows)]
    fn battlenet_product_install_paths() -> Vec<PathBuf> {
        let db = PathBuf::from(
            std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into()),
        )
        .join("Battle.net")
        .join("Agent")
        .join("product.db");
        if !db.is_file() {
            return Vec::new();
        }

        // Try protobuf format first (modern Battle.net client)
        if let Ok(data) = std::fs::read(&db) {
            let mut paths = Vec::new();
            Self::battlenet_extract_paths_from_protobuf(&data, &mut paths);
            if !paths.is_empty() {
                return paths;
            }
        }

        // Fall back to SQLite format (legacy Battle.net client)
        let Ok(conn) = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
            return Vec::new();
        };
        let queries = [
            "SELECT installPath FROM ProductInstall WHERE installPath IS NOT NULL AND length(trim(installPath)) > 0",
            "SELECT installPath FROM ProductInstall",
        ];
        for q in queries {
            if let Ok(mut stmt) = conn.prepare(q) {
                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                    let paths: Vec<PathBuf> =
                        rows.filter_map(|r| r.ok()).map(PathBuf::from).collect();
                    if !paths.is_empty() {
                        return paths;
                    }
                }
            }
        }
        Vec::new()
    }

    /// Decode a varint from `buf` starting at `pos`, returns (value, new_pos).
    #[cfg(windows)]
    fn protobuf_decode_varint(buf: &[u8], pos: usize) -> Option<(u64, usize)> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        let mut p = pos;
        loop {
            if p >= buf.len() {
                return None;
            }
            let b = buf[p] as u64;
            p += 1;
            result |= (b & 0x7F) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
        }
        Some((result, p))
    }

    /// Recursively extract plausible game install paths from a protobuf blob.
    /// Looks for length-delimited (wire-type 2) string fields whose content looks
    /// like an absolute Windows path (e.g. `D:/Games/Overwatch`).
    #[cfg(windows)]
    fn battlenet_extract_paths_from_protobuf(buf: &[u8], out: &mut Vec<PathBuf>) {
        Self::battlenet_parse_protobuf_message(buf, out, 0);
    }

    #[cfg(windows)]
    fn battlenet_parse_protobuf_message(buf: &[u8], out: &mut Vec<PathBuf>, max_depth: usize) {
        let mut pos = 0usize;
        while pos < buf.len() {
            // Decode tag (field_number << 3 | wire_type)
            let Some((tag, new_pos)) = Self::protobuf_decode_varint(buf, pos) else {
                break;
            };
            pos = new_pos;
            let wire_type = (tag & 0x07) as u8;
            // let _field_num = tag >> 3;

            match wire_type {
                0 => {
                    // Varint
                    if let Some((_, new_pos)) = Self::protobuf_decode_varint(buf, pos) {
                        pos = new_pos;
                    } else {
                        break;
                    }
                }
                1 => {
                    // 64-bit fixed
                    if pos + 8 > buf.len() {
                        break;
                    }
                    pos += 8;
                }
                2 => {
                    // Length-delimited: string or nested message
                    let Some((len, new_pos)) = Self::protobuf_decode_varint(buf, pos) else {
                        break;
                    };
                    pos = new_pos;
                    let len = len as usize;
                    if pos + len > buf.len() {
                        break;
                    }
                    let chunk = &buf[pos..pos + len];
                    pos += len;

                    // Try as UTF-8 string that looks like a path
                    if let Ok(s) = std::str::from_utf8(chunk) {
                        let trimmed = s.trim();
                        if Self::looks_like_install_path(trimmed) {
                            out.push(PathBuf::from(trimmed));
                        }
                    }

                    // Recurse into nested message
                    if max_depth < 4 {
                        Self::battlenet_parse_protobuf_message(chunk, out, max_depth + 1);
                    }
                }
                5 => {
                    // 32-bit fixed
                    if pos + 4 > buf.len() {
                        break;
                    }
                    pos += 4;
                }
                _ => break,
            }
        }
    }

    /// Returns true if `s` looks like an absolute Windows install path
    /// (e.g. `C:/Program Files/...` or `D:/Games/...`).
    #[cfg(windows)]
    fn looks_like_install_path(s: &str) -> bool {
        if s.len() < 4 {
            return false;
        }
        let bytes = s.as_bytes();
        // Drive letter followed by `:\` or `:/`
        bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
    }

    #[cfg(windows)]
    fn detect_battlenet_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        for install_path in Self::battlenet_product_install_paths() {
            if let Some(mut game) = self.try_match_known_game_in_folder(
                &install_path,
                GameLauncher::Battlenet,
                None,
                None,
            ) {
                game.details
                    .environment
                    .insert("battlenet".into(), "product.db".into());
                out.push(game);
            }
        }
        out
    }

    #[cfg(any(windows, target_os = "linux"))]
    fn collect_json_files_recursive(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
        if depth > 10 {
            return;
        }
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for entry in rd.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                GameDetector::collect_json_files_recursive(&p, out, depth + 1);
            } else if p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
            {
                out.push(p);
            }
        }
    }

    #[cfg(windows)]
    fn amazon_install_paths_from_json() -> Vec<PathBuf> {
        let base = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
            .join("Amazon Games")
            .join("Data");
        if !base.is_dir() {
            return Vec::new();
        }
        let mut jsons = Vec::new();
        GameDetector::collect_json_files_recursive(&base, &mut jsons, 0);
        let mut paths = Vec::new();
        for jf in jsons {
            let Ok(txt) = fs::read_to_string(&jf) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else {
                continue;
            };
            for key in [
                "installDirectory",
                "installationRoot",
                "gameRoot",
                "InstallLocation",
                "installLocation",
            ] {
                if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
                    let pb = PathBuf::from(s.trim());
                    if pb.is_dir() {
                        paths.push(pb);
                        break;
                    }
                }
            }
        }
        paths
    }

    #[cfg(windows)]
    fn detect_amazon_games_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        for install_path in Self::amazon_install_paths_from_json() {
            if let Some(mut game) =
                self.try_match_known_game_in_folder(&install_path, GameLauncher::Amazon, None, None)
            {
                game.details
                    .environment
                    .insert("amazon".into(), "json_manifest".into());
                out.push(game);
            }
        }
        out
    }

    #[cfg(windows)]
    fn gog_galaxy_install_paths() -> Vec<PathBuf> {
        let db = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
            .join("GOG.com")
            .join("Galaxy")
            .join("storage")
            .join("galaxy-2.0.db");
        if !db.is_file() {
            return Vec::new();
        }
        let Ok(conn) = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
            return Vec::new();
        };
        let queries = [
            "SELECT installationPath FROM DbGame WHERE installationPath IS NOT NULL AND length(trim(installationPath)) > 0",
            "SELECT installationPath FROM DbGame",
        ];
        for q in queries {
            if let Ok(mut stmt) = conn.prepare(q) {
                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                    let paths: Vec<PathBuf> =
                        rows.filter_map(|r| r.ok()).map(PathBuf::from).collect();
                    if !paths.is_empty() {
                        return paths;
                    }
                }
            }
        }
        Vec::new()
    }

    /// Дополнительные пути из локальной БД Galaxy (после реестра GOG).
    #[cfg(windows)]
    fn detect_gog_galaxy_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        let mut out = Vec::new();
        for install_path in Self::gog_galaxy_install_paths() {
            if let Some(mut game) =
                self.try_match_known_game_in_folder(&install_path, GameLauncher::Gog, None, None)
            {
                game.details
                    .environment
                    .insert("gog_source".into(), "galaxy_db".into());
                out.push(game);
            }
        }
        out
    }

    /// Best-effort: пути из `Uninstall` в реестре (win32-установки вне Steam).
    #[cfg(windows)]
    fn detect_microsoft_store_windows<F, E>(&self, _on_progress: &F, _on_error: &E) -> Vec<Game>
    where
        F: Fn(DetectionProgress),
        E: Fn(GameDetectionError),
    {
        /// Registry entries from these publishers belong to desktop launchers
        /// (Battle.net, EA App, Ubisoft, etc.) and should not be tagged as
        /// Microsoft Store / UWP.
        const KNOWN_LAUNCHER_PUBLISHERS: &[&str] = &[
            "blizzard entertainment",
            "electronic arts",
            "ubisoft",
            "riot games",
            "epic games",
            "valve",
            "hi-rez studios",
            "wargaming",
        ];

        let mut out = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        for uninstall_path in [
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ] {
            let Ok(uninstall) = hklm.open_subkey(uninstall_path) else {
                continue;
            };
            for (n, key_name) in uninstall.enum_keys().filter_map(|e| e.ok()).enumerate() {
                if n > 150 {
                    break;
                }
                let Ok(sub) = uninstall.open_subkey(&key_name) else {
                    continue;
                };

                // Skip entries from known desktop launchers — they are handled
                // by their own detectors (Battle.net, EA, Ubisoft …).
                let publisher: Option<String> = sub.get_value("Publisher").ok();
                if let Some(pub_name) = publisher {
                    let pub_lower = pub_name.to_lowercase();
                    if KNOWN_LAUNCHER_PUBLISHERS
                        .iter()
                        .any(|p| pub_lower.contains(p))
                    {
                        continue;
                    }
                }

                let install_loc: Option<String> = sub.get_value("InstallLocation").ok();
                let Some(loc) = install_loc else {
                    continue;
                };
                let loc = loc.trim();
                if loc.len() < 4 {
                    continue;
                }
                let install_path = PathBuf::from(loc);
                if !install_path.is_dir() {
                    continue;
                }
                let lower = loc.to_lowercase();
                if lower.contains("windows\\system32")
                    || lower.contains("program files\\windowsapps")
                {
                    continue;
                }
                if let Some(mut game) = self.try_match_known_game_in_folder(
                    &install_path,
                    GameLauncher::MicrosoftStore,
                    None,
                    None,
                ) {
                    game.details
                        .environment
                        .insert("uninstall_registry_key".into(), key_name.clone());
                    out.push(game);
                }
            }
        }
        out
    }

    fn find_game_by_app_id(&self, app_id: u32) -> Option<&GameDefinition> {
        KNOWN_GAMES.iter().find(|g| g.steam_app_id == app_id)
    }

    fn list_exe_files_in_dir(&self, dir: &Path) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    #[cfg(windows)]
                    let ok = path.extension().map(|e| e == "exe").unwrap_or(false);
                    #[cfg(not(windows))]
                    let ok = {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        ext.eq_ignore_ascii_case("exe")
                            || ext.is_empty()
                            || self.is_unix_executable(&path)
                    };
                    if ok {
                        if let Some(name) = path.file_name() {
                            out.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        out.sort_by_key(|a| a.to_lowercase());
        out
    }

    #[cfg(unix)]
    fn is_unix_executable(&self, path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    #[allow(dead_code)] // only used from list_exe_files_in_dir on Unix
    fn is_unix_executable(&self, _path: &Path) -> bool {
        false
    }

    /// Если ни один путь из `game_def.executables` не существует, пробуем угадать .exe в корне
    /// (один файл в папке, или имя содержит id игры / ключевое слово вроде "stardew").
    fn fallback_exe_for_known_game(&self, dir: &Path, game_def: &GameDefinition) -> Option<String> {
        let exes = self.list_exe_files_in_dir(dir);
        if exes.is_empty() {
            return None;
        }
        let id = game_def.id.to_lowercase();
        let name_lower = game_def.name.to_lowercase();

        // Check if the directory path contains the game name or id
        let dir_str = dir.to_string_lossy().to_lowercase();
        let folder_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let path_matches = dir_str.contains(&name_lower)
            || folder_name.contains(&name_lower)
            || folder_name.contains(&id);

        if !path_matches {
            return None;
        }

        // If path matches, return the first exe or one matching the id
        for e in &exes {
            let stem = Path::new(e)
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            // stem содержит id: "SkyrimSE" ← "skyrim" ✓
            // stem == id:         точное совпадение ✓
            if stem.contains(&id) || stem == id {
                return Some(e.clone());
            }
        }

        // Return first exe if folder name matches
        if exes.len() == 1 {
            return Some(exes[0].clone());
        }

        None
    }

    fn find_executable_in_dir(&self, dir: &Path) -> Option<String> {
        self.list_exe_files_in_dir(dir)
            .into_iter()
            .next()
            .or_else(|| self.find_executable_recursive(dir, 0))
    }

    fn find_executable_recursive(&self, dir: &Path, depth: usize) -> Option<String> {
        if depth > 5 {
            return None;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in &entries {
                let path = entry.path();
                if path.is_file() {
                    #[cfg(windows)]
                    let is_exe = path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("exe"))
                        .unwrap_or(false);
                    #[cfg(not(windows))]
                    let is_exe = {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        ext.eq_ignore_ascii_case("exe")
                            || ext.is_empty()
                            || self.is_unix_executable(&path)
                    };
                    if is_exe {
                        if let Ok(rel) = path.strip_prefix(dir) {
                            return Some(rel.to_string_lossy().to_string());
                        }
                    }
                }
            }

            for entry in &entries {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(exe) = self.find_executable_recursive(&path, depth + 1) {
                        return Some(exe);
                    }
                }
            }
        }
        None
    }

    fn get_game_name_from_manifest(&self, manifest_path: &Path) -> Option<String> {
        fs::read_to_string(manifest_path)
            .ok()
            .and_then(|c| steam_parse::parse_acf_field(&c, "name"))
    }

    #[cfg(windows)]
    fn get_steam_library_folders(&self, steam_path: &Path) -> Vec<PathBuf> {
        let mut folders = Vec::new();
        let library_file = steam_path.join("steamapps").join("libraryfolders.vdf");

        if library_file.exists() {
            if let Ok(content) = fs::read_to_string(&library_file) {
                folders.extend(steam_parse::parse_libraryfolders_vdf(&content));
            }
        }

        let mut seen = HashSet::new();
        let mut unique = Vec::new();
        for p in folders {
            let key = p.to_string_lossy().to_lowercase();
            if seen.insert(key) {
                unique.push(p);
            }
        }
        unique
    }

    fn parse_app_manifest(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .and_then(|c| steam_parse::parse_acf_field(&c, "installdir"))
    }

    #[allow(dead_code)]
    pub fn manual_register_game(&self, id: &str, name: &str, path: &Path) -> Option<Game> {
        if let Some(game_def) = KNOWN_GAMES.iter().find(|g| g.id == id) {
            for exe in game_def.executables {
                if path.join(exe).exists() {
                    let support_path = if game_def.support_path_suffix.is_empty() {
                        path.to_path_buf()
                    } else {
                        path.join(game_def.support_path_suffix)
                    };

                    return Some(Game {
                        id: game_def.id.to_string(),
                        name: game_def.name.to_string(),
                        install_path: path.to_path_buf(),
                        support_path,
                        install_path_missing: false,
                        launcher: GameLauncher::Manual,
                        extension_id: None,
                        supported_mod_types: game_def
                            .supported_mod_types
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        merge_mods: game_def.merge_mods,
                        mod_support: game_def.mod_support.clone(),
                        details: GameDetails {
                            steam_app_id: None,
                            gog_id: None,
                            epic_app_id: None,
                            logo: None,
                            required_files: vec![exe.to_string()],
                            environment: std::collections::HashMap::new(),
                        },
                        created_at: chrono::Utc::now().to_rfc3339(),
                        updated_at: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        if path.exists() && path.is_dir() {
            return Some(Game {
                id: id.to_string(),
                name: name.to_string(),
                install_path: path.to_path_buf(),
                support_path: path.to_path_buf(),
                install_path_missing: false,
                launcher: GameLauncher::Manual,
                extension_id: None,
                supported_mod_types: vec!["simple".to_string()],
                merge_mods: false,
                mod_support: ModSupportLevel::None,
                details: GameDetails::default(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        None
    }
}

/// Парсинг фрагментов Steam VDF / appmanifest `.acf` (без полноценного VDF-парсера).
mod steam_parse {
    use std::path::PathBuf;

    /// Извлечь значение между кавычками из строки, игнорируя экранированные кавычки.
    fn extract_quoted_values(line: &str) -> Vec<String> {
        let mut vals = Vec::new();
        let mut buf = String::new();
        let mut in_quote = false;
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' && chars.peek() == Some(&'"') {
                // escaped quote — пропускаем '\', пишем '"' как обычный символ
                buf.push('"');
                chars.next();
            } else if ch == '"' {
                if in_quote {
                    vals.push(buf.clone());
                    buf.clear();
                }
                in_quote = !in_quote;
            } else if in_quote {
                buf.push(ch);
            }
        }
        vals
    }

    /// Нормализовать разделители в Windows-путях («\\\\» → «\\», «/» → «\\»).
    fn normalize_win_path(raw: &str) -> PathBuf {
        let p = raw.replace("\\\\", "\\").replace('/', "\\");
        PathBuf::from(p)
    }

    pub fn parse_libraryfolders_vdf(content: &str) -> Vec<PathBuf> {
        let mut folders = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            let lowered = trimmed.to_lowercase();
            if !lowered.contains("\"path\"") {
                continue;
            }
            let vals = extract_quoted_values(trimmed);
            // vals[0] = "path", vals[1] = значение пути
            if vals.len() >= 2 {
                let p = normalize_win_path(&vals[1]);
                if !p.as_os_str().is_empty() {
                    folders.push(p);
                }
            }
        }
        folders
    }

    /// Поля `installdir`, `name` и т.д. в `.acf` / фрагментах VDF.
    ///
    /// Ключи сравниваются case-insensitive.  Значение берётся **до** закрывающей
    /// кавычки, поэтому кавычки внутри значения не сломают парсинг.
    pub fn parse_acf_field(content: &str, field: &str) -> Option<String> {
        let needle = format!("\"{}\"", field);
        let needle_lower = needle.to_lowercase();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.to_lowercase().contains(&needle_lower) {
                return extract_quoted_values(trimmed).into_iter().nth(1);
            }
        }
        None
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn libraryfolders_extracts_paths() {
            let vdf = r#""libraryfolders"
{
	"0"
	{
		"path"		"D:\\\\SteamLibrary"
	}
}"#;
            let paths = parse_libraryfolders_vdf(vdf);
            assert_eq!(paths.len(), 1);
            assert!(paths[0].to_string_lossy().contains("SteamLibrary"));
        }

        #[test]
        fn libraryfolders_case_insensitive() {
            let vdf = r#""0"
	{
		"Path"		"D:\\\\Games"
	}"#;
            let paths = parse_libraryfolders_vdf(vdf);
            assert_eq!(paths.len(), 1);
            assert!(paths[0].to_string_lossy().contains("Games"));
        }

        #[test]
        fn libraryfolders_forward_slash() {
            let vdf = r#""0"
	{
		"path"		"D:/SteamLibrary"
	}"#;
            let paths = parse_libraryfolders_vdf(vdf);
            assert_eq!(paths.len(), 1);
            assert!(paths[0].to_string_lossy().contains("SteamLibrary"));
        }

        #[test]
        fn acf_installdir() {
            let acf = r#""AppState"
{
	"installdir"		"My Game Dir"
}"#;
            assert_eq!(
                parse_acf_field(acf, "installdir").as_deref(),
                Some("My Game Dir")
            );
        }

        #[test]
        fn acf_name() {
            let acf = r#"	"name"		"Half-Life 2""#;
            assert_eq!(parse_acf_field(acf, "name").as_deref(), Some("Half-Life 2"));
        }

        #[test]
        fn acf_name_case_insensitive() {
            let acf = r#"	"Name"		"Half-Life 2""#;
            assert_eq!(parse_acf_field(acf, "name").as_deref(), Some("Half-Life 2"));
        }
    }
}

impl Default for GameDetector {
    fn default() -> Self {
        Self::new()
    }
}
