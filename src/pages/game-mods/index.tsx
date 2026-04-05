import { Component, createEffect, createSignal, Show } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { open } from '@tauri-apps/plugin-dialog';
import { useGameStore } from '@/entities/game';
import { gameApi } from '@/entities/game';
import { useModStore } from '@/entities/mod';
import { ToggleMod } from '@/features/toggle-mod';
import { useI18n } from '@/shared/lib/i18n';
import { Button } from '@/shared/ui/Button';
import { Dialog } from '@/shared/ui/Dialog';
import { GameModsActionToolbar } from './GameModsActionToolbar';

type ModsTab = 'installed' | 'all';

export const GameModsPage: Component = () => {
  const { t } = useI18n();
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { state: gameState, refreshGame, removeGameFromLibrary } = useGameStore();
  const { state: modState, loadMods } = useModStore();
  const [removeOpen, setRemoveOpen] = createSignal(false);
  const [removeBusy, setRemoveBusy] = createSignal(false);
  const [modsTab, setModsTab] = createSignal<ModsTab>('installed');

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
      title: t('gameDetail.dialogSelectInstall'),
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
      <div
        class="game-mods-page"
        classList={{ 'game-mods-page--toolbar-visible': modsTab() === 'installed' }}
      >
        <div class="settings-tabs-wrap">
        <nav class="settings-tabs" aria-label={t('gameMods.tabsAria')}>
          <button
            type="button"
            class="settings-tab"
            classList={{ 'settings-tab--active': modsTab() === 'installed' }}
            onClick={() => setModsTab('installed')}
          >
            {t('gameMods.tab.installed')}
          </button>
          <button
            type="button"
            class="settings-tab"
            classList={{ 'settings-tab--active': modsTab() === 'all' }}
            onClick={() => setModsTab('all')}
          >
            {t('gameMods.tab.all')}
          </button>
        </nav>
        </div>

        <Show when={game()?.installPathMissing}>
        <div class="alert alert-warning game-missing-banner">
          <span class="alert-icon">📁</span>
          <div class="game-missing-banner__text">
            <strong>{t('gameDetail.missingTitle')}</strong>
            <p>{t('gameDetail.missingDesc')}</p>
            <p class="game-missing-path">{game()?.installPath}</p>
          </div>
          <div class="game-missing-actions">
            <Button variant="primary" size="sm" onClick={() => void handleRelink()}>
              {t('gameDetail.relink')}
            </Button>
            <Button variant="danger" size="sm" onClick={() => setRemoveOpen(true)}>
              {t('gameDetail.removeFromLibrary')}
            </Button>
          </div>
        </div>
        </Show>

        <Dialog
        open={removeOpen()}
        onOpenChange={setRemoveOpen}
        title={t('gameDetail.dialogRemoveTitle')}
        description={t('gameDetail.dialogRemoveDesc')}
        actions={
          <>
            <Button variant="secondary" onClick={() => setRemoveOpen(false)}>
              {t('gameDetail.cancel')}
            </Button>
            <Button variant="danger" onClick={() => void handleRemoveConfirm()} isLoading={removeBusy()}>
              {t('gameDetail.remove')}
            </Button>
          </>
        }
      >
        <p>{t('gameDetail.dialogModsCount', { count: modState.mods.length })}</p>
        </Dialog>

        <Show when={modsTab() === 'installed'}>
        <>
          {modState.error && (
            <div class="alert alert-error">
              <span class="alert-icon">⚠️</span>
              {modState.error}
            </div>
          )}

          {modState.mods.length === 0 && !modState.isLoading ? (
            <div class="empty-state">
              <div class="empty-icon">📦</div>
              <h2>{t('gameDetail.emptyModsTitle')}</h2>
              <p>{t('gameDetail.emptyModsDesc')}</p>
            </div>
          ) : (
            <div class="mod-list">
              {modState.mods.map((mod) => (
                <div class="mod-card">
                  <div class="mod-info">
                    <div class="mod-header">
                      <h4>{mod.name}</h4>
                      <span class={`mod-status ${mod.enabled ? 'status-enabled' : 'status-disabled'}`}>
                        {mod.enabled ? `● ${t('gameDetail.modActive')}` : `○ ${t('gameDetail.modInactive')}`}
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
        </Show>

        <Show when={modsTab() === 'all'}>
        <div class="empty-state">
          <div class="empty-icon">🌐</div>
          <h2>{t('gameMods.allPlaceholderTitle')}</h2>
          <p>{t('gameMods.allPlaceholderDesc')}</p>
        </div>
        </Show>
      </div>

      <Show when={modsTab() === 'installed'}>
        <div class="mods-action-toolbar-dock">
          <GameModsActionToolbar />
        </div>
      </Show>
    </>
  );
};
