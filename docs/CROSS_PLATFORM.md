# Cross-Platform Support

## Overview

Pantheon разрабатывается с приоритетом на Windows, но архитектура предусматривает поддержку Linux (включая SteamOS) и macOS. Данный документ описывает архитектурные решения для кроссплатформенной совместимости.

## Platform Support Matrix

| Platform | Priority | Status | Notes |
|----------|----------|--------|-------|
| Windows 10/11 | **Primary** | Phase 1 | Full support, all features |
| Linux (SteamOS) | Secondary | Phase 3 | Flatpak, controller-first UI |
| Linux (Desktop) | Secondary | Phase 3 | Debian, Fedora, Arch |
| macOS | Future | Post-launch | Limited game support |

## Layered Architecture: Core + Platform Adapter

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│              (Platform-agnostic core logic)                   │
│                                                              │
│  Mod Installer • Deploy Manager • Download Manager • UI      │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                  Platform Adapter Layer                      │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Windows    │  │    Linux     │  │    macOS     │      │
│  │  Adapter     │  │   Adapter    │  │   Adapter    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                              │
│  • File paths    • File paths    • File paths               │
│  • Registry      • Config files  • Plist files              │
│  • Symlinks      • Symlinks      • Aliases                  │
│  • Game detect   • Game detect   • Game detect              │
│  • Permissions   • Permissions   • Permissions              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Platform APIs                             │
│                                                              │
│  Windows: winreg, std::os::windows, CreateSymbolicLink      │
│  Linux:   XDG dirs, std::os::unix, std::os::unix::fs        │
│  macOS:   CoreFoundation, std::os::macos, alias resolution   │
└─────────────────────────────────────────────────────────────┘
```

## Platform Adapter Trait

```rust
pub trait PlatformAdapter: Send + Sync {
    // File system
    fn create_symlink(&self, source: &Path, target: &Path) -> Result<(), String>;
    fn create_hardlink(&self, source: &Path, target: &Path) -> Result<(), String>;
    fn is_symlink(&self, path: &Path) -> bool;
    fn read_link(&self, path: &Path) -> Result<PathBuf, String>;
    
    // Game detection
    fn detect_games(&self) -> Result<Vec<GameDiscovery>, String>;
    fn get_steam_path(&self) -> Option<PathBuf>;
    fn get_gog_path(&self) -> Option<PathBuf>;
    fn get_epic_path(&self) -> Option<PathBuf>;
    
    // Paths
    fn get_app_data_path(&self) -> PathBuf;
    fn get_config_path(&self) -> PathBuf;
    fn get_cache_path(&self) -> PathBuf;
    
    // Permissions
    fn requires_admin(&self) -> bool;
    fn check_write_access(&self, path: &Path) -> bool;
    
    // Process
    fn launch_process(
        &self, 
        executable: &Path, 
        args: &[String],
        working_dir: &Path,
    ) -> Result<Child, String>;
    
    // Platform info
    fn platform_name(&self) -> &str;
    fn is_steam_deck(&self) -> bool;
}
```

## Platform-Specific Considerations

### Windows

| Area | Details |
|------|---------|
| Symlinks | `CreateSymbolicLinkW` — may require admin or Developer Mode |
| Game Detection | Windows Registry (`winreg` crate) |
| Paths | `%APPDATA%`, `%LOCALAPPDATA%`, `%PROGRAMFILES%` |
| Permissions | UAC elevation for admin operations |
| Archive | Native support for zip, 7z via Rust crates |

### Linux / SteamOS

| Area | Details |
|------|---------|
| Symlinks | `std::os::unix::fs::symlink` — no admin needed |
| Game Detection | Steam library folders (including Flatpak/Snap), Lutris (+ Flatpak), Heroic (+ Flatpak); `libraryfolders.vdf` parsed for additional library paths |
| Paths | XDG Base Directory spec (`$XDG_DATA_HOME`, `$XDG_CONFIG_HOME`) |
| Permissions | Standard Unix permissions, no UAC |
| Proton | Wine prefix detection for Windows games |
| Packaging | Flatpak (primary), AppImage, native deb/rpm |
| UI | Controller-first design for Steam Deck |

### Linux Game Discovery

```rust
// Steam library folders (including Flatpak/Snap)
const STEAM_PATHS: &[&str] = &[
    "~/.steam/steam",
    "~/.local/share/Steam",
    "~/.steam/root",
    "~/.var/app/com.valvesoftware.Steam/.local/share/Steam", // Flatpak
    "~/snap/steam/common/.local/share/Steam",                // Snap
];

// Lutris game library (including Flatpak)
const LUTRIS_PATHS: &[&str] = &[
    "~/.local/share/lutris/games",
    "~/.var/app/net.lutris.Lutris/data/lutris/games",  // Flatpak
    "~/Games",
];

// Heroic Games Launcher (Epic/GOG on Linux, including Flatpak)
const HEROIC_PATHS: &[&str] = &[
    "~/.config/heroic",
    "~/.var/app/com.heroic.games.launcher/config/heroic",  // Flatpak
];
```

### Proton Compatibility

```rust
struct ProtonPrefix {
    steam_app_id: String,
    prefix_path: PathBuf,        // compatdata/{app_id}/pfx
    drive_c: PathBuf,            // prefix_path/drive_c
    game_path: PathBuf,          // Within Steam library
    proton_version: String,
}

// When running under Proton:
// - Windows paths are translated to prefix paths
// - Symlinks work within the prefix
// - External tools (LOOT, xEdit) may not work
```

## Steam Deck / Controller-First UI

```
┌─────────────────────────────────────────────────────────────┐
│  Pantheon — Steam Deck Mode                                   │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  🎮  My Games                                        │  │
│  │                                                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │  Skyrim SE  │  │  Fallout 4  │  │  Stardew    │  │  │
│  │  │  142 mods   │  │  89 mods    │  │  Valley     │  │  │
│  │  │  ✅ Ready   │  │  ⚠️ Update  │  │  12 mods    │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  │                                                       │  │
│  │  [▶ Play]  [📦 Mods]  [⚙️ Settings]                  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  Navigation: D-Pad / Left Stick  │  Select: A / ✕           │
│  Back: B / ○                     │  Menu: ≡ / Options       │
└─────────────────────────────────────────────────────────────┘
```

## Packaging Strategies

| Format | Target | Sandboxing | Auto-Update | Notes |
|--------|--------|------------|-------------|-------|
| **Flatpak** | All Linux | Yes (sandbox) | Flathub | Primary Linux distribution |
| **AppImage** | All Linux | No | AppImageUpdate | Single-file, portable |
| **MSIX/AppX** | Windows 10/11 | Yes | Microsoft Store | Optional |
| **Installer (NSIS)** | Windows | No | In-app | Primary Windows distribution |
| **Portable ZIP** | Windows | No | In-app | For advanced users |

## File Path Abstraction

```rust
// Platform-agnostic path resolution
pub fn resolve_game_path(game: &Game, platform: &dyn PlatformAdapter) -> GamePaths {
    match platform.platform_name() {
        "windows" => GamePaths {
            install: game.install_path.clone(),
            data: game.install_path.join("Data"),
            plugins: game.install_path.join("Data"),
            saves: get_windows_saves_path(game),
            ini: get_windows_ini_path(game),
        },
        "linux" => {
            if let Some(prefix) = detect_proton_prefix(game) {
                // Running under Proton
                GamePaths {
                    install: prefix.drive_c.join(&game.install_path),
                    data: prefix.drive_c.join(&game.install_path).join("Data"),
                    plugins: prefix.drive_c.join(&game.install_path).join("Data"),
                    saves: get_linux_saves_path(game),
                    ini: get_linux_ini_path(game),
                }
            } else {
                // Native Linux game
                GamePaths {
                    install: game.install_path.clone(),
                    data: game.install_path.join("data"),
                    plugins: game.install_path.join("plugins"),
                    saves: get_native_linux_saves(game),
                    ini: get_native_linux_ini(game),
                }
            }
        },
        _ => panic!("Unsupported platform"),
    }
}
```

## CI/CD Matrix

```yaml
# .github/workflows/ci.yml (conceptual)
build:
  strategy:
    matrix:
      os:
        - windows-latest
        - ubuntu-22.04
        - ubuntu-24.04
      target:
        - x86_64-pc-windows-msvc
        - x86_64-unknown-linux-gnu
        - x86_64-unknown-linux-musl  # For AppImage

test:
  strategy:
    matrix:
      os: [windows-latest, ubuntu-22.04]
      # Test core logic, file operations, symlink creation
```

## Best Practices

1. **Core + Adapter pattern** — Keep platform-specific code in thin adapters
2. **Feature flags** — Use `#[cfg(target_os = "...")]` for conditional compilation
3. **Test on target platforms** — CI matrix should cover all supported platforms
4. **XDG compliance** — On Linux, follow XDG Base Directory specification
5. **Proton awareness** — Detect and handle Proton prefixes for Windows games on Linux
6. **Controller navigation** — All UI must be navigable with gamepad on Steam Deck
7. **Minimal permissions** — Request only necessary filesystem access
8. **Universal packaging** — Prefer Flatpak/AppImage over distro-specific packages

## Notes

- Linux support is Phase 3 priority — focus on Windows for MVP
- Proton compatibility is critical for SteamOS — most games run through Proton
- LOOT and xEdit tools are Windows-only — need alternatives on Linux
- Some mod formats (FOMOD) may need platform-specific handling
- Consider community-maintained adapters for less common distros
