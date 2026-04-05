import { Component, Show, createMemo, createResource } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { gameApi } from '@/entities/game';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';
import { gameSupportsGamebryoPlugins } from '@/shared/lib/game-support';

export const GamePluginsPage: Component = () => {
  const { t } = useI18n();
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { state: gameState } = useGameStore();

  const game = () => gameState.games.find((g) => g.id === params.id);

  const supportsPlugins = createMemo(() => {
    const g = game();
    return g !== undefined && gameSupportsGamebryoPlugins(g);
  });

  const [plugins] = createResource(
    () => {
      const id = params.id;
      const g = gameState.games.find((x) => x.id === id);
      if (!id || !g || !gameSupportsGamebryoPlugins(g)) {
        return undefined;
      }
      return id;
    },
    async (id) => gameApi.listGamePlugins(id),
  );

  const handleBack = () => {
    navigate(`/game/${params.id}/mods`);
  };

  return (
    <>
      <header class="top-bar">
        <div class="top-bar-left">
          <button type="button" class="btn-back" onClick={() => handleBack()} title={t('pluginsPage.backTitle')}>
            ←
          </button>
          <h1 class="page-title">{t('pluginsPage.title')}</h1>
          <span class="mod-count">{game()?.name ?? ''}</span>
        </div>
      </header>

      <Show when={!supportsPlugins()}>
        <div class="empty-state">
          <div class="empty-icon">➕</div>
          <h2>{t('pluginsPage.unsupportedTitle')}</h2>
          <p>{t('pluginsPage.unsupportedDesc')}</p>
        </div>
      </Show>

      <Show when={supportsPlugins() && plugins.error}>
        <div class="alert alert-error">
          <span class="alert-icon">⚠️</span>
          {String(plugins.error)}
        </div>
      </Show>

      <Show when={supportsPlugins() && plugins.loading}>
        <p class="text-muted">{t('pluginsPage.loading')}</p>
      </Show>

      <Show when={supportsPlugins() && !plugins.loading && !plugins.error}>
        <Show
          when={(plugins() ?? []).length > 0}
          fallback={
            <div class="empty-state">
              <div class="empty-icon">➕</div>
              <h2>{t('pluginsPage.emptyTitle')}</h2>
              <p>{t('pluginsPage.emptyDesc')}</p>
            </div>
          }
        >
          <ul class="plugin-file-list">
            {(plugins() ?? []).map((name) => (
              <li class="plugin-file-item">{name}</li>
            ))}
          </ul>
        </Show>
      </Show>
    </>
  );
};
