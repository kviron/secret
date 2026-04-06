import { Component, Show, createEffect, createMemo, createSignal, For } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';
import { launchGame } from '@/shared/lib/launch-game';
import { gameSupportsGamebryoPlugins, gameSupportsKnownSavesLocation } from '@/shared/lib/game-support';
import { steamHeaderImageUrl } from '@/shared/lib/steam-art';
import type { Game } from '@/shared/types';
import './Sidebar.css';

interface GeneralNavItem {
  path: string;
  labelKey: 'nav.games' | 'nav.downloads' | 'nav.settings';
  icon: string;
  end?: boolean;
}

const generalNavItems: GeneralNavItem[] = [
  { path: '/', labelKey: 'nav.games', icon: '🎮', end: true },
  { path: '/downloads', labelKey: 'nav.downloads', icon: '⬇️' },
  { path: '/settings', labelKey: 'nav.settings', icon: '⚙️' },
];

export const Sidebar: Component = () => {
  const { t } = useI18n();
  const { state: gameState } = useGameStore();
  const [collapsed, setCollapsed] = createSignal(false);
  const [artBroken, setArtBroken] = createSignal(false);
  const location = useLocation();

  const managedGame = createMemo(() => {
    const id = gameState.managedGameId;
    if (!id) {
      return undefined;
    }
    return gameState.games.find((g) => g.id === id);
  });

  const coverArtUrl = createMemo(() => {
    const g = managedGame();
    if (!g) {
      return undefined;
    }
    if (g.details.logo) {
      return g.details.logo;
    }
    if (g.launcher === 'steam' && g.details.steamAppId != null) {
      return steamHeaderImageUrl(g.details.steamAppId);
    }
    return undefined;
  });

  createEffect(() => {
    managedGame()?.id;
    setArtBroken(false);
  });

  const toggleTitle = createMemo(() =>
    collapsed() ? t('nav.expandSidebar') : t('nav.collapseSidebar'),
  );

  const isPathActive = (path: string, end?: boolean) => {
    if (path === '/') {
      return location.pathname === '/';
    }
    if (end) {
      return location.pathname === path;
    }
    return location.pathname === path || location.pathname.startsWith(`${path}/`);
  };

  const isGameSubActive = (segment: 'mods' | 'plugins' | 'saves') => {
    const g = managedGame();
    if (!g) {
      return false;
    }
    return location.pathname === `/game/${g.id}/${segment}`;
  };

  const handleLaunch = (game: Game) => {
    void launchGame(game).catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      window.alert(msg);
    });
  };

  const showArt = () => Boolean(coverArtUrl()) && !artBroken();

  return (
    <aside class="sidebar" classList={{ 'sidebar-collapsed': collapsed() }}>
      <div class="sidebar-top">
        <Show
          when={managedGame()}
          fallback={
            <div class="sidebar-brand-fallback">
              <span class="brand-icon" aria-hidden="true">
                ⚔️
              </span>
              <Show when={!collapsed()}>
                <span class="brand-text">Pantheon</span>
              </Show>
            </div>
          }
        >
          {(game) => (
            <div
              class="sidebar-game-banner"
              classList={{ 'sidebar-game-banner--collapsed': collapsed() }}
            >
              <Show when={showArt()}>
                <img
                  class="sidebar-game-banner__bg"
                  src={coverArtUrl()!}
                  alt=""
                  loading="lazy"
                  decoding="async"
                  onError={() => setArtBroken(true)}
                />
              </Show>
              <Show when={!showArt()}>
                <div
                  class="sidebar-game-banner__placeholder"
                  aria-hidden="true"
                >
                  {game().name.charAt(0)}
                </div>
              </Show>
              <div class="sidebar-game-banner__overlay" />
              <div class="sidebar-game-banner__content">
                <Show when={!collapsed()}>
                  <span class="sidebar-game-banner__title">{game().name}</span>
                </Show>
                <button
                  type="button"
                  class="sidebar-game-play"
                  disabled={game().installPathMissing === true}
                  title={t('sidebar.launchGame')}
                  aria-label={t('sidebar.launchGame')}
                  onClick={() => handleLaunch(game())}
                >
                  <svg class="sidebar-game-play__icon" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M8 5v14l11-7z" fill="currentColor" />
                  </svg>
                </button>
              </div>
            </div>
          )}
        </Show>
      </div>

      <nav class="sidebar-nav">
        <div class="sidebar-nav-section">
          <Show when={!collapsed()}>
            <div class="sidebar-nav-section-title">{t('nav.generalSection')}</div>
          </Show>
          <For each={generalNavItems}>
            {(item) => (
              <A
                href={item.path}
                class="nav-item"
                classList={{ 'nav-active': isPathActive(item.path, item.end) }}
                title={collapsed() ? t(item.labelKey) : undefined}
                end={item.end}
              >
                <span class="nav-icon">{item.icon}</span>
                <Show when={!collapsed()}>
                  <span class="nav-label">{t(item.labelKey)}</span>
                </Show>
              </A>
            )}
          </For>
        </div>

        <Show when={managedGame()} keyed>
          {(game) => (
            <div class="sidebar-nav-section sidebar-nav-section--game">
              <Show when={!collapsed()}>
                <div class="sidebar-nav-section-title sidebar-nav-section-title--game">{game.name}</div>
              </Show>
              <A
                href={`/game/${game.id}/mods`}
                class="nav-item"
                classList={{ 'nav-active': isGameSubActive('mods') }}
                title={collapsed() ? t('nav.gameMods') : undefined}
              >
                <span class="nav-icon">📦</span>
                <Show when={!collapsed()}>
                  <span class="nav-label">{t('nav.gameMods')}</span>
                </Show>
              </A>
              <Show when={gameSupportsGamebryoPlugins(game)}>
                <A
                  href={`/game/${game.id}/plugins`}
                  class="nav-item"
                  classList={{ 'nav-active': isGameSubActive('plugins') }}
                  title={collapsed() ? t('nav.gamePlugins') : undefined}
                >
                  <span class="nav-icon">➕</span>
                  <Show when={!collapsed()}>
                    <span class="nav-label">{t('nav.gamePlugins')}</span>
                  </Show>
                </A>
              </Show>
              <Show when={gameSupportsKnownSavesLocation(game)}>
                <A
                  href={`/game/${game.id}/saves`}
                  class="nav-item"
                  classList={{ 'nav-active': isGameSubActive('saves') }}
                  title={collapsed() ? t('nav.gameSaves') : undefined}
                >
                  <span class="nav-icon">💾</span>
                  <Show when={!collapsed()}>
                    <span class="nav-label">{t('nav.gameSaves')}</span>
                  </Show>
                </A>
              </Show>
            </div>
          )}
        </Show>
      </nav>

      <div class="sidebar-footer">
        <button
          type="button"
          class="sidebar-toggle"
          onClick={() => setCollapsed(!collapsed())}
          title={toggleTitle()}
          aria-expanded={!collapsed()}
          aria-label={toggleTitle()}
        >
          <span class="toggle-icon">{collapsed() ? '→' : '←'}</span>
        </button>
      </div>
    </aside>
  );
};
