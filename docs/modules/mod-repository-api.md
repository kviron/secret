# Module: Mod Repository API

## Responsibility

Интеграция с собственным репозиторием модов Pantheon. Поиск, просмотр, скачивание и управление модами через официальный API. Включает аутентификацию, браузер модов, CDN-загрузки и отслеживание обновлений.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Pantheon Client                           │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │  Mod Browser │    │  Downloader  │    │  Update Check│  │
│  │  (UI)        │    │  (Background)│    │  (Scheduled) │  │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘  │
│         │                   │                   │           │
│         └───────────────────┼───────────────────┘           │
│                             ▼                               │
│                  ┌────────────────────┐                     │
│                  │  Repository API    │                     │
│                  │  Client            │                     │
│                  └────────┬───────────┘                     │
└───────────────────────────┼─────────────────────────────────┘
                            │ HTTPS / REST
┌───────────────────────────▼─────────────────────────────────┐
│                    Pantheon Repository Server                 │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  API Server  │  │  CDN / Mirrors│  │  Database        │  │
│  │  (REST)      │  │  (Downloads)  │  │  (Mods, Users)   │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Auth Server │  │  Search Engine│  │  File Storage    │  │
│  │  (OAuth2)    │  │  (Full-text)  │  │  (S3-compatible) │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Authentication Flow

```
1. User clicks "Sign In"
        │
        ▼
2. Open browser to Pantheon OAuth URL
        │
        └──► https://auth.pantheon-mods.com/oauth/authorize
             ?client_id={CLIENT_ID}
             &redirect_uri=pantheon://auth/callback
             &response_type=code
        │
        ▼
3. User authorizes application
        │
        ▼
4. Browser redirects to pantheon://auth/callback?code=AUTH_CODE
        │
        ▼
5. Exchange code for access token
        │
        └──► POST https://api.pantheon-mods.com/oauth/token
             { code, client_id, client_secret, grant_type }
        │
        ▼
6. Store tokens securely (OS keychain)
        │
        ▼
7. API calls include Authorization: Bearer {token}
```

## Data Model

```rust
struct RepositoryAuth {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
    user_id: String,
    username: String,
    is_premium: bool,
}

struct RepositoryMod {
    id: String,                     // Server-assigned mod ID
    game_id: String,                // Game domain/ID
    name: String,
    slug: String,                   // URL-friendly name
    summary: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: ModAuthor,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: DateTime<Utc>,
    size: u64,                      // File size in bytes
    download_count: u64,
    endorsement_count: u64,
    category: Option<ModCategory>,
    tags: Vec<String>,
    images: Vec<ModImage>,          // Screenshots
    files: Vec<ModFile>,
    dependencies: Vec<ModDependency>,
    requirements: Vec<ModRequirement>,
    rating: Option<f32>,            // Average rating (0-5)
    is_featured: bool,
}

struct ModAuthor {
    id: String,
    username: String,
    avatar_url: Option<String>,
}

struct ModImage {
    url: String,
    thumbnail_url: String,
    caption: Option<String>,
    is_primary: bool,
}

struct ModFile {
    id: String,
    name: String,
    version: String,
    size: u64,
    uploaded_at: DateTime<Utc>,
    is_primary: bool,
    category: ModFileCategory,
    changelog: Option<String>,
    file_hash: String,              // SHA256
    cdn_urls: Vec<CdnMirror>,
}

enum ModFileCategory {
    Main,
    Update,
    Optional,
    Old,
    Misc,
    Archive,
}

struct CdnMirror {
    name: String,                   // e.g., "EU-West", "US-East"
    region: String,
    url: String,
    latency_ms: Option<u64>,        // Measured latency
}

struct ModDependency {
    mod_id: String,
    game_id: String,
    relation_type: DependencyRelation,
    version_constraint: Option<String>,
}

enum DependencyRelation {
    Requires,
    Recommends,
    Optional,
    Incompatible,
}

struct ModSearchQuery {
    game_id: String,
    search_term: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    author: Option<String>,
    sort: ModSortOption,
    page: u32,
    per_page: u32,
    date_range: Option<DateRange>,
}

enum ModSortOption {
    Trending,
    Endorsed,
    Updated,
    Created,
    Downloads,
    Rating,
}
```

## API Client

```rust
pub struct RepositoryApiClient {
    client: reqwest::Client,
    auth: Option<RepositoryAuth>,
    base_url: String,
}

impl RepositoryApiClient {
    // Mod browsing
    pub async fn search_mods(&self, query: &ModSearchQuery) 
        -> Result<PaginatedResponse<RepositoryMod>, String>;
    
    pub async fn get_mod(&self, mod_id: &str) 
        -> Result<RepositoryMod, String>;
    
    pub async fn get_mod_files(&self, mod_id: &str) 
        -> Result<Vec<ModFile>, String>;
    
    // Download
    pub async fn get_download_links(&self, mod_id: &str, file_id: &str) 
        -> Result<Vec<CdnMirror>, String>;
    
    pub async fn download_mod(
        &self,
        mod_id: &str,
        file_id: &str,
        dest: &Path,
    ) -> Result<PathBuf, String>;
    
    // User actions
    pub async fn endorse_mod(&self, mod_id: &str) -> Result<(), String>;
    pub async fn rate_mod(&self, mod_id: &str, rating: u8) -> Result<(), String>;
    
    // Updates
    pub async fn check_mod_updates(
        &self,
        mod_refs: &[ModVersionRef],
    ) -> Result<Vec<ModUpdateInfo>, String>;
    
    // Categories & tags
    pub async fn get_categories(&self, game_id: &str) 
        -> Result<Vec<ModCategory>, String>;
    
    pub async fn get_tags(&self, game_id: &str) 
        -> Result<Vec<ModTag>, String>;
}
```

## Download Flow with CDN

```
1. User clicks "Download"
        │
        ▼
2. Request download links from API
        │
        └──► GET /api/v1/mods/{id}/files/{file_id}/links
        │
        ▼
3. Receive list of CDN mirrors
        │
        └──► [
               { "name": "EU-West", "url": "https://eu.cdn...", "latency_ms": 12 },
               { "name": "US-East", "url": "https://us.cdn...", "latency_ms": 45 },
             ]
        │
        ▼
4. Select fastest mirror (lowest latency)
        │
        └──► Measure latency if not cached
        │
        ▼
5. Pass URL to download-manager
        │
        └──► Download with resume support, chunked
        │
        ▼
6. Verify file hash after download
        │
        └──► Compare SHA256 with file_hash from API
        │
        ▼
7. Pass to mod-installer
```

## Mod Browser UI Integration

```
┌─────────────────────────────────────────────────────────────┐
│  Mod Repository Browser                                       │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Search: [________________________] [🔍] [Filters ▼]  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  Categories: [All] [Gameplay] [Graphics] [Quests] [UI]      │
│  Sort: [Trending ▼]  Page: 1 of 50                          │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  ┌─────┐  Mod Name                          ⭐ 1.2k   │  │
│  │  │ IMG │  Author • Updated 2d ago                     │  │
│  │  │     │  50k downloads • 4.8 ★                       │  │
│  │  └─────┘  [Download] [Details]                       │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  ... more mod cards ...                               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## API Endpoints (Expected)

```
GET    /api/v1/mods                     # Search/list mods
GET    /api/v1/mods/{id}                # Get mod details
GET    /api/v1/mods/{id}/files          # Get mod files
GET    /api/v1/mods/{id}/files/{fid}/links  # Get CDN links
POST   /api/v1/mods/{id}/endorse        # Endorse mod
POST   /api/v1/mods/{id}/rate           # Rate mod
GET    /api/v1/mods/updates             # Batch update check
GET    /api/v1/games/{id}/categories     # Get categories
GET    /api/v1/games/{id}/tags           # Get tags

POST   /oauth/token                     # Token exchange
POST   /oauth/refresh                   # Refresh token
GET    /api/v1/user/me                  # Get current user
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `download-manager` | Receives CDN URLs for mod downloads |
| `mod-installer` | Installs downloaded mods with repository metadata |
| `database` | Stores repository mod IDs for update tracking |
| `update-checker` | Uses repository API to check for mod updates |
| `security-validation` | Validates downloaded mods with SHA256 from API |
| `dependency-resolution` | Fetches dependency info from repository |

## API

```rust
#[tauri::command]
pub async fn repo_auth_start() -> Result<String, String>;

#[tauri::command]
pub async fn repo_auth_callback(
    code: String,
) -> Result<RepositoryAuth, String>;

#[tauri::command]
pub async fn repo_search_mods(
    query: ModSearchQuery,
) -> Result<PaginatedResponse<RepositoryMod>, String>;

#[tauri::command]
pub async fn repo_get_mod(
    mod_id: String,
) -> Result<RepositoryMod, String>;

#[tauri::command]
pub async fn repo_download_mod(
    mod_id: String,
    file_id: String,
    dest: PathBuf,
) -> Result<String, String>;  // Download ID

#[tauri::command]
pub async fn repo_check_updates(
    installed_mods: Vec<ModVersionRef>,
) -> Result<Vec<ModUpdateInfo>, String>;

#[tauri::command]
pub async fn repo_endorse_mod(
    mod_id: String,
) -> Result<(), String>;
```

## Tauri Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `repo_auth_complete` | `RepositoryAuth` | OAuth flow completed |
| `repo_download_started` | `DownloadInfo` | Repository download initiated |
| `repo_updates_available` | `Vec<ModUpdateInfo>` | Mod updates found |

## Rate Limiting & Caching

```rust
struct RateLimiter {
    requests_per_minute: u32,
    requests_remaining: u32,
    reset_at: DateTime<Utc>,
}

// Caching strategy:
// 1. Cache mod metadata (changes infrequently)
// 2. Cache CDN mirror latencies (refresh every 5 min)
// 3. Cache categories and tags (refresh every 1 hour)
// 4. Use ETags for conditional requests
// 5. Implement exponential backoff on 429 responses
```

## Best Practices

1. **CDN selection** — Auto-select fastest mirror, allow manual override
2. **Hash verification** — Always verify SHA256 after download
3. **Token storage** — Store tokens in OS keychain, not in plain text
4. **Respect rate limits** — Cache aggressively, batch requests
5. **Attribution** — Always show mod author and link to mod page
6. **Offline fallback** — Cache mod metadata for offline browsing
7. **Graceful degradation** — App works without repository (local mods)

## Notes

- Repository API is not yet implemented — this document defines expected contract
- CDN mirrors should be geographically distributed for fast downloads
- File hashes (SHA256) are critical for integrity verification
- Consider supporting third-party mod sources via plugin system (future)
- App should work fully offline with locally installed mods
