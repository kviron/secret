import { Component, Show, createMemo, createResource } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { gameApi } from '@/entities/game';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';
import { gameSupportsKnownSavesLocation } from '@/shared/lib/game-support';

export const GameSavesPage: Component = () => {
  const { t } = useI18n();
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { state: gameState } = useGameStore();

  const game = () => gameState.games.find((g) => g.id === params.id);

  const supportsSaves = createMemo(() => {
    const g = game();
    return g !== undefined && gameSupportsKnownSavesLocation(g);
  });

  const [saves] = createResource(
    () => {
      const id = params.id;
      const g = gameState.games.find((x) => x.id === id);
      if (!id || !g || !gameSupportsKnownSavesLocation(g)) {
        return undefined;
      }
      return id;
    },
    async (id) => gameApi.listGameSaves(id),
  );

  const handleBack = () => {
    navigate(`/game/${params.id}/mods`);
  };

  return (
    <>
      <header class="top-bar">
        <div class="top-bar-left">
          <button type="button" class="btn-back" onClick={() => handleBack()} title={t('savesPage.backTitle')}>
            ←
          </button>
          <h1 class="page-title">{t('savesPage.title')}</h1>
          <span class="mod-count">{game()?.name ?? ''}</span>
        </div>
      </header>

      <Show when={!supportsSaves()}>
        <div class="empty-state">
          <div class="empty-icon">💾</div>
          <h2>{t('savesPage.unsupportedTitle')}</h2>
          <p>{t('savesPage.unsupportedDesc')}</p>
        </div>
      </Show>

      <Show when={supportsSaves() && saves.error}>
        <div class="alert alert-error">
          <span class="alert-icon">⚠️</span>
          {String(saves.error)}
        </div>
      </Show>

      <Show when={supportsSaves() && saves.loading}>
        <p class="text-muted">{t('savesPage.loading')}</p>
      </Show>

      <Show when={supportsSaves() && !saves.loading && !saves.error}>
        <Show
          when={(saves() ?? []).length > 0}
          fallback={
            <div class="empty-state">
              <div class="empty-icon">💾</div>
              <h2>{t('savesPage.emptyTitle')}</h2>
              <p>{t('savesPage.emptyDesc')}</p>
            </div>
          }
        >
          <ul class="save-file-list">
            {(saves() ?? []).map((entry) => (
              <li class="save-file-item">
                <span class="save-file-name">{entry.name}</span>
                <span class="save-file-path">{entry.path}</span>
              </li>
            ))}
          </ul>
        </Show>
      </Show>
    </>
  );
};
