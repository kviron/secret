import { Component, createMemo, createSignal, onMount, onCleanup, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { DetectGamesButton, DetectionProgress, ScanCustomPathButton } from '@/features/detect-games';
import { getGameModStatusBadge, isGameUnsupportedByPantheon } from '@/shared/lib/game-support';
import { useI18n } from '@/shared/lib/i18n';
import { launchGame } from '@/shared/lib/launch-game';
import { steamHeaderImageUrl } from '@/shared/lib/steam-art';
import type { Game } from '@/shared/types';
import { Button } from '@/shared/ui/Button';
import { Card } from '@/shared/ui/Card';

const GameCardCover: Component<{ game: Game }> = (props) => {
  const artUrl = createMemo(() => {
    const g = props.game;
    if (g.details.logo) return g.details.logo;
    if (g.launcher === 'steam' && g.details.steamAppId != null) {
      return steamHeaderImageUrl(g.details.steamAppId);
    }
    return undefined;
  });

  const [artBroken, setArtBroken] = createSignal(false);

  const showArt = () => Boolean(artUrl()) && !artBroken();

  return (
    <div class="game-card-header">
      <Show when={showArt()}>
        <img
          class="game-card-art"
          src={artUrl()!}
          alt=""
          loading="lazy"
          decoding="async"
          onError={() => setArtBroken(true)}
        />
      </Show>
      <Show when={!showArt()}>
        <div class="game-icon-placeholder">{props.game.name.charAt(0)}</div>
      </Show>
    </div>
  );
};

export const DashboardPage: Component = () => {
  const { t } = useI18n();
  const { state, loadGames, cleanupListeners, setManagedGame } = useGameStore();
  const navigate = useNavigate();

  onMount(() => {
    loadGames();
  });

  onCleanup(() => {
    cleanupListeners();
  });

  const handleManageGame = (gameId: string) => {
    setManagedGame(gameId);
    navigate(`/game/${gameId}/mods`);
  };

  const handleLaunchGame = (game: Game) => {
    void launchGame(game).catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(err);
      window.alert(msg);
    });
  };

  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">{t('dashboard.title')}</h1>
        <div class="detect-actions">
          <DetectGamesButton onDetected={loadGames} />
          <ScanCustomPathButton />
        </div>
      </header>

      <DetectionProgress />

      {state.error && (
        <div class="alert alert-error">
          <span class="alert-icon">⚠️</span>
          {state.error}
        </div>
      )}

      {state.games.length === 0 && !state.isLoading && !state.isDetecting ? (
        <div class="empty-state">
          <div class="empty-icon">🎮</div>
          <h2>{t('dashboard.emptyTitle')}</h2>
          <p>{t('dashboard.emptyDescription')}</p>
        </div>
      ) : (
        <div class="game-grid">
          {state.games.map((game) => {
            const modBadge = getGameModStatusBadge(game);
            const unsupported = isGameUnsupportedByPantheon(game);
            return (
              <Card
                class={`game-card${game.installPathMissing ? ' game-card--missing' : ''}`}
                hoverable={!unsupported}
                onClick={unsupported ? undefined : () => handleManageGame(game.id)}
              >
                <GameCardCover game={game} />
                <div class="game-card-body">
                  <h3>{game.name}</h3>
                  <div class="game-meta">
                    <span class={`launcher-badge launcher-${game.launcher}`}>
                      {game.launcher}
                    </span>
                    {game.installPathMissing && (
                      <span class="game-meta-badge game-meta-badge--missing-path">
                        {t('dashboard.installPathMissing')}
                      </span>
                    )}
                    <span
                      class={`game-meta-badge game-meta-badge--${modBadge.variant}`}
                    >
                      {t(modBadge.labelKey)}
                    </span>
                  </div>
                  <p class="game-path">{game.installPath}</p>
                  <Show when={game.installPathMissing !== true}>
                    <div
                      class="game-card-actions"
                      onClick={(e) => e.stopPropagation()}
                      role="presentation"
                    >
                      <Show when={!unsupported}>
                        <Button
                          variant="primary"
                          size="sm"
                          class="game-card-manage"
                          onClick={() => handleManageGame(game.id)}
                        >
                          {t('dashboard.manage')}
                        </Button>
                      </Show>
                      <Button
                        variant={unsupported ? 'primary' : 'secondary'}
                        size="sm"
                        class="game-card-launch"
                        onClick={() => handleLaunchGame(game)}
                      >
                        {t('dashboard.launch')}
                      </Button>
                    </div>
                  </Show>
                </div>
              </Card>
            );
          })}
        </div>
      )}
    </>
  );
};
