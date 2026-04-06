# Module: Game Detector

## Responsibility

Обнаружение установленных игр на компьютере пользователя.

## Implementation status

| Area | Status | Notes |
|------|--------|--------|
| Steam (Windows) | **Implemented** | `HKCU\Software\Valve\Steam` → `SteamPath`, `appmanifest_*.acf`, `libraryfolders.vdf` (case-insensitive, normalize paths), `KNOWN_GAMES`, generic `steam_<appid>`, skip list for tool depots |
| Steam (Linux) | **Implemented** | `~/.steam/steam`, `~/.local/share/Steam`, Flatpak (`~/.var/app/com.valvesoftware.Steam`), Snap (`~/snap/steam`); `libraryfolders.vdf` parsed for additional libraries |
| GOG (Windows) | **Implemented** | `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games` — priority: `path` > `exe` (parent dir) > `InstallDir`; quote trimming |
| GOG Galaxy (Windows) | **Implemented** | `%LOCALAPPDATA%\GOG.com\Galaxy\storage\galaxy-2.0.db` — `DbGame.installationPath`; dedup by `Game.id` |
| Epic (Windows) | **Implemented** | `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` JSON → `InstallLocation` |
| EA app / Origin (Windows) | **Implemented** | `HKLM\SOFTWARE\...\EA Games\<key>\Install Dir` + `HKCU\...\EA Desktop\<key>\InstallLocation` (EA Desktop app) |
| Ubisoft Connect (Windows) | **Implemented** | `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>\InstallDir` |
| Battle.net (Windows) | **Implemented** | `%ProgramData%\Battle.net\Agent\product.db` — protobuf (modern client) с fallback на SQLite (legacy); поддержка игр: Overwatch 2, Diablo IV, Diablo II: Resurrected, Diablo III, World of Warcraft, StarCraft II, Hearthstone, Warcraft III: Reforged |
| Amazon Games (Windows) | **Implemented** | `%LOCALAPPDATA%\Amazon Games\Data\**\*.json` — поля `installDirectory` / `InstallLocation` |
| Xbox / PC Game Pass | **Implemented (best-effort)** | `HKLM\SOFTWARE\Microsoft\Windows\XboxGameExport` |
| Microsoft Store (Windows) | **Implemented (best-effort)** | Дополнительно: `HKLM\...\Uninstall` — `InstallLocation` для путей, не попавших в другие источники (без полного доступа к WindowsApps/UWP API) |
| Heroic (Linux) | **Implemented** | `~/.config/heroic/**/*.json` + Flatpak (`~/.var/app/com.heroic.games.launcher/config/heroic`) |
| Lutris (Linux) | **Implemented** | `~/.local/share/lutris/games/*.yml` + Flatpak (`~/.var/app/net.lutris.Lutris/data/lutris/games`) |
| Manual folder (`scan_custom_path`) | **Implemented** | Known games by exe (через индекс путей); optional generic game if a single `.exe` is found and no known match |
| Extension manifests / `extension_id` | **Planned** | `extension_id` remains `None` until extension system exists |
| GOG Galaxy API / Steam Web API | **Not planned (v1)** | Local filesystem and registry only |

## Changelog (2026-04-06)

### Исправления

- **Battle.net product.db**: modern Battle.net client использует protobuf вместо SQLite для `product.db`. Добавлен protobuf-парсер с рекурсивным извлечением путей; fallback на SQLite для legacy-клиента
- **Microsoft Store detector**: фильтрация записей от известных лаунчеров (Blizzard, EA, Ubisoft, Riot, Epic, Valve и др.) по ключу `Publisher` в реестре — исключает false-positive классификацию игр Battle.net как «Microsoft Store»

### Добавленные игры

- **Blizzard / Battle.net**: добавлены игры в `KNOWN_GAMES` — Overwatch 2, Diablo IV, Diablo II: Resurrected, Diablo III, World of Warcraft (с поддержкой модов через AddOns), StarCraft II, Hearthstone, Warcraft III: Reforged
- **Overwatch 2**: исполняемый файл `Overwatch Launcher.exe` в корне установки

### Исправленные критичные проблемы

- **VDF парсер**: case-insensitive ключи (`"path"`, `"Path"`), надёжное извлечение значений между кавычками, нормализация путей (`/` → `\\`, `\\\\` → `\\`), fallback через `path` реестра GOG вместо `exe` parent
- **GOG детекция**: приоритет `path` > `exe` > `InstallDir`; обрезка кавычек в путях из реестра
- **Linux Steam**: парсинг `libraryfolders.vdf` для дополнительных библиотек; поддержка Flatpak и Snap путей; предупреждение при отсутствии `$HOME`
- **Рекурсивный поиск exe**: depth limit=5 (защита от циклических симлинков); поддержка Linux-файлов без `.exe` (Unix executable bits)
- **Bidirectional substring matching**: `stem.contains(&id)` вместо `stem.contains(&id) || id.contains(&stem)` — убраны false positives (например `skyrim` не матчит `skyrimse`)
- **Опечатка**: `"foomad"` → `"fomod"` в `supported_mod_types` Stardew Valley

### Новые возможности

- **EA Desktop**: детекция через `HKCU\...\EA Desktop\<key>\InstallLocation`; дополнительные пути `Electronic Arts` в реестре
- **Heroic/Lutris Flatpak**: пути `~/.var/app/com.heroic.games.launcher/config/heroic` и `~/.var/app/net.lutris.Lutris/data/lutris/games`
- **Windows long path**: `\\?\\` префикс в `normalize_path` для путей >260 символов
- **Логирование**: `eprintln!` при ошибках чтения директорий (Steam libraries)

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
| Steam | `SteamPath`, `libraryfolders.vdf` (case-insensitive), `appmanifest_*.acf` |
| GOG | Registry (priority: `path` > `exe` > `InstallDir`), Galaxy `galaxy-2.0.db` |
| Epic | `Manifests\*.item` (JSON) |
| EA / Origin | Registry `EA Games` + EA Desktop `HKCU\...\EA Desktop` |
| Ubisoft | Registry `Ubisoft\Launcher\Installs` |
| Battle.net | `product.db` (SQLite) |
| Amazon | JSON под `Amazon Games\Data` |
| Xbox/Game Pass | Registry `XboxGameExport` |
| Microsoft Store | Best-effort: `Uninstall` + InstallLocation |
| Heroic / Lutris | JSON / YAML под `~/.config`, `~/.local/share` + Flatpak paths |
| Manual | User-selected folder |

## Data Model

See [MODELS.md](../MODELS.md) — `Game`, `GameDetails`, `GameLauncher` in `src-tauri/src/models.rs`.

Дополнительные метаданные источника (ключи реестра, `heroic`/`lutris` и т.д.) могут попадать в `GameDetails.environment`.

## Detection flow (actual pipeline)

`GameDetector::detect_games` runs, in order:

**Windows**

1. **Steam** — library roots → manifests → `KNOWN_GAMES` or generic `steam_<appid>` (with skip list for tool depots; executable search: root → recursive subdirs)
2. **GOG** — registry (`path` → `exe` parent → `InstallDir`; quote trimming) → install paths → known-game exe match
3. **GOG Galaxy** — SQLite paths → same match (`launcher`: `gog`, `environment.gog_source`: `galaxy_db`)
4. **Epic** — manifest JSON → `InstallLocation` → match
5. **EA / Origin** — EA Games registry + EA Desktop (`HKCU\...\EA Desktop\<key>\InstallLocation`) → match (`launcher`: `origin`)
6. **Ubisoft** — Ubisoft Connect registry → match
7. **Battle.net** — `product.db` install paths → match (поддерживаемые игры: Overwatch 2, Diablo IV, Diablo II: Resurrected, Diablo III, World of Warcraft, StarCraft II, Hearthstone, Warcraft III: Reforged)
8. **Amazon** — JSON manifests → match
9. **Xbox** — `XboxGameExport` registry → match
10. **Microsoft Store (best-effort)** — `Uninstall` hive scan (cap on keys) → match

**Linux**

1. **Steam** — `~/.steam/steam`, `~/.local/share/Steam` + Flatpak/Snap; `libraryfolders.vdf` parsed for additional libraries; same manifest logic as Windows; recursive exe search (Linux executable bits)
2. **Heroic** — JSON scan under `~/.config/heroic` + Flatpak (`~/.var/app/com.heroic.games.launcher/config/heroic`)
3. **Lutris** — `.yml` game configs under `~/.local/share/lutris/games` + Flatpak (`~/.var/app/net.lutris.Lutris/data/lutris/games`)

Non-Windows: нет реестра Windows-лаунчеров; Epic/GOG на Linux часто через Heroic/Lutris, не через отдельные нативные сканеры.

Results are **deduplicated by `Game.id`** (first wins).

## Windows / Steam (details)

**Module**: `src-tauri/src/services/game_detector.rs`

| Step | Behavior |
|------|----------|
| Steam root | Registry `HKCU\Software\Valve\Steam`, value `SteamPath` |
| Library roots | `steamapps` under Steam root + paths from `steamapps/libraryfolders.vdf` (case-insensitive parsing, normalize `\\` and `/`) |
| Installed apps | Parse `steamapps/appmanifest_<appid>.acf` → `installdir` (case-insensitive field lookup) |
| Skip list | Tool/redistributable App IDs (e.g. Steamworks Shared) are skipped for generic registration |
| Known games | `KNOWN_GAMES` — индекс относительных путей exe → ускорение сопоставления с каталогом |
| Unknown Steam App ID | Registered as `steam_<appid>` if an executable is found and App ID not skipped |
| Known game, exe mismatch | Fallback: single `.exe` in folder, or heuristics (e.g. Stardew) |
| Executable search | Root directory first, then recursive search into subdirectories (depth limit=5) |
| Fallback exe matching | `stem.contains(&id)` — no false positives from `id.contains(&stem)` |

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
