import { Component, createEffect, createSignal, Show } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { open } from '@tauri-apps/plugin-dialog';
import { useGameStore } from '@/entities/game';
import { gameApi } from '@/entities/game';
import { useModStore } from '@/entities/mod';
import { InstallModButton } from '@/features/install-mod';
import { ToggleMod } from '@/features/toggle-mod';
import { Button } from '@/shared/ui/Button';
import { Dialog } from '@/shared/ui/Dialog';

export const GameDetailPage: Component = () => {
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { state: gameState, refreshGame, removeGameFromLibrary } = useGameStore();
  const { state: modState, loadMods } = useModStore();
  const [removeOpen, setRemoveOpen] = createSignal(false);
  const [removeBusy, setRemoveBusy] = createSignal(false);

  const handleBack = () => {
    if (typeof window !== 'undefined' && window.history.length > 1) {
      window.history.back();
    } else {
      navigate('/');
    }
  };

  const load = () => {
    loadMods(params.id);
  };

  createEffect(() => {
    const id = params.id;
    void refreshGame(id);
    loadMods(id);
  });

  const game = () => gameState.games.find((g) => g.id === params.id);

  const handleRelink = async () => {
    const g = game();
    if (!g) {
      return;
    }
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Выберите папку установки игры',
    });
    if (selected && typeof selected === 'string') {
      await gameApi.registerGame({
        ...g,
        installPath: selected,
        supportPath: selected,
      });
      await refreshGame(params.id);
    }
  };

  const handleRemoveConfirm = async () => {
    setRemoveBusy(true);
    try {
      await removeGameFromLibrary(params.id);
      setRemoveOpen(false);
      navigate('/');
    } finally {
      setRemoveBusy(false);
    }
  };

  return (
    <>
      <header class="top-bar">
        <div class="top-bar-left">
          <button
            type="button"
            class="btn-back"
            onClick={() => handleBack()}
            title="Назад к библиотеке"
            aria-label="Назад к библиотеке"
          >
            ←
          </button>
          <h1 class="page-title">{game()?.name ?? 'Game'}</h1>
          <span class="mod-count">
            {modState.mods.length} mod{modState.mods.length !== 1 ? 's' : ''}
          </span>
        </div>
        <InstallModButton
          gameId={params.id}
          onInstalled={load}
          disabled={game()?.installPathMissing === true}
        />
      </header>

      <Show when={game()?.installPathMissing}>
        <div class="alert alert-warning game-missing-banner">
          <span class="alert-icon">📁</span>
          <div class="game-missing-banner__text">
            <strong>Папка установки не найдена.</strong>
            <p>
              Запись игры и каталог модов в Pantheon сохранены. Укажите новую папку или удалите игру из
              библиотеки (моды в базе будут удалены).
            </p>
            <p class="game-missing-path">{game()?.installPath}</p>
          </div>
          <div class="game-missing-actions">
            <Button variant="primary" size="sm" onClick={() => void handleRelink()}>
              Указать папку…
            </Button>
            <Button variant="danger" size="sm" onClick={() => setRemoveOpen(true)}>
              Удалить из библиотеки…
            </Button>
          </div>
        </div>
      </Show>

      <Dialog
        open={removeOpen()}
        onOpenChange={setRemoveOpen}
        title="Удалить игру из библиотеки?"
        description="Из базы Pantheon будут удалены связанные моды и состояние деплоя."
        actions={
          <>
            <Button variant="secondary" onClick={() => setRemoveOpen(false)}>
              Отмена
            </Button>
            <Button variant="danger" onClick={() => void handleRemoveConfirm()} isLoading={removeBusy()}>
              Удалить
            </Button>
          </>
        }
      >
        <p>
          Будет удалено модов в каталоге: <strong>{modState.mods.length}</strong>.
        </p>
      </Dialog>

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
