# Module: Game Detector

## Responsibility

Обнаружение установленных игр на компьютере пользователя.

## Implementation status

| Area | Status | Notes |
|------|--------|--------|
| Steam (Windows) | **Implemented** | `HKCU\Software\Valve\Steam` → `SteamPath`, `appmanifest_*.acf`, `libraryfolders.vdf`, `KNOWN_GAMES`, generic `steam_<appid>` |
| Steam (Linux) | **Implemented** | `~/.steam/steam` and `~/.local/share/Steam` roots (same manifest scan) |
| GOG (Windows) | **Implemented** | `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games` (and non-WOW64 fallback): match install dir against `KNOWN_GAMES` executables |
| GOG Galaxy (Windows) | **Implemented** | `%LOCALAPPDATA%\GOG.com\Galaxy\storage\galaxy-2.0.db` — `DbGame.installationPath`; дубликаты по `Game.id` отсекаются после реестра |
| Epic (Windows) | **Implemented** | `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` JSON → `InstallLocation`, same matching |
| EA app / Origin (Windows) | **Implemented** | `HKLM\SOFTWARE\...\EA Games\<key>\Install Dir` (или `InstallDir`) |
| Ubisoft Connect (Windows) | **Implemented** | `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>\InstallDir` |
| Battle.net (Windows) | **Implemented** | `%ProgramData%\Battle.net\Agent\product.db` (SQLite, таблица `ProductInstall`) |
| Amazon Games (Windows) | **Implemented** | `%LOCALAPPDATA%\Amazon Games\Data\**\*.json` — поля вроде `installDirectory` / `InstallLocation` |
| Xbox / PC Game Pass | **Implemented (best-effort)** | `HKLM\SOFTWARE\Microsoft\Windows\XboxGameExport` |
| Microsoft Store (Windows) | **Implemented (best-effort)** | Дополнительно: `HKLM\...\Uninstall` — `InstallLocation` для путей, не попавших в другие источники (без полного доступа к WindowsApps/UWP API) |
| Heroic (Linux) | **Implemented** | `~/.config/heroic/**/*.json` — абсолютные пути каталогов в JSON; `launcher` по полям `runner` / `store` |
| Lutris (Linux) | **Implemented** | `~/.local/share/lutris/games/*.yml` — `game_path:` / `exe:`; `runner:` для угадывания лаунчера |
| Manual folder (`scan_custom_path`) | **Implemented** | Known games by exe (через индекс путей); optional generic game if a single `.exe` is found and no known match |
| Extension manifests / `extension_id` | **Planned** | `extension_id` remains `None` until extension system exists |
| GOG Galaxy API / Steam Web API | **Not planned (v1)** | Local filesystem and registry only |

## Target architecture (roadmap)

Покрытие основных локальных источников (в духе multi-store менеджеров вроде Vortex); код Vortex (GPL) не копируется — только собственная реализация.

1. Steam registry / local manifests (done)
2. GOG registry + Galaxy DB (done)
3. Epic Games Launcher manifests (done)
4. EA, Ubisoft, Battle.net, Amazon (done)
5. Xbox / Microsoft Store partial (done)
6. Linux Heroic / Lutris (done)
7. Manual path detection (done)

## Supported Platforms (reference)

| Platform | Typical detection method |
|----------|-------------------------|
| Steam | `SteamPath`, `libraryfolders.vdf`, `appmanifest_*.acf` |
| GOG | Registry, Galaxy `galaxy-2.0.db` |
| Epic | `Manifests\*.item` (JSON) |
| EA / Origin | Registry `EA Games` |
| Ubisoft | Registry `Ubisoft\Launcher\Installs` |
| Battle.net | `product.db` (SQLite) |
| Amazon | JSON под `Amazon Games\Data` |
| Xbox/Game Pass | Registry `XboxGameExport` |
| Microsoft Store | Best-effort: `Uninstall` + InstallLocation |
| Heroic / Lutris | JSON / YAML под `~/.config`, `~/.local/share` |
| Manual | User-selected folder |

## Data Model

See [MODELS.md](../MODELS.md) — `Game`, `GameDetails`, `GameLauncher` in `src-tauri/src/models.rs`.

Дополнительные метаданные источника (ключи реестра, `heroic`/`lutris` и т.д.) могут попадать в `GameDetails.environment`.

## Detection flow (actual pipeline)

`GameDetector::detect_games` runs, in order:

**Windows**

1. **Steam** — library roots → manifests → `KNOWN_GAMES` or generic `steam_<appid>` (with skip list for tool depots)
2. **GOG** — registry → install paths → known-game exe match
3. **GOG Galaxy** — SQLite paths → same match (`launcher`: `gog`, `environment.gog_source`: `galaxy_db`)
4. **Epic** — manifest JSON → `InstallLocation` → match
5. **EA / Origin** — EA Games registry → match (`launcher`: `origin`)
6. **Ubisoft** — Ubisoft Connect registry → match
7. **Battle.net** — `product.db` install paths → match
8. **Amazon** — JSON manifests → match
9. **Xbox** — `XboxGameExport` registry → match
10. **Microsoft Store (best-effort)** — `Uninstall` hive scan (cap on keys) → match

**Linux**

1. **Steam** — same manifest logic as Windows
2. **Heroic** — JSON scan under `~/.config/heroic`
3. **Lutris** — `.yml` game configs

Non-Windows: нет реестра Windows-лаунчеров; Epic/GOG на Linux часто через Heroic/Lutris, не через отдельные нативные сканеры.

Results are **deduplicated by `Game.id`** (first wins).

## Windows / Steam (details)

**Module**: `src-tauri/src/services/game_detector.rs`

| Step | Behavior |
|------|----------|
| Steam root | Registry `HKCU\Software\Valve\Steam`, value `SteamPath` |
| Library roots | `steamapps` under Steam root + paths from `steamapps/libraryfolders.vdf` (парсер в модуле `steam_parse`) |
| Installed apps | Parse `steamapps/appmanifest_<appid>.acf` → `installdir` |
| Skip list | Tool/redistributable App IDs (e.g. Steamworks Shared) are skipped for generic registration |
| Known games | `KNOWN_GAMES` — индекс относительных путей exe → ускорение сопоставления с каталогом |
| Unknown Steam App ID | Registered as `steam_<appid>` if an executable is found and App ID not skipped |
| Known game, exe mismatch | Fallback: single `.exe` in folder, or heuristics (e.g. Stardew) |

Events: `game_detection_started`, `game_detection_progress`, `game_detected`, `game_detection_error`, `game_detection_completed`.

## Public API (Rust)

The Tauri command calls `GameDetector::detect_games(&detector, on_progress, on_error)` — a **synchronous** method taking progress/error callbacks. Все лаунчеры вызываются внутри `detect_games`; отдельных публичных команд на каждый магазин нет.

## Game Extension Manifest (future)

JSON manifests for per-game extensions remain a **planned** integration; see historical example in git history or MODULE_SPECS.

## Key Interactions

| Module | Interaction |
|--------|----------------|
| `database` | `insert_or_update_game` after detection |
| `mod-installer` / `deploy-manager` | Consume `Game.support_path` (future) |

## Notes

- Some Bethesda titles use `merge_mods: true` for deployment strategy.
- Mod paths vary: `Data/`, `Mods/`, etc. — encoded in `KNOWN_GAMES.support_path_suffix`.
- Полный перечень установок Microsoft Store / защищённых папок WindowsApps без WinRT API недоступен — см. best-effort ветку `Uninstall`.
