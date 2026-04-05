import { Component, Show } from 'solid-js';
import { Navigate } from '@solidjs/router';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';

/** Redirects to `/game/:managedGameId/mods` when a managed game is set; otherwise empty state. */
export const ModsPage: Component = () => {
  const { t } = useI18n();
  const { state } = useGameStore();

  return (
    <Show
      when={state.managedGameId}
      fallback={
        <>
          <header class="top-bar">
            <h1 class="page-title">{t('modsPage.title')}</h1>
          </header>
          <div class="empty-state">
            <div class="empty-icon">📦</div>
            <h2>{t('modsPage.emptyTitle')}</h2>
            <p>{t('modsPage.emptyDescription')}</p>
          </div>
        </>
      }
    >
      {(id) => <Navigate href={`/game/${id}/mods`} />}
    </Show>
  );
};
