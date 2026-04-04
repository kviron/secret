# Module: Security & Validation

## Responsibility

Валидация и проверка безопасности модов перед установкой. Обнаружение потенциально вредоносных файлов, проверка целостности, верификация источников и защита сохранений пользователя.

## Threat Model

| Угроза | Описание | Последствия |
|--------|----------|-------------|
| Вредоносные скрипты | .exe, .bat, .ps1 в архивах модов | Выполнение произвольного кода |
| Подмена файлов | Модифицированные DLL с бэкдорами | Кража данных, кейлоггеры |
| Повреждение сохранений | Несовместимые моды меняют формат сейвов | Потеря прогресса |
| Конфликт модов | Перезапись критических файлов | Краши, баги, сломанные квесты |
| Устаревшие лоадеры | Script extender не соответствует версии игры | Невозможность запуска |

## Validation Pipeline

```
Mod Archive Received
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 1: Archive Validation                                      │
│                                                                  │
│  ├──► Verify archive integrity (CRC, checksum)                   │
│  ├──► Detect archive type (zip, 7z, rar)                         │
│  ├──► Check for corruption                                      │
│  └──► Estimate extraction size                                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 2: File Type Scanning                                      │
│                                                                  │
│  ├──► Enumerate all files in archive                             │
│  ├──► Classify files by type:                                    │
│  │   ├──► Safe: .esp, .esm, .esl, .nif, .dds, .bsa, .ba2       │
│  │   ├──► Caution: .dll, .exe, .bat, .ps1, .cmd, .vbs          │
│  │   ├──► Config: .ini, .json, .xml, .txt, .yaml               │
│  │   └──► Scripts: .pex, .psc, .lua, .py                        │
│  └──► Flag suspicious files                                     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 3: Security Checks                                         │
│                                                                  │
│  ├──► Check against known malware signatures                    │
│  ├──► Verify file hashes against repository/moddb (if available)     │
│  ├──► Scan executables/DLLs with heuristic analysis             │
│  ├──► Check for known exploit patterns                          │
│  └──► Verify digital signatures (if present)                    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 4: Compatibility Check                                     │
│                                                                  │
│  ├──► Check game version compatibility                          │
│  ├──► Check loader version requirements                         │
│  ├──► Check required dependencies are installed                 │
│  ├──► Check for known conflicts with enabled mods               │
│  └──► Verify mod is not outdated for current game version       │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ Stage 5: User Notification                                       │
│                                                                  │
│  ├──► Show validation summary                                    │
│  ├──► Warn about flagged files                                   │
│  ├──► Show conflict report                                       │
│  └──► Require explicit confirmation for risky mods               │
└─────────────────────────────────────────────────────────────────┘
```

## Data Model

```rust
struct ValidationResult {
    mod_id: Option<String>,
    archive_path: PathBuf,
    status: ValidationStatus,
    file_scan: FileScanResult,
    security_scan: SecurityScanResult,
    compatibility: CompatibilityResult,
    warnings: Vec<ValidationWarning>,
    blocked_reasons: Vec<String>,
}

enum ValidationStatus {
    Safe,               // No issues found
    Caution,            // Minor warnings, proceed with awareness
    Risky,              // Significant concerns, requires confirmation
    Blocked,            // Cannot install (malware, corruption)
}

struct FileScanResult {
    total_files: usize,
    total_size: u64,
    file_types: HashMap<FileType, usize>,
    suspicious_files: Vec<SuspiciousFile>,
}

struct SuspiciousFile {
    path: String,
    reason: SuspicionReason,
    severity: Severity,
}

enum SuspicionReason {
    ExecutableInDataFolder,     // .exe in Data/ is unusual
    ScriptInRoot,               // Script files in root directory
    KnownMalwarePattern,        // Matches known malware signature
    ObfuscatedName,             // Suspicious filename (random chars)
    HiddenFile,                 // Hidden/system attribute files
    AlternateDataStream,        // ADS on Windows
}

enum Severity {
    Info,
    Warning,
    Critical,
}

enum FileType {
    Plugin,         // .esp, .esm, .esl
    Mesh,           // .nif
    Texture,        // .dds, .png, .tga
    Archive,        // .bsa, .ba2
    Script,         // .pex, .psc, .lua
    Config,         // .ini, .json, .xml
    Executable,     // .exe, .dll
    Script,         // .bat, .ps1, .cmd
    Document,       // .txt, .md, .html
    Other,
}

struct SecurityScanResult {
    malware_signatures: Vec<String>,   // Matched signatures
    hash_verified: bool,                // Hash matches known good
    signature_valid: bool,              // Digital signature valid
    heuristic_score: f32,              // 0.0 (safe) to 1.0 (malicious)
}

struct CompatibilityResult {
    game_version_compatible: bool,
    loader_compatible: bool,
    dependencies_met: Vec<String>,      // Missing dependency names
    conflicts: Vec<String>,             // Conflicting mod names
    requires_update: bool,              // Mod is outdated
}

struct ValidationWarning {
    code: String,
    message: String,
    severity: Severity,
    file: Option<String>,
    suggestion: Option<String>,
}
```

## Malware Signature Database

```rust
// Known malware patterns (updated from remote source)
struct MalwareSignatureDB {
    version: u64,
    last_updated: DateTime<Utc>,
    signatures: Vec<MalwareSignature>,
}

struct MalwareSignature {
    id: String,
    name: String,                    // Malware family/name
    pattern: PatternType,
    description: String,
    severity: Severity,
}

enum PatternType {
    Hash(String),                    // Known file hash
    Filename(String),                // Known malicious filename
    BytePattern(Vec<u8>),            // Byte sequence pattern
    Behavior(String),                // Behavioral pattern
}
```

## Save Game Protection

```rust
struct SaveGameInfo {
    path: PathBuf,
    name: String,
    created_at: DateTime<Utc>,
    game_version: String,
    active_plugins: Vec<String>,
    mod_count: usize,
    is_corrupted: bool,
}

// Before installing/uninstalling mods that affect saves:
// 1. Scan active save games
// 2. Check if mod changes would invalidate saves
// 3. Warn user about potential save corruption
// 4. Offer to backup saves before proceeding
```

## Validation Rules

```rust
// File type rules
const SAFE_EXTENSIONS: &[&str] = &[
    ".esp", ".esm", ".esl", ".nif", ".dds", ".bsa", ".ba2",
    ".wav", ".ogg", ".mp3", ".txt", ".md", ".ini", ".json",
    ".xml", ".yaml", ".pex", ".psc",
];

const CAUTION_EXTENSIONS: &[&str] = &[
    ".dll", ".exe", ".bat", ".ps1", ".cmd", ".vbs",
    ".lua", ".py", ".sh",
];

const BLOCKED_EXTENSIONS: &[&str] = &[
    ".scr", ".com", ".msi", ".inf", ".reg",
    ".js", ".wsf", ".cpl",
];

// Validation checks
pub fn validate_mod_archive(
    archive_path: &Path,
    game: &Game,
    installed_mods: &[Mod],
) -> ValidationResult {
    let mut result = ValidationResult::default();
    
    // Stage 1: Archive integrity
    if !verify_archive_integrity(archive_path) {
        result.status = ValidationStatus::Blocked;
        result.blocked_reasons.push("Archive is corrupted".to_string());
        return result;
    }
    
    // Stage 2: File type scan
    result.file_scan = scan_file_types(archive_path);
    
    // Stage 3: Security checks
    result.security_scan = run_security_checks(&result.file_scan);
    
    // Stage 4: Compatibility
    result.compatibility = check_compatibility(game, installed_mods);
    
    // Determine overall status
    result.status = determine_status(&result);
    
    result
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `mod-installer` | Validates archives before installation |
| `download-manager` | Validates downloaded files |
| `game-launcher` | Checks loader compatibility before launch |
| `database` | Stores validation results and malware DB |
| `deploy-manager` | Checks for conflicts before deployment |

## API

```rust
#[tauri::command]
pub async fn validate_mod(
    archive_path: PathBuf,
    game_id: String,
) -> Result<ValidationResult, String>;

#[tauri::command]
pub async fn scan_game_folder(
    game_id: String,
) -> Result<FileScanResult, String>;

#[tauri::command]
pub async fn update_malware_db() -> Result<(), String>;

#[tauri::command]
pub async fn check_save_compatibility(
    game_id: String,
    mod_changes: Vec<ModChange>,
) -> Result<Vec<SaveGameWarning>, String>;

#[tauri::command]
pub async fn backup_saves(
    game_id: String,
    dest: PathBuf,
) -> Result<PathBuf, String>;
```

## Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `validation_complete` | `ValidationResult` | Mod validation finished |
| `malware_detected` | `SecurityAlert` | Potential malware found |
| `save_at_risk` | `SaveGameWarning` | Save game may be affected |

## Best Practices

1. **Treat all mods as untrusted** — Validate every archive, even from trusted sources
2. **Sandboxed validation** — Run risky operations in isolated subprocesses
3. **Transparent reporting** — Show users exactly what files are being installed
4. **Opt-in scanning** — Allow users to enable additional scanning (ClamAV, etc.)
5. **Signature updates** — Periodically update malware signature database
6. **Save protection** — Always backup saves before making destructive changes
7. **Human-in-the-loop** — Never auto-apply destructive changes without confirmation

## Notes

- Hash verification requires a known-good database (repository provides file hashes)
- Heuristic analysis can produce false positives — always allow user override
- Some legitimate mods include .exe installers (FOMOD, BepInEx) — context matters
- Script files (.pex) are compiled and generally safe, but source (.psc) review is ideal
- Consider integrating with community-driven malware databases
