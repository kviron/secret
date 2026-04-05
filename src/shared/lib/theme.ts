export type ThemePreference = 'dark' | 'light' | 'system';

const STORAGE_KEY = 'pantheon-theme-preference';

export function getStoredPreference(): ThemePreference {
  if (typeof localStorage === 'undefined') {
    return 'dark';
  }
  const v = localStorage.getItem(STORAGE_KEY);
  if (v === 'light' || v === 'dark' || v === 'system') {
    return v;
  }
  return 'dark';
}

export function resolveTheme(pref: ThemePreference): 'light' | 'dark' {
  if (pref === 'system') {
    if (typeof window === 'undefined' || !window.matchMedia) {
      return 'dark';
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return pref;
}

/** Применяет `data-theme` на `<html>` и сохраняет предпочтение. */
export function applyThemePreference(pref: ThemePreference): void {
  if (typeof document === 'undefined') {
    return;
  }
  const resolved = resolveTheme(pref);
  document.documentElement.setAttribute('data-theme', resolved);
  document.documentElement.setAttribute('data-theme-preference', pref);
  try {
    localStorage.setItem(STORAGE_KEY, pref);
  } catch {
    /* ignore */
  }
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) {
    meta.setAttribute('content', resolved === 'dark' ? '#0a0a0b' : '#f4f4f6');
  }
}

/** Синхронизация из storage при старте приложения. */
export function initThemeFromStorage(): void {
  applyThemePreference(getStoredPreference());
}

/** Подписка на смену системной темы (для режима «Системная»). */
export function subscribeSystemThemeChange(onChange: () => void): () => void {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return () => {};
  }
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const handler = () => onChange();
  mq.addEventListener('change', handler);
  return () => mq.removeEventListener('change', handler);
}
