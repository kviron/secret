# Pantheon Deep Linking & URL Schemes

## Overview

Pantheon supports custom URL schemes for inter-app communication, mod installation from external sources, OAuth authentication callbacks, and deep linking from websites.

## URL Schemes

### Supported Schemes

| Scheme | Purpose | Example |
|--------|---------|---------|
| `pantheon://` | Primary app protocol | `pantheon://mod/install?game=skyrim&id=12345` |
| `pantheon-api://` | API callbacks (OAuth) | `pantheon-api://oauth/callback?code=xxx&state=yyy` |
| `https://pantheon.app/` | Web fallback (desktop) | `https://pantheon.app/mod/12345` |

### Scheme Registration

On Windows/Linux, schemes are registered via Tauri configuration. On macOS, use CFBundleURLTypes in the app bundle.

## URL Format

### Mod Installation

```
pantheon://mod/install
pantheon://mod/install?game={gameId}&id={modId}
pantheon://mod/install?game=skyrim&id=12345
```

**Parameters**:
| Parameter | Required | Description |
|-----------|----------|-------------|
| `game` | Yes | Game ID (e.g., `skyrim`, `fallout4`) |
| `id` | Yes | Mod ID in repository |
| `file` | No | Specific file ID (defaults to latest) |
| `version` | No | Pin to specific version |

**Actions**:
1. Parse URL and validate parameters
2. Check if user is logged in (if auth required)
3. Add mod to download queue
4. Optionally trigger installation after download

### Mod Download

```
pantheon://mod/download?game={gameId}&id={modId}
pantheon://mod/download?game=skyrim&id=12345&file=67890
```

**Parameters**:
| Parameter | Required | Description |
|-----------|----------|-------------|
| `game` | Yes | Game ID |
| `id` | Yes | Mod ID |
| `file` | No | Specific file ID |

### Collection Install

```
pantheon://collection/install?game={gameId}&id={collectionId}
pantheon://collection/install?game=skyrim&id=abc123
```

### Game Launch

```
pantheon://game/launch?game={gameId}
pantheon://game/launch?game=skyrim&profile=modded
```

### Profile Switch

```
pantheon://profile/switch?id={profileId}
pantheon://profile/switch?name=Vampire+Build
```

### Settings

```
pantheon://settings/open?page={page}
pantheon://settings/open?page=downloads
```

## OAuth Flow

### Authorization Request

```
pantheon-api://oauth/authorize
  ?client_id={clientId}
  &redirect_uri={encodedRedirectUri}
  &state={randomState}
  &scope={scopes}
```

### Callback

```
pantheon-api://oauth/callback
  ?code={authorizationCode}
  &state={originalState}
```

### Token Response

After validation, the app stores the token and emits an event to the frontend.

## Implementation

### Rust Backend (src-tauri)

#### URL Handler Command

```rust
// src-tauri/src/commands/protocol.rs

#[tauri::command]
pub async fn handle_url(url: String) -> Result<ProtocolAction, String> {
    let parsed = url.parse::<ProtocolUrl>()
        .map_err(|e| format!("Invalid URL: {}", e))?;
    
    match parsed.scheme() {
        "pantheon" => handle_pantheon_url(parsed).await,
        "pantheon-api" => handle_api_url(parsed).await,
        _ => Err(format!("Unknown scheme: {}", parsed.scheme()))
    }
}

pub enum ProtocolAction {
    InstallMod {
        game_id: String,
        mod_id: String,
        file_id: Option<String>,
        version: Option<String>,
    },
    DownloadMod {
        game_id: String,
        mod_id: String,
        file_id: Option<String>,
    },
    InstallCollection {
        game_id: String,
        collection_id: String,
    },
    LaunchGame {
        game_id: String,
        profile: Option<String>,
    },
    SwitchProfile {
        profile_id: Option<String>,
        profile_name: Option<String>,
    },
    OpenSettings {
        page: Option<String>,
    },
    OAuthCallback {
        code: String,
        state: String,
    },
}
```

#### Deep Link Event Emission

```rust
// src-tauri/src/lib.rs

use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // Handle deep link from single instance
            if argv.len() > 1 {
                let url = &argv[1];
                if url.starts_with("pantheon://") || url.starts_with("pantheon-api://") {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = handle.deep_link_event(url) {
                            log::error!("Failed to handle deep link: {}", e);
                        }
                    });
                }
            }
        }))
        .setup(|app| {
            // Register protocol handlers
            #[cfg(target_os = "windows")]
            {
                app.register_window_protocol("pantheon", |window, request| {
                    // Handle pantheon:// URLs
                })?;
            }
            Ok(())
        })
        .build()
        .unwrap()
}
```

### Frontend (Solid.js)

#### URL Event Handler

```typescript
// src/app/router/DeepLinkHandler.tsx

import { onMount, onCleanup } from 'solid-js';
import { listen } from '@tauri-apps/api/event';
import { useNavigate } from '@solidjs/router';
import { api } from '@/shared/api/client';

export function DeepLinkHandler() {
  const navigate = useNavigate();
  
  onMount(async () => {
    // Listen for deep link events from Rust
    const unlisten = await listen<ProtocolAction>('deep-link', (event) => {
      handleProtocolAction(event.payload);
    });
    
    onCleanup(() => unlisten());
  });
  
  const handleProtocolAction = (action: ProtocolAction) => {
    switch (action.type) {
      case 'InstallMod':
        navigate(`/games/${action.gameId}/mods?install=${action.modId}`);
        break;
      case 'DownloadMod':
        navigate(`/downloads?add=${action.modId}&game=${action.gameId}`);
        break;
      case 'InstallCollection':
        navigate(`/games/${action.gameId}/collections/${action.collectionId}`);
        break;
      case 'LaunchGame':
        invoke('launch_game', { 
          gameId: action.gameId, 
          profileId: action.profile 
        });
        break;
      case 'SwitchProfile':
        invoke('switch_profile_by_name', { name: action.profileName });
        break;
      case 'OpenSettings':
        navigate(`/settings?page=${action.page || ''}`);
        break;
    }
  };
  
  return null; // Component doesn't render anything
}
```

### URL Parser

```typescript
// src/shared/lib/urlParser.ts

export interface ProtocolUrl {
  scheme: string;
  host: string;
  path: string;
  params: Record<string, string>;
}

export function parseProtocolUrl(url: string): ProtocolUrl {
  try {
    const parsed = new URL(url);
    const params: Record<string, string> = {};
    parsed.searchParams.forEach((value, key) => {
      params[key] = value;
    });
    
    return {
      scheme: parsed.protocol.replace(':', ''),
      host: parsed.host,
      path: parsed.pathname.replace(/^\/+/, ''),
      params,
    };
  } catch {
    throw new Error(`Invalid URL: ${url}`);
  }
}

export function buildModInstallUrl(
  gameId: string, 
  modId: string, 
  fileId?: string
): string {
  const params = new URLSearchParams({ game: gameId, id: modId });
  if (fileId) params.set('file', fileId);
  return `pantheon://mod/install?${params.toString()}`;
}

export function buildCollectionInstallUrl(
  gameId: string, 
  collectionId: string
): string {
  const params = new URLSearchParams({ game: gameId, id: collectionId });
  return `pantheon://collection/install?${params.toString()}`;
}
```

## Configuration

### Tauri Configuration (tauri.conf.json)

```json
{
  "plugins": {
    "deep-link": {
      "schemes": ["pantheon", "pantheon-api"]
    }
  }
}
```

### Single Instance

```json
{
  "app": {
    "windows": [...],
    "security": {
      "capabilities": ["single-instance"]
    }
  }
}
```

### Cargo Dependencies (Cargo.toml)

```toml
[dependencies]
tauri-plugin-deep-link = "2"
tauri-plugin-single-instance = "2"
```

### NPM Dependencies

```bash
npm install @tauri-apps/plugin-deep-link
```

## Security

### URL Validation

1. **Scheme validation**: Only accept registered schemes
2. **Host validation**: Ensure known hosts
3. **Parameter validation**: Validate all query parameters
4. **State validation** (OAuth): Verify state parameter matches

### Security Checks

```rust
fn validate_mod_url(parsed: &ProtocolUrl) -> Result<(), String> {
    // Must be pantheon scheme
    if parsed.scheme != "pantheon" {
        return Err("Invalid scheme".to_string());
    }
    
    // Must be mod/install path
    if parsed.path != "mod/install" {
        return Err("Invalid path".to_string());
    }
    
    // Validate game ID
    let game_id = parsed.params.get("game")
        .ok_or("Missing game parameter")?;
    if !is_valid_game_id(game_id) {
        return Err("Invalid game ID".to_string());
    }
    
    // Validate mod ID format
    let mod_id = parsed.params.get("id")
        .ok_or("Missing mod ID")?;
    if !is_valid_mod_id(mod_id) {
        return Err("Invalid mod ID".to_string());
    }
    
    Ok(())
}
```

## Events

### Deep Link Events (Rust → Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `deep-link` | `ProtocolAction` | URL was activated |
| `oauth-callback` | `OAuthResult` | OAuth flow completed |
| `install-request` | `InstallRequest` | Mod install requested |
| `download-request` | `DownloadRequest` | Mod download requested |

### Frontend Events

| Event | Payload | Description |
|-------|---------|-------------|
| `protocol-action` | `ProtocolAction` | Dispatch to appropriate handler |

## Error Handling

### Invalid URL

```typescript
interface ProtocolError {
  code: 'INVALID_URL' | 'MISSING_PARAMS' | 'UNKNOWN_ACTION' | 'AUTH_REQUIRED';
  message: string;
  url: string;
}
```

### Error Responses

| Code | HTTP Equivalent | Description |
|------|-----------------|-------------|
| `INVALID_URL` | 400 | URL format is invalid |
| `MISSING_PARAMS` | 400 | Required parameters missing |
| `UNKNOWN_ACTION` | 404 | Unknown action type |
| `AUTH_REQUIRED` | 401 | Action requires authentication |
| `GAME_NOT_FOUND` | 404 | Unknown game ID |
| `MOD_NOT_FOUND` | 404 | Mod doesn't exist |

## Use Cases

### 1. Website "Install with Pantheon" Button

```html
<a href="pantheon://mod/install?game=skyrim&id=12345">
  <img src="install-button.png" alt="Install with Pantheon">
</a>
```

### 2. Nexus Mods Integration

When user clicks "Download with Manager" on Nexus:
- Nexus site redirects to `pantheon://mod/download?game=skyrim&id=12345`
- Pantheon opens, validates URL, starts download

### 3. OAuth Login Flow

```
1. User clicks "Login" on Pantheon website
2. Website redirects to `pantheon-api://oauth/authorize?client_id=xxx&...`
3. Pantheon opens, shows login dialog
4. On success, redirects to `pantheon-api://oauth/callback?code=xxx&state=yyy`
5. App receives callback, exchanges code for token
```

### 4. Collection Sharing

```html
<a href="pantheon://collection/install?game=skyrim&id=abc123">
  Install Skyrim Graphics Collection
</a>
```

## Testing

### Manual Testing

```bash
# Windows
start pantheon://mod/install?game=skyrim&id=12345

# Linux
xdg-open "pantheon://mod/install?game=skyrim&id=12345"

# macOS
open "pantheon://mod/install?game=skyrim&id=12345"
```

### Automated Tests

```typescript
describe('URL Protocol Handling', () => {
  it('should parse mod install URL', () => {
    const url = 'pantheon://mod/install?game=skyrim&id=12345';
    const parsed = parseProtocolUrl(url);
    
    expect(parsed.scheme).toBe('pantheon');
    expect(parsed.path).toBe('mod/install');
    expect(parsed.params.game).toBe('skyrim');
    expect(parsed.params.id).toBe('12345');
  });
  
  it('should handle missing optional parameters', () => {
    const url = 'pantheon://mod/install?game=skyrim&id=12345';
    const parsed = parseProtocolUrl(url);
    
    expect(parsed.params.file).toBeUndefined();
    expect(parsed.params.version).toBeUndefined();
  });
});
```

## Browser Integration

### Register as Protocol Handler (Web)

```typescript
// In web app
if ('registerProtocolHandler' in navigator) {
  navigator.registerProtocolHandler(
    'pantheon',
    'https://pantheon.app/handle?url=%s',
    'Pantheon Mod Manager'
  );
}
```

### Fallback for Unsupported Browsers

On browsers that don't support custom protocol handlers:

```html
<a href="https://pantheon.app/mod/12345" 
   onclick="return openPantheonApp()">
  Install Mod
</a>

<script>
function openPantheonApp() {
  // Try opening app
  window.location.href = 'pantheon://mod/install?game=skyrim&id=12345';
  
  // Fallback: show instructions after 500ms
  setTimeout(() => {
    if (document.hidden) {
      showInstallInstructions();
    }
  }, 500);
  
  return false;
}
</script>
```

## Platform-Specific

### Windows

- Registry entries for protocol handler
- User prompted "Open with Pantheon?" on first use
- `HKEY_CLASSES_ROOT\pantheon`

### macOS

- CFBundleURLTypes in Info.plist
- Finder "Open with" integration
- System Preferences → General → Default apps

### Linux

- .desktop file with MimeType
- Shared MIME database
- `xdg-open` integration