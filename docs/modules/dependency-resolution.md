# Module: Dependency Resolution

## Responsibility

Автоматическое определение, проверка и разрешение зависимостей между модами. Построение графа зависимостей, обнаружение конфликтов и предложение решений для стабильной установки модпаков.

## Problem Statement

Зависимости модов — одна из самых сложных задач в мод-менеджменте:

```
Мод A требует Мод B >= 2.0
Мод B требует Мод C >= 1.5
Мод D требует Мод C < 1.5  ← КОНФЛИКТ!

Мод E и Мод F оба меняют один файл ← КОНФЛИКТ!
```

Это классическая задача разрешения зависимостей (как в package managers: npm, cargo, apt), но с дополнительными сложностями:
- Моды часто не имеют чётких версий
- Зависимости могут быть неявными
- Конфликты файлов не всегда означают несовместимость
- Пользователь может вручную разрешить конфликты

## Data Model

```rust
struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
    edges: Vec<DependencyEdge>,
    conflicts: Vec<DependencyConflict>,
}

struct DependencyNode {
    mod_id: String,
    name: String,
    version: Option<Version>,
    is_installed: bool,
    is_enabled: bool,
    requirements: Vec<DependencyRequirement>,
    optional_deps: Vec<DependencyRequirement>,
    incompatibilities: Vec<String>,  // Mod IDs this is incompatible with
}

struct DependencyEdge {
    from: String,    // Dependent mod ID
    to: String,      // Required mod ID
    relation: DependencyRelation,
}

enum DependencyRelation {
    Requires,        // Hard dependency — must be installed
    Recommends,      // Soft dependency — suggested but not required
    Optional,        // Optional feature dependency
    Conflicts,       // Incompatible with
    LoadsBefore,     // Should load before (for load order)
    LoadsAfter,      // Should load after
}

struct DependencyRequirement {
    mod_id: String,
    version_constraint: Option<VersionConstraint>,
    game_version: Option<VersionConstraint>,
    loader_version: Option<VersionConstraint>,
}

struct DependencyConflict {
    conflict_type: ConflictType,
    mods_involved: Vec<String>,
    description: String,
    severity: ConflictSeverity,
    resolution: Option<ConflictResolution>,
}

enum ConflictType {
    VersionMismatch,       // Required version not available
    MissingDependency,     // Required mod not installed
    IncompatibleMods,      // Mods explicitly incompatible
    FileConflict,          // Mods modify same files
    LoadOrderConflict,     // Incorrect load order
    CircularDependency,    // A → B → A
}

enum ConflictSeverity {
    Info,      // Informational, no action needed
    Warning,   // May cause issues, user should be aware
    Error,     // Will cause problems, action required
    Critical,  // Will crash game, must resolve
}

enum ConflictResolution {
    InstallMissing(String),          // Install the missing mod
    UpdateMod(String, String),       // Update mod to version
    DisableMod(String),              // Disable conflicting mod
    PrioritizeMod(String),           // Set load order priority
    ManualReview,                    // User must decide
    Ignore,                          // User chose to ignore
}
```

## Resolution Algorithm

```
1. Build dependency graph
        │
        ├──► Collect all installed mods
        ├──► Parse dependency metadata from each mod
        └──► Create nodes and edges
        │
        ▼
2. Validate dependencies
        │
        ├──► Check all required mods are installed
        ├──► Check version constraints are satisfied
        ├──► Check game version compatibility
        └──► Check for circular dependencies
        │
        ▼
3. Detect conflicts
        │
        ├──► File-level conflicts (same file modified)
        ├──► Incompatibility declarations
        ├──► Load order violations
        └──► Version conflicts
        │
        ▼
4. Resolve conflicts (automatic)
        │
        ├──► Install missing dependencies
        ├──► Suggest updates for version mismatches
        ├──► Apply load order rules
        └──► Flag unresolvable conflicts for manual review
        │
        ▼
5. Generate resolution plan
        │
        └──► Ordered list of actions to resolve all issues
        │
        ▼
6. Present to user
        │
        └──► Show conflicts, suggested resolutions, allow override
```

## Version Constraint Syntax

```rust
// Supports semver-like constraints
enum VersionConstraint {
    Exact(Version),           // =1.2.3
    GreaterThan(Version),     // >1.2.0
    GreaterThanOrEqual(Version), // >=1.2.0
    LessThan(Version),        // <2.0.0
    LessThanOrEqual(Version), // <=2.0.0
    Range { min: Version, max: Version }, // >=1.0.0, <2.0.0
    Compatible(Version),      // ~1.2.0 (>=1.2.0, <1.3.0)
    Caret(Version),           // ^1.2.0 (>=1.2.0, <2.0.0)
    Any,                      // Any version
}

// Parsing from mod metadata
// "skyui": ">=5.2"
// "skse64": "~2.2.0"
// "address_library": "^1.0"
```

## Circular Dependency Detection

```rust
// DFS-based cycle detection
fn detect_cycles(graph: &DependencyGraph) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();
    
    for node_id in graph.nodes.keys() {
        if !visited.contains(node_id) {
            dfs_cycle(
                node_id, graph, 
                &mut visited, &mut rec_stack, 
                &mut path, &mut cycles
            );
        }
    }
    
    cycles
}

// Circular dependencies are flagged as errors
// and broken by removing the lowest-priority edge
```

## Dependency Metadata Sources

| Source | Format | Reliability |
|--------|--------|-------------|
| Pantheon Repository API | JSON (requirements field) | High |
| mod.json in archive | JSON | High |
| FOMOD XML | XML (requiredInstallFiles) | Medium |
| Mod description page | HTML scraping | Low |
| Community database | JSON (LOOT-style) | Medium |
| User-defined | Manual input | Variable |

## Dependency Metadata Format

```json
{
    "mod_id": "example-mod",
    "version": "1.2.0",
    "dependencies": {
        "requires": [
            {
                "mod_id": "skyui",
                "version": ">=5.2.0",
                "name": "SkyUI"
            },
            {
                "mod_id": "skse64",
                "version": ">=2.2.0",
                "name": "SKSE64",
                "type": "loader"
            }
        ],
        "recommends": [
            {
                "mod_id": "address-library",
                "version": "^1.0",
                "name": "Address Library for SKSE Plugins"
            }
        ],
        "optional": [
            {
                "mod_id": "race-menu",
                "name": "RaceMenu",
                "feature": "enhanced character creation"
            }
        ],
        "incompatible": [
            {
                "mod_id": "old-skyui-patch",
                "name": "Old SkyUI Patch",
                "reason": "No longer needed, causes conflicts"
            }
        ]
    }
}
```

## Resolution UI

```
┌─────────────────────────────────────────────────────────────┐
│  Dependency Resolution                                        │
│                                                              │
│  ⚠️ 3 issues found                                            │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  ❌ Missing: Address Library for SKSE Plugins         │  │
│  │      Required by: Example Mod                         │  │
│  │      [Install from Repository] [Cancel Install]            │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  ⚠️  Version: SkyUI 5.1 installed, 5.2+ required      │  │
│  │      Required by: Example Mod                         │  │
│  │      [Update SkyUI] [Ignore]                          │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  ⚠️  Conflict: Mod A and Mod B modify same file       │  │
│  │      File: Data/meshes/armor/helmet.nif               │  │
│  │      [Use Mod A] [Use Mod B] [Both (load order)]      │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  [Apply All] [Cancel]                                       │
└─────────────────────────────────────────────────────────────┘
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `mod-installer` | Checks dependencies before installation |
| `mod-repository-api` | Fetches dependency info from repository |
| `load-order-manager` | Uses dependency graph for load ordering |
| `database` | Stores dependency metadata and resolution state |
| `update-checker` | Checks if dependency updates resolve conflicts |

## API

```rust
#[tauri::command]
pub async fn build_dependency_graph(
    game_id: String,
) -> Result<DependencyGraph, String>;

#[tauri::command]
pub async fn resolve_dependencies(
    game_id: String,
    mods_to_install: Vec<String>,
) -> Result<ResolutionPlan, String>;

#[tauri::command]
pub async fn install_missing_dependencies(
    game_id: String,
    mod_id: String,
) -> Result<Vec<String>, String>;  // Installed mod IDs

#[tauri::command]
pub async fn get_dependency_tree(
    mod_id: String,
) -> Result<DependencyTree, String>;

#[tauri::command]
pub async fn get_dependents(
    mod_id: String,
) -> Result<Vec<String>, String>;  // Mods that depend on this one
```

## Best Practices

1. **Non-blocking warnings** — Recommendations should not block installation unless critical
2. **Transparency** — Show full dependency tree, not just immediate dependencies
3. **User override** — Always allow users to ignore warnings and proceed
4. **Dependent awareness** — Warn before disabling a mod that others depend on
5. **Lazy resolution** — Resolve dependencies at install time, not at scan time
6. **Caching** — Cache dependency info to avoid repeated API calls
7. **Community data** — Allow community-maintained dependency databases (LOOT-style)

## Notes

- Many mods don't declare dependencies — use community knowledge bases
- Version constraints are often informal — be lenient in matching
- Circular dependencies are rare but possible — detect and break gracefully
- File conflicts don't always mean incompatibility — load order may resolve them
- Consider implementing a SAT solver for complex dependency resolution (like cargo)
