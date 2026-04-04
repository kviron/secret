# Module: Mod Installer

## Responsibility

Installation of mods: archive extraction, config application, validation, and mod type detection.

## Pantheon Comparison

Pantheon `mod_management` extension handles mod installation with:
- Multiple installer types (simple, FOMOD, etc.)
- File validation and conflict detection
- Mod staging and metadata storage
- Category/mod type management

## Installation Flow

```
1. User selects mod archive (zip, 7z, rar, or mod package)
       │
       ▼
2. Detect archive type / mod type
       │
       ├──► FOMOD (XML-based installer)
       ├──► BSAT (Bethesda Archive - BA2)
       ├──► BepInEx (.NET mod loader)
       ├──► Simple Archive (just files)
       └──► Unknown type
       │
       ▼
3. Run game-specific installer (if exists)
       │
       ▼
4. Extract to staging directory
       │
       ▼
5. Validate files (checksums, required files)
       │
       ▼
6. Parse metadata (mod.json, info.json)
       │
       ▼
7. Register mod in database
       │
       ▼
8. Return ModInfo
```

## Mod Types

| Type | Description | Pantheon Handler |
|------|-------------|----------------|
| Simple Archive | Just files to copy | Default installer |
| FOMOD | XML-based installer with options | `installer-fomod` |
| FOOMAD | FOMOD variant for Morrowind | `installer-foomad` |
| BSAT | Bethesda Archive (BA2) | `gamebryo-ba2-support` |
| BepInEx | .NET mod loader | `modtype-bepinex` |
| DAZip | Dragon Age specific | `modtype-dazip` |
| ENB | ENB preset files | `modtype-enb` |
| Script Extender | SKSE, ScriptHook, etc. | `modtype-umm` |
| Gamebryo Plugin | ESP/ESM/ESL files | Built-in |

## Data Model

```rust
struct Mod {
    id: String,                     // Unique mod ID (UUID)
    game_id: String,                // Parent game ID
    name: String,                   // Display name
    version: Option<String>,        // Semantic version
    category: String,               // Primary category
    mod_type: ModType,              // Type of mod
    install_path: PathBuf,          // Staging directory
    files: Vec<ModFile>,            // List of installed files
    enabled: bool,                  // Is mod enabled
    flags: Vec<String>,             // State flags (installed, upgradeable, missing)
    attributes: HashMap<String, String>,  // Custom attributes
    install_time: DateTime<Utc>,   // When installed
    last_modified: DateTime<Utc>,   // Last update time
    metadata: Option<ModMetadata>,   // Extended metadata
    conflicts: Vec<String>,          // IDs of conflicting mods
    dependencies: Vec<String>,       // IDs of required mods
}

struct ModFile {
    path: PathBuf,                  // Relative path in staging
    size: u64,                      // File size
    hash: Option<String>,           // SHA256 hash
    isArchive: bool,                 // Is this an archive (BSA/BA2)
}

enum ModType {
    Simple,
    Fomod,
    FooMAD,
    Bsat,
    BepInEx,
    Dazip,
    ENB,
    ScriptExtender,                  // SKSE, ScriptHook, etc.
    ModPlugin,                      // ESP/ESM/ESL
}

struct ModMetadata {
    author: Option<String>,
    description: Option<String>,
    website: Option<String>,
    installer_version: Option<String>,
    installation_files: Vec<InstallationFile>,
    screenshot: Option<String>,
    category: Option<String>,
}

struct InstallationFile {
    type: String,                   // "replace", "optional", "隐蔽"
    source: String,                 // Archive path
    destination: String,            // Extraction path
}
```

## Installer Interface

```rust
// Extension trait for custom installers
pub trait ModInstaller {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn supported_types(&self) -> &[ModType];
    
    // Test if this installer can handle the mod
    fn test(&self, archive: &Path) -> Result<bool, String>;
    
    // Install the mod
    fn install(
        &self,
        archive: &Path,
        dest: &Path,
        game: &Game,
        options: InstallOptions,
    ) -> Result<Mod, String>;
    
    // Uninstall the mod (cleanup)
    fn uninstall(&self, mod_id: &str) -> Result<(), String>;
    
    // Parse mod info from archive without installing
    fn parse_info(&self, archive: &Path) -> Result<ModMetadata, String>;
}

// Install options passed to installers
struct InstallOptions {
    staging_path: PathBuf,
    game_support_path: PathBuf,
    mod_type: Option<ModType>,
    fileOverrides: HashMap<String, String>,  // source -> dest mappings
}
```

## Pantheon Installer API (Reference)

```typescript
// Pantheon installer signature
interface IInstallFunc {
  (context: IExtensionContext, tool: IInstallTool): Promise<IInstallResult>;
}

// IInstallResult
interface IInstallResult {
  container?: IContainer;
  directives?: IDirective[];
  files?: IFileChange[];
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | Gets game info for game-specific installers |
| `deploy-manager` | After install, mod goes to staging; deploy publishes to game folder |
| `database` | Stores mod records and metadata |
| `download-manager` | Downloads can be mod archives ready for installation |
| `load-order-manager` | Plugins (ESP/ESM) are added to load order |

## Archive Handling

```
Archive Detection:
├── .zip  → Native Rust zip crate
├── .7z   → sevenz-rust crate  
├── .rar  → unrar crate (or external tool)
├── .bsa  → gamebryo-bsa-support (Bethesda)
└── .ba2  → gamebryo-ba2-support (Creation Engine)
```

## Validation Rules

1. **Required Files**: Some mods require specific files to be present
2. **File Size**: Warn on unusual file sizes
3. **Hash Validation**: Verify file integrity against known good hashes
4. **Conflict Detection**: Check against already-installed mods
5. **Version Check**: Detect if newer version is available

## Notes

- FOMOD parsing requires XML reading and option handling UI
- BSAT/BA2 archives need special handling for Bethesda games
- BepInEx requires .NET runtime detection and patching
- Script extenders (SKSE) need special installer logic
- Some games have merge behavior where mods overwrite each other in Data folder
