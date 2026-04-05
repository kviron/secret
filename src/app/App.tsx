import { Component, onCleanup, onMount } from 'solid-js';
import { AppRouter } from './router';
import { I18nProvider } from '@/shared/lib/i18n';
import { getStoredPreference, initThemeFromStorage, subscribeSystemThemeChange } from '@/shared/lib/theme';

export const App: Component = () => {
  onMount(() => {
    initThemeFromStorage();
    const unsub = subscribeSystemThemeChange(() => {
      if (getStoredPreference() === 'system') {
        initThemeFromStorage();
      }
    });
    onCleanup(unsub);
  });

  return (
    <I18nProvider>
      <AppRouter />
    </I18nProvider>
  );
};
