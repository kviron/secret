# Module: Extension System

## Responsibility

Plugin system for adding support for new games and mod types at runtime, similar to Pantheon's extension architecture.

## Pantheon Comparison

Pantheon has a comprehensive extension system where:
- Each game has its own extension (`game-skyrim`, `game-fallout4`)
- Each mod type has a handler extension (`modtype-bepinex`)
- Feature extensions provide core functionality (`mod_management`, `nexus_integration`)
- Extensions register reducers, pages, settings, game handlers, installers

## Extension Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Extension System Architecture                       │
└─────────────────────────────────────────────────────────────────────────┘

    Extension Directory
    ┌─────────────────────────────────────────────────────────────────┐
    │  extensions/                                                      │
    │  ├── game-skyrim/        # Game-specific extension                │
    │  │   ├── Cargo.toml      # Rust crate                            │
    │  │   ├── src/                                                   │
    │  │   │   ├── lib.rs      # Extension entry (impl Extension trait)│
    │  │   │   ├── detection.rs                                         │
    │  │   │   ├── installers.rs                                        │
    │  │   │   └── lo.rs                                               │
    │  │   └── assets/                                                 │
    │  │       └── game.json    # Game manifest                         │
    │  │                                                              │
    │  ├── modtype-bepinex/   # Mod type handler                       │
    │  │   └── ...                                                     │
    │  │                                                              │
    │  └── mod_management/     # Core feature extension                │
    │      └── ...                                                     │
    └─────────────────────────────────────────────────────────────────┘
```

## Extension Types

| Type | Description | Pantheon Equivalent | Pantheon Example |
|------|-------------|-------------------|------------------|
| Game Extension | Game-specific logic | `game-skyrim`, `game-fallout4` | `game-skyrim`, `game-witcher3` |
| Mod Type | Mod type handler | `modtype-bepinex` | `modtype-fomod` |
| Installer | Custom installer | `installer-fomod` | `installer-fomod` |
| Feature | Core functionality | `mod_management` | (built-in) |
| Store Integration | Game store detection | `gamestore-steam` | (built-in) |

## Extension Manifest

```json
{
    "id": "game-skyrimse",
    "name": "The Elder Scrolls V: Skyrim Special Edition",
    "version": "1.0.0",
    "type": "game",
    "runtime": {
        "requires": ["modtype-plugin", "modtype-fomod", "installer-fomod"],
        "optional": ["gamebryo-plugin-management"]
    },
    "detection": {
        "steam_app_id": "489830",
        "gog_game_id": "1458055853",
        "epic_offer_id": "some-id",
        "xbox_product_id": "some-guid"
    },
    "mod_paths": {
        "default": "Data",
        "modorganizer": "Mods",
        "pantheon": "staging/mods"
    },
    "merge_mods": true,
    "archive_types": [".bsa", ".ba2"],
    "plugin_types": [".esp", ".esm", ".esl"],
    "loot": {
        "metadata_file": "skyrim.yaml",
        "groups": ["skyrim", "skyrim_unofficial"]
    }
}
```

## Extension Trait (Rust)

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Lifecycle
    fn init(&self, ctx: &mut ExtensionContext) -> Result<(), String>;
    fn shutdown(&self) -> Result<(), String> { Ok(()) }
}

pub trait GameExtension: Extension {
    fn detect(&self) -> Option<GameInfo>;
    fn get_mod_paths(&self, install_path: &Path) -> HashMap<String, PathBuf>;
    fn list_plugins(&self, game_path: &Path) -> Result<Vec<PluginInfo>, String>;
    fn validate_mod(&self, mod_path: &Path) -> Result<ValidationResult, String>;
}

pub trait ModTypeExtension: Extension {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
    fn test(&self, archive: &Path) -> bool;
    fn install(&self, archive: &Path, dest: &Path) -> Result<Mod, String>;
}

pub trait InstallerExtension: Extension {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
    fn test(&self, archive: &Path) -> Result<bool, String>;
    fn install(
        &self,
        archive: &Path,
        dest: &Path,
        game: &Game,
    ) -> Result<Mod, String>;
}
```

## Extension Context

```rust
pub struct ExtensionContext {
    pub db: Database,
    pub games: GameRegistry,
    pub mod_types: ModTypeRegistry,
    pub installers: InstallerRegistry,
    pub settings: SettingsManager,
}

impl ExtensionContext {
    // Register game detection
    pub fn register_game(&mut self, ext: Arc<dyn GameExtension>) { ... }
    
    // Register mod type handler
    pub fn register_mod_type(&mut self, ext: Arc<dyn ModTypeExtension>) { ... }
    
    // Register installer
    pub fn register_installer(&mut self, ext: Arc<dyn InstallerExtension>) { ... }
    
    // Register settings
    pub fn register_settings(&mut self, category: &str, component: Component) { ... }
}
```

## Extension Loading Flow

```
1. On startup, scan extensions/ directory
         │
         ▼
2. Load extension manifest (game.json)
         │
         ▼
3. Check dependencies (runtime.requires)
         │
         ├──► Missing dependency ──► Skip extension, log warning
         │
         ▼
4. Initialize extension (call init())
         │
         ▼
5. Register game detection handlers
         │
         ▼
6. Register mod type handlers
         │
         ▼
7. Register installers
         │
         ▼
8. Update game list with detected games
```

## Pantheon Extension API Reference

```typescript
// Pantheon extension entry point
function init(context: IExtensionContext): boolean {
  // Register state reducer
  context.registerReducer(
    ['persistent', 'plugins'],
    reducer
  );
  
  // Register main page
  context.registerMainPage(
    'category-icons',
    'plugins',
    PluginList,
    {
      props: {},
      title: '...',
      icon: 'plugin-icon'
    }
  );
  
  // Register game
  context.registerGame(gameMode);
  
  // Register installer
  context.registerInstaller(
    'fomod-installer',
    2000,  // priority
    testFomod,
    installFomod
  );
  
  // Register deployment method
  context.registerDeploymentMethod({
    id: 'symlink',
    name: 'Symlink',
    detect: async () => true
  });
  
  return true;
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | Extensions provide game detection |
| `mod-installer` | Extensions provide mod type handlers and installers |
| `deploy-manager` | Extensions can add deployment methods |
| `load-order-manager` | Game extensions provide LOOT metadata |
| `database` | Extensions store custom data |

## Built-in Extensions (Core)

These should be built into Pantheon core:

| Extension | Purpose |
|-----------|---------|
| `game-generic` | Generic game support (fallback) |
| `modtype-simple` | Simple archive handler |
| `modtype-plugin` | ESP/ESM/ESL plugin handler |
| `installer-default` | Default file copy installer |
| `deployment-symlink` | Symlink deployment |
| `deployment-hardlink` | Hardlink deployment |
| `deployment-copy` | File copy deployment |

## Extension Distribution

```
Extension Distribution Options:
├── Built-in (compiled with core)
├── Dynamic loading (.so/.dll in extensions/)
└── Bundled (shipped with app)
```

## Notes

- Extensions should be sandboxed where possible
- Use semver for extension version compatibility
- Extension manifests should be validated on load
- Provide extension API versioning for future compatibility
- Consider extension dependencies and load order
- Some games (Bethesda) need multiple extensions working together
