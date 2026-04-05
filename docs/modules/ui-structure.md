# Module: UI Structure (Solid.js + FSD v2.1)

## Responsibility

Frontend UI architecture using Solid.js with Feature-Sliced Design methodology.

## Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| UI Framework | Solid.js | Fine-grained reactivity, no VDOM |
| Language | TypeScript | Type safety |
| Styling | Panda CSS | CSS-in-JS with theme modes, compile-time generation |
| Design System | 3-tier tokens | OKLCH colors (ui-design-system skill) |
| Routing | @solidjs/router | Client-side routing |
| State | Solid.js Stores | Reactive state management |

## Styling (Panda CSS)

### Theme Modes

```typescript
// Light/Dark theme via CSS variables
import { css } from Panda CSS;

export const button = css({
  bg: '{colors.primary}',
  color: '{colors.primary.foreground}',
  borderRadius: '{radii.md}',
  _dark: {
    bg: '{colors.primary.dark}',
  },
});

// Usage in component
function MyButton(props: ButtonProps) {
  return <button class={button()}>{props.children}</button>;
}
```

### Design Tokens (3-tier)

```typescript
// Tier 1: Primitives (OKLCH)
const primitives = {
  colors: {
    gray: {
      50: 'oklch(97% 0 0)',
      900: 'oklch(25% 0 0)',
    },
    blue: {
      500: 'oklch(55% 0.22 264)',
    },
  },
};

// Tier 2: Semantics
const semantics = {
  colors: {
    background: primitives.colors.gray[50],
    foreground: primitives.colors.gray[900],
    primary: primitives.colors.blue[500],
  },
};

// Tier 3: Components reference semantics
const components = {
  button: {
    bg: '{colors.primary}',
    color: '{colors.primary.foreground}',
  },
};
```

## FSD Layers

FSD uses 6 standardized layers with strict top-down import direction:

```
app/       → App initialization, providers, routing (NO business logic)
pages/     → Route-level composition, owns its own logic
widgets/   → Large composite UI blocks reused across pages
features/  → Reusable user interactions (2+ pages use it)
entities/  → Business domain models (2+ features use it)
shared/    → Infrastructure: UI kit, API client, utils (NO business logic)
```

**Import rule**: A module may ONLY import from layers strictly below it.

## FSD Structure

```
src/
├── app/                    # App initialization
│   ├── App.tsx            # Root component
│   ├── index.tsx          # Entry point
│   ├── providers/
│   │   └── ThemeProvider.tsx  # Dark/light theme
│   └── router/
│       └── index.tsx      # Route definitions
│
├── pages/                  # Route-level (pages/**/index.ts + ui/**)
│   ├── dashboard/
│   │   └── index.tsx       # Games Library: grid of cards, GameCardCover (Steam header art)
│   ├── games/
│   ├── game-detail/
│   │   ├── model/        # Page-specific stores
│   │   │   └── store.ts
│   │   └── ui/
│   │       ├── ModList.tsx
│   │       └── LoadOrderTab.tsx
│   ├── settings/
│   └── downloads/
│
├── widgets/                # Reusable composites (GameCard, ModList)
│   ├── GameCard/
│   │   ├── index.ts
│   │   └── ui/
│   │       └── GameCard.tsx
│   └── ModList/
│       └── ...
│
├── features/               # User interactions (install-mod, toggle-mod)
│   ├── install-mod/
│   │   ├── index.ts
│   │   ├── model/
│   │   │   └── install-store.ts
│   │   └── ui/
│   │       ├── InstallModal.tsx
│   │       └── InstallWizard.tsx
│   └── toggle-mod/
│       └── ...
│
├── entities/               # Business models (game, mod, deployment)
│   ├── game/
│   │   ├── index.ts
│   │   ├── model/
│   │   │   └── game.ts     # Game types + store
│   │   └── api/
│   │       └── games.ts    # Tauri invoke wrapper
│   ├── mod/
│   │   └── ...
│   └── deployment/
│       └── ...
│
└── shared/                # Infrastructure
    ├── ui/                # Button, Input, Modal, Card (Panda CSS)
    │   ├── Button/
    │   ├── Input/
    │   ├── Modal/
    │   └── ...
    ├── api/
    │   ├── client.ts      # Base Tauri invoke wrapper
    │   ├── games.ts
    │   ├── mods.ts
    │   └── deploy.ts
    ├── lib/
    │   ├── format-date.ts
    │   └── ...
    └── config/
        └── routes.ts
```

## Games Library (`pages/dashboard`)

| Concern | Detail |
|---------|--------|
| Entry | `src/pages/dashboard/index.tsx` — grid of game cards, `GameCardCover` for the top banner |
| Images | `src/shared/lib/steam-art.ts` — `steamHeaderImageUrl(appId)` loads Steam **header** art (`header.jpg`, ~460×215). Card header uses CSS `aspect-ratio: 460 / 215` in `src/index.css`. Optional `game.details.logo` overrides with a full `https` URL |
| Data | `entities/game` store — `invoke('get_games')` / detection events; payloads must be camelCase JSON (see MODELS.md) |

## Routing

```typescript
// src/app/router/index.tsx
import { Router, Route } from '@solidjs/router';

export function AppRouter() {
  return (
    <Router>
      <Route path="/" component={Dashboard} />
      <Route path="/games" component={GamesList} />
      <Route path="/games/:id" component={GameDetail} />
      <Route path="/games/:id/mods" component={GameDetail} />
      <Route path="/games/:id/lo" component={LoadOrderPage} />
      <Route path="/settings" component={Settings} />
      <Route path="/downloads" component={Downloads} />
    </Router>
  );
}
```

## State Management (Solid.js Stores)

```typescript
// entities/game/model/game.ts
import { createStore } from 'solid-js/store';

export interface Game {
  id: string;
  name: string;
  installPath: string;
  supportPath: string;
  launcher: 'steam' | 'gog' | 'epic' | 'xbox' | 'manual';
}

interface GameState {
  games: Game[];
  selectedGameId: string | null;
  isLoading: boolean;
}

const [state, setState] = createStore<GameState>({
  games: [],
  selectedGameId: null,
  isLoading: false,
});

export const gameStore = {
  get games() { return state.games; },
  get selectedGame() { 
    return state.games.find(g => g.id === state.selectedGameId);
  },
  
  async loadGames() {
    setState('isLoading', true);
    try {
      const games = await invoke<Game[]>('get_games');
      setState('games', games);
    } finally {
      setState('isLoading', false);
    }
  },
};
```

## Component Pattern

```typescript
// shared/ui/Button/Button.tsx
import { Component, splitProps, JSX } from 'solid-js';
import { button } from './button.css';  // Panda CSS

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ['variant', 'size', 'class']);
  
  return (
    <button 
      class={`${button({ variant: local.variant, size: local.size })} ${local.class || ''}`}
      {...rest}
    >
      {props.children}
    </button>
  );
};
```

## Tauri Integration

```typescript
// shared/api/client.ts
import { invoke } from '@tauri-apps/api/core';

export async function tauriInvoke<T>(
  cmd: string, 
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    console.error(`Command ${cmd} failed:`, error);
    throw error;
  }
}

// shared/api/games.ts
import { tauriInvoke } from './client';

export async function getGames(): Promise<Game[]> {
  return tauriInvoke<Game[]>('get_games');
}

export async function detectGames(): Promise<Game[]> {
  return tauriInvoke<Game[]>('detect_games');
}
```

## Key Interactions

| Module | Interaction |
|--------|-------------|
| `game-detector` | UI shows detected games, triggers detection |
| `mod-installer` | UI shows mod list, triggers install/uninstall |
| `deploy-manager` | UI shows deployment state, toggles mods |
| `load-order-manager` | UI shows drag-drop load order editor |
| `download-manager` | UI shows download queue with progress |

## UI Components to Build

### Core UI Kit (shared/ui)

| Component | Purpose |
|-----------|---------|
| Button | Primary, secondary, ghost, danger variants |
| Input | Text, number, search inputs |
| Modal | Dialog overlay with backdrop |
| Card | Container with title, content, actions |
| Spinner | Loading indicator |
| Progress | Progress bar |
| Toast | Notification toasts |
| Dropdown | Select/menu dropdown |
| Tabs | Tab navigation |

### Page Components

| Page | Components |
|------|------------|
| Dashboard | GameCards, QuickActions, RecentMods |
| GamesList | GameCard grid, SearchBar, FilterDropdown |
| GameDetail | ModList, LoadOrderTab, InstallButton |
| Settings | GeneralSettings, ThemeSettings, PathSettings |
| Downloads | DownloadQueue, DownloadItem, ProgressBars |

## Notes

- Use `splitProps` for component props destructuring
- Use `createResource` for async data fetching
- Use `ErrorBoundary` for error handling in components
- Panda CSS generates atomic CSS at compile time
- All colors should use design tokens, not hardcoded values
- Follow mobile-first responsive design
