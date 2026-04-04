import { Component, createEffect, onMount } from 'solid-js';
import { useParams } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { useModStore } from '@/entities/mod';
import { InstallModButton } from '@/features/install-mod';
import { ToggleMod } from '@/features/toggle-mod';

export const GameDetailPage: Component = () => {
  const params = useParams<{ id: string }>();
  const { state: gameState } = useGameStore();
  const { state: modState, loadMods } = useModStore();

  const load = () => {
    loadMods(params.id);
  };

  onMount(load);

  createEffect(() => {
    load();
  });

  const game = gameState.games.find((g) => g.id === params.id);

  return (
    <>
      <header class="top-bar">
        <div class="top-bar-left">
          <h1 class="page-title">{game?.name ?? 'Game'}</h1>
          <span class="mod-count">{modState.mods.length} mod{modState.mods.length !== 1 ? 's' : ''}</span>
        </div>
        <InstallModButton gameId={params.id} />
      </header>

      {modState.error && (
        <div class="alert alert-error">
          <span class="alert-icon">⚠️</span>
          {modState.error}
        </div>
      )}

      {modState.mods.length === 0 && !modState.isLoading ? (
        <div class="empty-state">
          <div class="empty-icon">📦</div>
          <h2>No mods installed</h2>
          <p>Click "Install Mod" to add mods from a local archive file.</p>
        </div>
      ) : (
        <div class="mod-list">
          {modState.mods.map((mod) => (
            <div class="mod-card">
              <div class="mod-info">
                <div class="mod-header">
                  <h4>{mod.name}</h4>
                  <span class={`mod-status ${mod.enabled ? 'status-enabled' : 'status-disabled'}`}>
                    {mod.enabled ? '● Active' : '○ Inactive'}
                  </span>
                </div>
                <div class="mod-meta">
                  <span class="mod-type">{mod.modType}</span>
                  {mod.version && <span class="mod-version">v{mod.version}</span>}
                </div>
              </div>
              <ToggleMod mod={mod} onToggle={load} />
            </div>
          ))}
        </div>
      )}
    </>
  );
};
