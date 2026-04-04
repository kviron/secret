# Module: Load Order Manager

## Responsibility

Manages plugin load order for Bethesda games (Skyrim, Fallout, etc.) by reading plugin headers, building dependency graphs, and providing LOOT-like sorting with user override support.

## Pantheon Comparison

Pantheon `gamebryo-plugin-management` extension handles:
- Plugin list reading (esp/esm/esl files)
- Load order persistence (loadorder.txt)
- Plugin enabling/disabling (plugins.txt)
- Userlist management (managed plugins)
- LOOT integration for auto-sorting

## Data Model

```rust
struct Plugin {
    name: String,                 // Plugin filename (e.g., "Dragonborn.esm")
    file_path: PathBuf,          // Full path to plugin
    is_native: bool,             // Is it a native game plugin
    mod_id: Option<String>,       // Associated mod ID if installed
    is_esm: bool,                // Is master file (.esm)
    is_esl: bool,                // Is light plugin (.esl)
    is_esl_flagged: bool,        // Has ESL flag but .esp extension
    is_master: bool,              // Is master file
    header_version: u32,         // Plugin header version
    file_flags: u32,             // Record flags
    masters: Vec<String>,         // Master files
    load_order_index: Option<u32>, // Current load order position
    enabled: bool,                // Is enabled (in plugins.txt)
    warnings: Vec<PluginWarning>,  // LOOT warnings
    hash: u64,                    // File hash for identification
}

struct LoadOrder {
    game_id: String,
    plugins: Vec<PluginEntry>,
    last_updated: DateTime<Utc>,
}

struct PluginEntry {
    name: String,
    enabled: bool,
    load_order: u32,
    group: Option<String>,
}

enum PluginWarning {
    Delev(uint32_t),    // FormID delev tracker
    Relev(uint32_t),    // FormID relev tracker
    MissingMaster(String),
    Itm(uint32_t),      // Identical to Master record
    Undelected(uint32_t), // Undefined record
    Context(String),     // Context-specific warning
}
```

## Load Order Flow

```
1. Get all plugins (esp/esm) from deployed mods
         │
         ▼
2. Read plugin headers (TES4, FO3, SSE, FO4 format)
         │
         ├──► Parse masters list
         ├──► Parse record flags
         └──► Parse special flags (ESL, ESM)
         │
         ▼
3. Build dependency graph
         │
         ├──► Direct dependencies (masters)
         ├──► File dependencies (CRASH plugin)
         └──► Soft dependencies (FRBL data)
         │
         ▼
4. Run topological sort (with LOOT-like heuristics)
         │
         ├──► LOOT metadata if available
         ├──► Group sorting (official, DLC, community)
         └──► Tie-breaking (alphabetical)
         │
         ▼
5. Apply user overrides
         │
         ├──► User-defined group placement
         ├──► Manual reorder
         └──► Disabled plugins
         │
         ▼
6. Return sorted list
         │
         ▼
7. Persist to loadorder.txt / plugins.txt
```

## LOOT Integration

Pantheon integrates with LOOT (Load Order Optimisation Tool):

```rust
struct LOOTMetadata {
    globals: Vec<LOOTGlobal>,
    plugins: Vec<LOOTPlugin>,
    groups: Vec<LOOTGroup>,
}

struct LOOTPlugin {
    name: String,
    after: Vec<LOOTReference>,
    before: Vec<LOOTReference>,
    group: Option<String>,
    requirements: Vec<LOOTReference>,
    incompatibilities: Vec<LOOTReference>,
    messages: Vec<LOOTMessage>,
}

struct LOOTGroup {
    name: String,
    after: Vec<String>,
    before: Vec<String>,
}

struct LOOTReference {
    name: String,
    display_name: Option<String>,
    condition: Option<String>,
}
```

## Plugin Header Format (Bethesda Games)

### Skyrim (TES4)

```rust
struct TES4Header {
    flags: TES4Flags,           // 4 bytes: ESM, ESL, light
    form_id: u32,              // Always 0 for plugin header
    timestamp: u32,            // File timestamp
    version: u32,              // Version number (e.g., 1.70)
    plugin_info: PluginInfo,   // Internal plugin info
    description: String,        // Plugin description
    masters: Vec<MasterRef>,   // Master files
}

enum TES4Flags {
    ESM = 0x00000001,
    ESL = 0x00000200,
    // ...
}
```

### Fallout 4 (FO4)

Similar structure but with different header format and additional fields.

## Userlist Management

Pantheon maintains a "userlist" for managed plugins:

```rust
struct Userlist {
    game_id: String,
    managed: Vec<String>,      // Plugins user has explicitly managed
    automatic: bool,           // Is auto-sort enabled
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | Gets game info, extension for plugin parsing |
| `mod-installer` | Plugins (ESP/ESM) are installed as mod files |
| `deploy-manager` | Deployed mods contain plugins |
| `database` | Stores load order and plugin info |
| `settings` | User preferences for sorting |

## API

```rust
#[tauri::command]
pub async fn get_load_order(game_id: String) -> Result<Vec<Plugin>, String>;

#[tauri::command]
pub async fn set_load_order(
    game_id: String,
    order: Vec<String>,  // Plugin names in desired order
) -> Result<(), String>;

#[tauri::command]
pub async fn set_plugin_enabled(
    game_id: String,
    plugin: String,
    enabled: bool,
) -> Result<(), String>;

#[tauri::command]
pub async fn auto_sort_load_order(game_id: String) -> Result<Vec<String>, String>;

#[tauri::command]
pub async fn get_plugin_info(
    game_id: String,
    plugin_path: PathBuf,
) -> Result<Plugin, String>;
```

## LOOT Metadata Format

Pantheon and LOOT use YAML metadata files:

```yaml
plugins:
  - name: Dragonborn.esm
    after:
      - 'Dawnguard.esm'
      - 'HearthFires.nif'
    group: dragonborn_dlc
    
  - name: unofficial_skyrim_patch.esp
    after:
      - name: Skyrim.esm
        condition: 'file("Dragonborn.esm")'
    group: unofficial_patches
    
groups:
  - name: main
    after:
      - dlc
  - name: dlc
    after:
      - official
  - name: official
```

## Notes

- ESL plugins (.esl) sort differently and can load in any position
- Light plugins (.esp with ESL flag) have 4096 record limit
- Load order affects which plugin "wins" in case of conflicts
- Some records override others based on load order
- LOOT metadata needs updating for new mods
- Plugin hash can identify mods across different filenames
