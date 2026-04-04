# Module: Game Launcher

## Responsibility

Запуск игр с учётом установленных модов, мод-лоадеров и скрипт-экстендеров. Обеспечивает корректную инъекцию мод-лоадера в процесс игры, передачу аргументов запуска и проверку совместимости перед стартом.

## Loader Types

| Type | Hook Method | Use Case | Example |
|------|-------------|----------|---------|
| Proxy DLL | Library replacement + forwarding | DirectX/engine library hooks | dinput8.dll, winmm.dll |
| Bootstrap EXE | Wrapper process with parameters | Games with consistent CLI support | SKSE64 Loader |
| Script Extender | Runtime injection + API expansion | Script-heavy games | SKSE, F4SE, OBSE |
| Native Mod Support | Game's built-in mod system | Games with official mod support | Starfield, Fallout 4 |
| Root Builder | Symlink root files, deploy Data | Games needing root file mods | SSE with Root Builder |

## Data Model

```rust
struct GameLaunchConfig {
    game_id: String,
    executable: PathBuf,             // Game executable path
    working_directory: PathBuf,      // Working directory for launch
    launch_args: Vec<String>,        // Command line arguments
    loader: Option<ModLoader>,       // Mod loader to use
    environment_vars: HashMap<String, String>, // Custom env vars
    pre_launch_checks: Vec<PreLaunchCheck>,
    post_launch_actions: Vec<PostLaunchAction>,
}

struct ModLoader {
    id: String,                      // e.g., "skse64", "f4se", "wraper"
    name: String,
    executable: PathBuf,             // Loader executable
    version: String,                 // Loader version
    required_game_version: Option<String>, // Compatible game version
    loader_type: LoaderType,
    proxy_dll: Option<ProxyDllConfig>, // For proxy DLL loaders
}

enum LoaderType {
    ProxyDll,
    Bootstrap,
    ScriptExtender,
    Native,
}

struct ProxyDllConfig {
    original_name: String,           // Original DLL name (e.g., "dinput8.dll")
    proxy_name: String,              // Proxy DLL name
    backup_path: PathBuf,            // Where original is backed up
}

struct PreLaunchCheck {
    check_type: CheckType,
    description: String,
    is_blocking: bool,               // If true, prevents launch on failure
}

enum CheckType {
    LoaderVersionMatch,              // Loader version matches game version
    GameVersionCompatible,           // Game version is supported
    NoConflictingMods,               // No known conflicting mods enabled
    RequiredModsPresent,             // Required dependency mods are enabled
    DiskSpaceAvailable,              // Enough disk space for operation
    AnticheatDisabled,               // Anticheat not active (for single-player)
}

enum PostLaunchAction {
    OpenConsole,                     // Open game console
    ShowLog,                         // Show loader log
    MonitorProcess,                  // Monitor game process for crashes
}
```

## Launch Flow

```
1. User clicks "Play"
        │
        ▼
2. Pre-launch validation
        │
        ├──► Check loader compatibility with game version
        ├──► Check required mods are enabled
        ├──► Check for known conflicting mods
        ├──► Verify disk space
        └──► Check anticheat status
        │
        ▼
3. Resolve launch target
        │
        ├──► Has mod loader? ──► Launch via loader
        │         │
        │         ├──► Proxy DLL: Ensure proxy is in place
        │         ├──► Bootstrap: Build command line
        │         └──► Script Extender: Check runtime version
        │
        └──► No loader? ──► Launch game directly
        │
        ▼
4. Launch game
        │
        ├──► Set working directory
        ├──► Set environment variables
        ├──► Apply launch arguments
        └──► Start process
        │
        ▼
5. Post-launch monitoring
        │
        ├──► Read loader log (if available)
        ├──► Monitor process health
        └──► Report crashes/errors
        │
        ▼
6. On game exit
        │
        └──► Log session duration, any errors
```

## Loader Detection

```rust
// Detect installed mod loaders for a game
pub fn detect_loaders(game: &Game) -> Vec<ModLoader> {
    let mut loaders = Vec::new();
    
    // Check for SKSE (Skyrim)
    if let Some(skse) = detect_script_extender(
        &game.install_path,
        "skse64_loader.exe",
        "skse64_1_10_163.dll",
    ) {
        loaders.push(skse);
    }
    
    // Check for F4SE (Fallout 4)
    if let Some(f4se) = detect_script_extender(
        &game.install_path,
        "f4se_loader.exe",
        "f4se_1_10_163.dll",
    ) {
        loaders.push(f4se);
    }
    
    // Check for proxy DLLs
    loaders.extend(detect_proxy_dlls(&game.install_path));
    
    loaders
}
```

## Executable Management

```rust
struct GameExecutable {
    name: String,                    // Display name
    path: PathBuf,                   // Path to executable
    is_primary: bool,                // Default executable
    launch_args: Vec<String>,        // Executable-specific args
    description: Option<String>,     // e.g., "SKSE64", "Steam", "VR"
}

// Multiple executables per game (common for Bethesda games)
// - Game executable (normal launch)
// - SKSE/F4SE loader (modded launch)
// - VR executable (if applicable)
// - Creation Kit (modding tools)
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | Gets game install path and executable location |
| `profile-manager` | Uses profile-specific launch args and loader settings |
| `deploy-manager` | Ensures mods are deployed before launch |
| `mod-installer` | Installs loader files (script extenders) |
| `settings` | User preferences for default loader, launch args |

## API

```rust
#[tauri::command]
pub async fn launch_game(
    game_id: String,
    profile_id: Option<String>,
    executable_id: Option<String>,
) -> Result<(), String>;

#[tauri::command]
pub async fn get_launch_config(
    game_id: String,
) -> Result<GameLaunchConfig, String>;

#[tauri::command]
pub async fn update_launch_args(
    game_id: String,
    args: Vec<String>,
) -> Result<(), String>;

#[tauri::command]
pub async fn detect_loaders(
    game_id: String,
) -> Result<Vec<ModLoader>, String>;

#[tauri::command]
pub async fn install_loader(
    game_id: String,
    loader_id: String,
    archive_path: PathBuf,
) -> Result<(), String>;

#[tauri::command]
pub async fn get_executables(
    game_id: String,
) -> Result<Vec<GameExecutable>, String>;
```

## Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `game_launched` | `GameLaunchInfo` | Game process started |
| `game_exited` | `GameExitInfo` | Game process ended |
| `game_crashed` | `GameCrashInfo` | Game crashed with error |
| `loader_error` | `LoaderError` | Loader failed to start |

## Best Practices

1. **Reversibility** — Loader installation must be fully reversible; always backup originals
2. **Least invasive first** — Prefer parameterized launch over proxy DLL injection
3. **Version pinning** — Pin loader version to compatible game version
4. **Empty profile test** — Always validate loader works with empty profile first
5. **Log collection** — Capture and display loader logs for troubleshooting
6. **Process monitoring** — Track game process health and report crashes
7. **Clean baseline** — Verify game launches cleanly before adding mods

## Notes

- Proxy DLLs can collide with anticheat — detect and warn
- Script extenders require exact game version match
- Some storefronts (Steam) force their own launchers — handle working directory carefully
- Starfield and newer games use different mod loading mechanisms
- Linux/Proton may require different loader strategies (shell scripts, Proton prefixes)
