# Module: Download Manager

## Responsibility

Handles all file downloads with pause/resume capability, chunked parallel downloads, progress tracking, and download queue management.

## Pantheon Comparison

Pantheon `download_management` extension handles:
- Download queue with concurrency control
- Chunked downloads with pause/resume (range requests)
- URL resolution from mod download pages
- File validation after download
- Download progress reporting

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Download Manager Architecture                       │
└─────────────────────────────────────────────────────────────────────────┘

    ┌──────────────────────────────────────────────────────────────────┐
    │                     DownloadManager                               │
    │                                                                   │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │
    │  │  Queue     │  │  Resolver   │  │  ProgressTracker        │   │
    │  │  Manager   │  │  (URL → URL)│  │  (events → UI)         │   │
    │  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘   │
    │         │                │                      │                 │
    │         └────────────────┼──────────────────────┘                 │
    │                          ▼                                        │
    │  ┌─────────────────────────────────────────────────────────────┐  │
    │  │                   Download Strategy                         │  │
    │  │                                                              │  │
    │  │  ┌─────────────────────┐   ┌────────────────────────────┐ │  │
    │  │  │  ChunkedDownloader  │   │  SingleStreamDownloader   │ │  │
    │  │  │  (Range requests)   │   │  (No range support)       │ │  │
    │  │  │  Parallel chunks    │   │  Streaming to file        │ │  │
    │  │  └─────────────────────┘   └────────────────────────────┘ │  │
    │  └─────────────────────────────────────────────────────────────┘  │
    │                          │                                        │
    │                          ▼                                        │
    │                   ┌─────────────┐                                │
    │                   │   File      │                                │
    │                   │   Writer    │                                │
    │                   └─────────────┘                                │
    └──────────────────────────────────────────────────────────────────┘
```

## Download Flow

```
download(resource, dest)
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 1. Resolve URL                                             │
│    └──► Resolver converts mod page URL to actual file URL  │
└────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 2. Probe Server (HEAD request)                            │
│    ├──► Check Accept-Ranges header                        │
│    ├──► Get Content-Length                                │
│    └──► Get ETag for validation                           │
└────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 3. Choose Strategy                                          │
│    ├──► Range supported ──► ChunkedDownloader              │
│    └──► No range ─────────► SingleStreamDownloader         │
└────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 4. Download with Progress                                    │
│    ├──► Update progress (bytesWritten, bytesTotal)          │
│    ├──► Emit progress events                               │
│    └──► Handle network errors with retry                   │
└────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 5. Validate Download                                        │
│    ├──► Verify ETag if available                          │
│    ├──► Check file size                                   │
│    └──► Optional hash verification                        │
└────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ 6. Return Download Result                                    │
│    └──► File path, final size, ETag                        │
└────────────────────────────────────────────────────────────┘
```

## Data Model

```rust
struct Download {
    id: String,                    // UUID
    file_name: String,             // Original filename
    url: String,                   // Current download URL
    destination: PathBuf,          // Final destination path
    state: DownloadState,
    bytes_written: u64,            // Progress
    bytes_total: Option<u64>,       // Total size (None = unknown)
    etag: Option<String>,          // For validation
    error: Option<String>,         // Error message if failed
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    chunks: Vec<ByteRange>,        // For resume support
}

enum DownloadState {
    Pending,      // Queued, not started
    Running,      // Currently downloading
    Paused,       // User paused
    Completed,    // Successfully downloaded
    Failed,       // Error occurred
    Cancelled,    // User cancelled
}

struct ByteRange {
    start: u64,
    end: u64,
    completed: bool,
}

struct DownloadProgress {
    id: String,
    bytes_written: u64,
    bytes_total: Option<u64>,
    speed: u64,              // bytes per second
    eta_seconds: Option<u64>,
}

struct DownloadResult {
    path: PathBuf,
    size: u64,
    etag: Option<String>,
}
```

## Resolver Plugin System

Pantheon uses a resolver system to extract download URLs from various sources:

```rust
pub trait DownloadResolver: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    
    // Test if this resolver can handle the resource
    fn can_resolve(&self, resource: &str) -> bool;
    
    // Resolve resource to actual download URL
    fn resolve(&self, resource: &str) -> Result<String, String>;
}

struct ResolverRegistry {
    resolvers: Vec<Arc<dyn DownloadResolver>>,
}

impl ResolverRegistry {
    pub fn resolve(&self, resource: &str) -> Result<String, String> {
        for resolver in &self.resolvers {
            if resolver.can_resolve(resource) {
                return resolver.resolve(resource);
            }
        }
        Err("No resolver found for resource".to_string())
    }
}
```

### Built-in Resolvers

| Resolver | Description | Example |
|----------|-------------|---------|
| `url` | Direct URL | `https://example.com/mod.zip` |
| `repository` | Pantheon Repository API | `pantheon://mods/123` |
| `moddb` | ModDB integration | `moddb.com/mods/123` |

## Chunked Download

For servers that support range requests (pause/resume):

```rust
struct ChunkedDownloader {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,   // Concurrency control (tokio)
    chunk_size: u64,             // Default 5MB
}

impl ChunkedDownloader {
    pub async fn download(
        &self,
        url: &str,
        dest: &Path,
        checkpoint: Option<DownloadCheckpoint>,
    ) -> Result<DownloadResult, DownloadError> {
        // 1. Get file size via HEAD request
        let file_size = self.get_file_size(url).await?;
        
        // 2. Calculate chunks
        let chunks = self.calculate_chunks(file_size, self.chunk_size);
        
        // 3. Resume from checkpoint if provided
        let pending_chunks = self.pending_chunks(chunks, checkpoint);
        
        // 4. Download chunks in parallel (tokio tasks + Semaphore)
        let handles: Vec<_> = pending_chunks
            .into_iter()
            .map(|chunk| {
                let url = url.to_string();
                let client = self.client.clone();
                let permit = self.semaphore.clone();
                tokio::spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    Self::download_chunk(&client, &url, chunk).await
                })
            })
            .collect();
        
        let results = join_all(handles).await;
        
        // 5. Combine chunks into file
        self.combine_chunks(dest, results).await
    }
}
```

## Queue Management

```rust
struct DownloadQueue {
    downloads: Arc<Mutex<HashMap<String, Download>>>,
    max_concurrent: usize,        // Default 3
    max_retries: u32,             // Default 3
}

impl DownloadQueue {
    pub async fn add(&self, download: Download) -> String { ... }
    pub async fn pause(&self, id: &str) -> Result<(), String> { ... }
    pub async fn resume(&self, id: &str) -> Result<(), String> { ... }
    pub async fn cancel(&self, id: &str) -> Result<(), String> { ... }
    pub async fn get_progress(&self, id: &str) -> Result<DownloadProgress, String> { ... }
    pub fn get_state(&self, id: &str) -> DownloadState { ... }
}
```

## Error Handling

```rust
enum DownloadError {
    Cancellation,
    ResolverError(String),
    NetworkError(String),
    FileSystemError(String),
    ValidationError(String),
    ServerError(u16),         // HTTP 4xx/5xx
    Timeout,
}

impl DownloadError {
    pub fn is_retryable(&self) -> bool {
        match self {
            DownloadError::NetworkError(_) => true,
            DownloadError::Timeout => true,
            DownloadError::ServerError(500..=599) => true,
            _ => false,
        }
    }
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `database` | Stores download records, tracks state |
| `mod-installer` | Downloads are mod archives ready for installation |
| `settings` | User preferences for concurrent downloads |
| `ui` | Progress events for UI updates |

## API

```rust
#[tauri::command]
pub async fn start_download(
    resource: String,
    dest: PathBuf,
) -> Result<String, String>;  // Returns download ID

#[tauri::command]
pub async fn pause_download(id: String) -> Result<(), String>;

#[tauri::command]
pub async fn resume_download(id: String) -> Result<(), String>;

#[tauri::command]
pub async fn cancel_download(id: String) -> Result<(), String>;

#[tauri::command]
pub async fn get_download_progress(id: String) -> Result<DownloadProgress, String>;

#[tauri::command]
pub async fn list_downloads() -> Result<Vec<Download>, String>;

#[tauri::command]
pub async fn clear_completed_downloads() -> Result<(), String>;
```

## Notes

- Chunk size should be configurable (balance between parallelism and overhead)
- ETag validation allows resume without re-downloading
- Some servers don't support range requests (no pause/resume)
- Consider download bandwidth limits for users
- File validation should happen after download completes
- Queue should persist across app restarts
