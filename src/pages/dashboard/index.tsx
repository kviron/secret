import { Component, onMount, onCleanup } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { DetectGamesButton, DetectionProgress, ScanCustomPathButton } from '@/features/detect-games';
import { Card } from '@/shared/ui/Card';

export const DashboardPage: Component = () => {
  const { state, loadGames, cleanupListeners } = useGameStore();
  const navigate = useNavigate();

  onMount(() => {
    loadGames();
  });

  onCleanup(() => {
    cleanupListeners();
  });

  const handleSelectGame = (gameId: string) => {
    navigate(`/game/${gameId}`);
  };

  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">Games Library</h1>
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
          <h2>No games detected</h2>
          <p>Click "Detect Games" to scan for installed games on your system, or "Add from Folder..." to manually select a game folder.</p>
        </div>
      ) : (
        <div class="game-grid">
          {state.games.map((game) => (
            <Card class="game-card" onClick={() => handleSelectGame(game.id)}>
              <div class="game-card-header">
                <div class="game-icon-placeholder">{game.name.charAt(0)}</div>
              </div>
              <div class="game-card-body">
                <h3>{game.name}</h3>
                <span class={`launcher-badge launcher-${game.launcher}`}>
                  {game.launcher}
                </span>
                <p class="game-path">{game.installPath}</p>
              </div>
            </Card>
          ))}
        </div>
      )}
    </>
  );
};