import type { Locale } from './types';

const STORAGE_KEY = 'pantheon-locale';

export function resolveNavigatorLocale(): Locale {
  if (typeof navigator === 'undefined') {
    return 'en';
  }
  return navigator.language.toLowerCase().startsWith('ru') ? 'ru' : 'en';
}

/** Читает сохранённую локаль или эвристику по языку ОС (без записи в storage). */
export function getStoredLocale(): Locale {
  if (typeof localStorage === 'undefined') {
    return resolveNavigatorLocale();
  }
  const v = localStorage.getItem(STORAGE_KEY);
  if (v === 'ru' || v === 'en') {
    return v;
  }
  return resolveNavigatorLocale();
}

export function applyLocale(locale: Locale): void {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale === 'ru' ? 'ru' : 'en';
  }
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, locale);
    } catch {
      /* ignore */
    }
  }
}

/**
 * При первом запуске записывает локаль в storage (как для темы).
 * Вызывать до первого рендера или в onMount приложения.
 */
export function initLocaleFromStorage(): void {
  if (typeof localStorage === 'undefined') {
    applyLocale(resolveNavigatorLocale());
    return;
  }
  const v = localStorage.getItem(STORAGE_KEY);
  if (v === 'ru' || v === 'en') {
    applyLocale(v);
    return;
  }
  const initial = resolveNavigatorLocale();
  try {
    localStorage.setItem(STORAGE_KEY, initial);
  } catch {
    /* ignore */
  }
  applyLocale(initial);
}
