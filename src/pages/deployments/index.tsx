import { Component } from 'solid-js';
import { useI18n } from '@/shared/lib/i18n';

export const DeploymentsPage: Component = () => {
  const { t } = useI18n();

  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">{t('deploymentsPage.title')}</h1>
      </header>
      <div class="empty-state">
        <div class="empty-icon">🚀</div>
        <h2>{t('deploymentsPage.emptyTitle')}</h2>
        <p>{t('deploymentsPage.emptyDescription')}</p>
      </div>
    </>
  );
};