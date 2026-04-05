import {
  type Component,
  type ParentProps,
  createContext,
  createSignal,
  onMount,
  useContext,
} from 'solid-js';
import { en } from './locales/en';
import { ru } from './locales/ru';
import type { MessageKey } from './locales/en';
import { interpolate } from './interpolate';
import { applyLocale, getStoredLocale } from './storage';
import type { Locale } from './types';

const dictionaries: Record<Locale, Record<MessageKey, string>> = {
  en: en as Record<MessageKey, string>,
  ru,
};

export type TranslateFn = (
  key: MessageKey,
  params?: Record<string, string | number>,
) => string;

interface I18nValue {
  locale: () => Locale;
  setLocale: (locale: Locale) => void;
  t: TranslateFn;
}

const I18nContext = createContext<I18nValue>();

export const I18nProvider: Component<ParentProps> = (props) => {
  const [locale, setLocaleSignal] = createSignal<Locale>(getStoredLocale());

  onMount(() => {
    const l = getStoredLocale();
    setLocaleSignal(l);
    if (typeof document !== 'undefined') {
      document.title = interpolate(dictionaries[l]['app.documentTitle'], {});
    }
  });

  const setLocale = (next: Locale) => {
    setLocaleSignal(next);
    applyLocale(next);
    if (typeof document !== 'undefined') {
      document.title = interpolate(dictionaries[next]['app.documentTitle'], {});
    }
  };

  const t: TranslateFn = (key, params) => {
    const track = locale();
    const raw = dictionaries[track][key] ?? dictionaries.en[key] ?? key;
    return interpolate(raw, params);
  };

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>{props.children}</I18nContext.Provider>
  );
};

export function useI18n(): I18nValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error('useI18n must be used within I18nProvider');
  }
  return ctx;
}
