import { Component, For, onCleanup, onMount } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { DetectGamesButton, DetectionProgress, ScanCustomPathButton } from '@/features/detect-games';
import { useI18n } from '@/shared/lib/i18n';
import { GameLibraryCard } from './GameLibraryCard';

export const DashboardPage: Component = () => {
  const { t, locale } = useI18n();
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
          <For each={state.games}>
            {(game) => (
              <GameLibraryCard
                game={game}
                managedGameId={state.managedGameId}
                t={t}
                locale={locale()}
                onManage={handleManageGame}
              />
            )}
          </For>
        </div>
      )}
    </>
  );
};
