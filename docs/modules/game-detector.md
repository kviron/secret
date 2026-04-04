# Module: Game Detector

## Responsibility

Обнаружение установленных игр на компьютере пользователя.

## Pantheon Comparison

Pantheon uses `gamemode_management` extension with `GameStore` integration for game discovery. Games are discovered via:
1. Steam registry / Steam API
2. GOG registry / Galaxy API  
3. Epic Games launcher database
4. Xbox Game Pass / Microsoft Store
5. Manual path detection

## Supported Platforms

| Platform | Detection Method | Pantheon Equivalent |
|----------|-----------------|-------------------|
| Steam | Steam registry (`HKCU\Software\Valve\Steam\Apps`), Steam API | `gameinfo-steam` |
| GOG | GOG registry (`HKLM\SOFTWARE\GOG.com\Games`), Galaxy API | `gamestore-gog` |
| Epic | Epic manifest files in AppData | `gamestore-epic` |
| Xbox/Game Pass | Windows registry, Xbox app detection | `gamestore-xbox` |
| Manual | User-specified path | Direct path input |

## Data Model

```rust
struct Game {
    id: String,                    // Unique identifier (e.g., "skyrim", "fallout4")
    name: String,                  // Display name
    install_path: PathBuf,         // Game executable location
    support_path: PathBuf,         // Mod directory (Data folder, Mods folder, etc.)
    launcher: Launcher,           // Steam/GOG/Epic/Xbox/Manual
    extension_id: String,         // Which extension handles this game
    supported_mod_types: Vec<String>,  // e.g., ["fomod", "bepinex", "bsa"]
    merge_mods: bool,             // Games like Skyrim that merge mod content
}

enum Launcher {
    Steam,
    GOG,
    Epic,
    Xbox,
    Manual,
}
```

## Detection Flow

```
1. Scan Steam registry
       │
       ▼
   Found Steam IDs ──────► Match against known games DB
       │                         │
       ▼                         │
2. Scan GOG registry              │
       │                         │
       ▼                         │
   Found GOG IDs ───────────────►│
       │                         │
       ▼                         ▼
3. Scan Epic manifests ◄──────────┘
       │
       ▼
4. Scan Xbox/Game Pass ◄──────────┘
       │
       ▼
5. Combine all discoveries
       │
       ▼
6. Return list of detected games
```

## Pantheon Implementation

```rust
// Game discovery command
#[tauri::command]
pub async fn detect_games() -> Result<Vec<Game>, String> {
    let mut games = Vec::new();
    
    // Steam detection
    games.extend(detect_steam_games().await?);
    
    // GOG detection  
    games.extend(detect_gog_games().await?);
    
    // Epic detection
    games.extend(detect_epic_games().await?);
    
    // Xbox detection
    games.extend(detect_xbox_games().await?);
    
    Ok(games)
}

// Platform-specific detection
async fn detect_steam_games() -> Result<Vec<Game>, String> { ... }
async fn detect_gog_games() -> Result<Vec<Game>, String> { ... }
async fn detect_epic_games() -> Result<Vec<Game>, String> { ... }
```

## Game Extension Manifest (JSON)

```json
{
    "id": "game-skyrimse",
    "name": "The Elder Scrolls V: Skyrim Special Edition",
    "extension_id": "gamebryo-skyrim",
    "steam": {
        "app_id": "489830",
        "branch": "public"
    },
    "gog": {
        "game_id": "1458055853"
    },
    "mod_types": [
        { "id": "fomod", "priority": 100 },
        { "id": "bsa", "priority": 90 },
        { "id": "bepinex", "priority": 80 }
    ],
    "mod_paths": {
        "default": "Data",
        "modorganizer": "Mods"
    },
    "merge_mods": true,
    "loot_metadata": "skyrim.yaml"
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-extension` | Loads game-specific detection logic |
| `database` | Stores discovered games |
| `mod-installer` | Uses game ID to determine mod staging path |
| `deploy-manager` | Uses game support_path for deployment target |
| `load-order-manager` | Uses game ID to load LOOT metadata |

## Notes

- Game detection should be expandable via extensions
- Each game should have a known "extension" that handles game-specific logic
- Some games (Bethesda) have `merge_mods: true` which affects deployment strategy
- Mod paths vary by game: `Data/` for Bethesda, `Mods/` for MO2-style, root for some games
