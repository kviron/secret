import { Select, createListCollection } from '@ark-ui/solid/select';
import { Component, For, Show, createMemo, createSignal, onMount } from 'solid-js';
import { useLocation, useNavigate } from '@solidjs/router';
import type { MessageKey } from '@/shared/lib/i18n';
import { useI18n } from '@/shared/lib/i18n';
import type { ThemePreference } from '@/shared/lib/theme';
import {
  applyThemePreference,
  getStoredPreference,
} from '@/shared/lib/theme';

const TAB_IDS = [
  'appearance',
  'games',
  'mods',
  'deploy',
  'downloads',
  'loadorder',
  'extensions',
] as const;

type TabId = (typeof TAB_IDS)[number];

function isTabId(s: string): s is TabId {
  return (TAB_IDS as readonly string[]).includes(s);
}

const TAB_LABEL_KEYS: Record<TabId, MessageKey> = {
  appearance: 'settings.tab.appearance',
  games: 'settings.tab.games',
  mods: 'settings.tab.mods',
  deploy: 'settings.tab.deploy',
  downloads: 'settings.tab.downloads',
  loadorder: 'settings.tab.loadorder',
  extensions: 'settings.tab.extensions',
};

const TAB_DESC_KEYS: Record<TabId, MessageKey> = {
  appearance: 'settings.tabDesc.appearance',
  games: 'settings.tabDesc.games',
  mods: 'settings.tabDesc.mods',
  deploy: 'settings.tabDesc.deploy',
  downloads: 'settings.tabDesc.downloads',
  loadorder: 'settings.tabDesc.loadorder',
  extensions: 'settings.tabDesc.extensions',
};

const EMPTY_TAB_IDS: TabId[] = ['games', 'mods', 'deploy', 'downloads', 'loadorder', 'extensions'];

/** Региональный код для отображения в селекте (как на макете: GB / RU). */
function localeRegionCode(value: string): string {
  return value === 'en' ? 'GB' : 'RU';
}

export const SettingsPage: Component = () => {
  const { t, locale, setLocale } = useI18n();
  const location = useLocation();
  const navigate = useNavigate();

  const localeCollection = createMemo(() =>
    createListCollection({
      items: [
        {
          value: 'en',
          label: `${localeRegionCode('en')} ${t('settings.language.en')}`,
        },
        {
          value: 'ru',
          label: `${localeRegionCode('ru')} ${t('settings.language.ru')}`,
        },
      ],
    }),
  );

  const activeTab = createMemo((): TabId => {
    const q = new URLSearchParams(location.search).get('tab');
    if (q && isTabId(q)) {
      return q;
    }
    return 'appearance';
  });

  const setTab = (id: TabId) => {
    navigate(`/settings?tab=${id}`, { replace: true });
  };

  const [themePref, setThemePref] = createSignal<ThemePreference>(getStoredPreference());

  onMount(() => {
    setThemePref(getStoredPreference());
  });

  const handleThemeChange = (pref: ThemePreference) => {
    setThemePref(pref);
    applyThemePreference(pref);
  };

  const handleLocaleChange = (details: { value: string[] }) => {
    const v = details.value[0];
    if (v === 'en' || v === 'ru') {
      setLocale(v);
    }
  };

  const activeTabLabel = () => t(TAB_LABEL_KEYS[activeTab()]);

  return (
    <div class="settings-page">
      <header class="settings-page-header">
        <h1 class="page-title settings-page-title">{t('settings.title')}</h1>
      </header>

      <div class="settings-tabs-wrap">
        <nav class="settings-tabs" aria-label={t('settings.themeNavAria')}>
          <For each={TAB_IDS}>
            {(id) => (
              <button
                type="button"
                class="settings-tab"
                classList={{ 'settings-tab--active': activeTab() === id }}
                onClick={() => setTab(id)}
              >
                {t(TAB_LABEL_KEYS[id])}
              </button>
            )}
          </For>
        </nav>
      </div>

      <div class="settings-panel">
        <p class="settings-panel-desc">{t(TAB_DESC_KEYS[activeTab()])}</p>

        <Show when={activeTab() === 'appearance'}>
          <section class="settings-section" aria-label={t('settings.tab.appearance')}>
            <div class="settings-theme-grid">
              <button
                type="button"
                class="settings-theme-card"
                classList={{ 'settings-theme-card--selected': themePref() === 'light' }}
                onClick={() => handleThemeChange('light')}
                aria-pressed={themePref() === 'light'}
              >
                <div class="settings-theme-preview settings-theme-preview--light">
                  <span class="settings-theme-preview__bar1" />
                  <span class="settings-theme-preview__bar2" />
                  <span class="settings-theme-preview__check" aria-hidden="true">
                    <Show when={themePref() === 'light'}>
                      <svg viewBox="0 0 24 24" width="20" height="20" fill="none">
                        <circle cx="12" cy="12" r="11" fill="var(--settings-accent-selection)" />
                        <path
                          d="M8 12.5l2.5 2.5 5.5-6"
                          stroke="white"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        />
                      </svg>
                    </Show>
                  </span>
                </div>
                <span class="settings-theme-card__title">{t('settings.theme.light.title')}</span>
                <span class="settings-theme-card__desc">{t('settings.theme.light.desc')}</span>
              </button>

              <button
                type="button"
                class="settings-theme-card"
                classList={{ 'settings-theme-card--selected': themePref() === 'dark' }}
                onClick={() => handleThemeChange('dark')}
                aria-pressed={themePref() === 'dark'}
              >
                <div class="settings-theme-preview settings-theme-preview--dark">
                  <span class="settings-theme-preview__bar1" />
                  <span class="settings-theme-preview__bar2" />
                  <span class="settings-theme-preview__check" aria-hidden="true">
                    <Show when={themePref() === 'dark'}>
                      <svg viewBox="0 0 24 24" width="20" height="20" fill="none">
                        <circle cx="12" cy="12" r="11" fill="var(--settings-accent-selection)" />
                        <path
                          d="M8 12.5l2.5 2.5 5.5-6"
                          stroke="white"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        />
                      </svg>
                    </Show>
                  </span>
                </div>
                <span class="settings-theme-card__title">{t('settings.theme.dark.title')}</span>
                <span class="settings-theme-card__desc">{t('settings.theme.dark.desc')}</span>
              </button>

              <button
                type="button"
                class="settings-theme-card"
                classList={{ 'settings-theme-card--selected': themePref() === 'system' }}
                onClick={() => handleThemeChange('system')}
                aria-pressed={themePref() === 'system'}
              >
                <div class="settings-theme-preview settings-theme-preview--system">
                  <span class="settings-theme-preview__half settings-theme-preview__half--light" />
                  <span class="settings-theme-preview__half settings-theme-preview__half--dark" />
                  <span class="settings-theme-preview__bar1" />
                  <span class="settings-theme-preview__bar2" />
                  <span class="settings-theme-preview__check" aria-hidden="true">
                    <Show when={themePref() === 'system'}>
                      <svg viewBox="0 0 24 24" width="20" height="20" fill="none">
                        <circle cx="12" cy="12" r="11" fill="var(--settings-accent-selection)" />
                        <path
                          d="M8 12.5l2.5 2.5 5.5-6"
                          stroke="white"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        />
                      </svg>
                    </Show>
                  </span>
                </div>
                <span class="settings-theme-card__title">{t('settings.theme.system.title')}</span>
                <span class="settings-theme-card__desc">{t('settings.theme.system.desc')}</span>
              </button>
            </div>
          </section>

          <section class="settings-section settings-section--language" aria-label={t('settings.language.title')}>
            <div class="settings-lang-row">
              <div class="settings-lang-row__text">
                <h2 class="settings-lang-row__title">{t('settings.language.title')}</h2>
                <p class="settings-lang-row__desc">{t('settings.language.description')}</p>
              </div>
              <div class="settings-lang-row__control">
                <Select.Root
                  collection={localeCollection()}
                  value={[locale() as string]}
                  onValueChange={handleLocaleChange}
                  positioning={{ sameWidth: true, gutter: 8, placement: 'bottom-start' }}
                >
                  <Select.Control class="settings-lang-select-control">
                    <Select.Trigger class="settings-lang-select-trigger" type="button">
                      <Select.ValueText class="settings-lang-select__value" />
                      <Select.Indicator class="settings-lang-select__chevron">
                        <svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true">
                          <path
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 9l6 6 6-6"
                          />
                        </svg>
                      </Select.Indicator>
                    </Select.Trigger>
                    <Select.Positioner class="settings-lang-select-positioner">
                      <Select.Content class="settings-lang-select-content">
                        <Select.List class="settings-lang-select-list">
                          <For each={localeCollection().items}>
                            {(item) => (
                              <Select.Item item={item} class="settings-lang-select__item">
                                <Select.ItemText class="settings-lang-select__item-label">
                                  {item.label}
                                </Select.ItemText>
                                <Select.ItemIndicator class="settings-lang-select__check">
                                  <svg
                                    class="settings-lang-select__check-icon"
                                    viewBox="0 0 24 24"
                                    width="18"
                                    height="18"
                                    aria-hidden="true"
                                  >
                                    <path
                                      fill="currentColor"
                                      d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"
                                    />
                                  </svg>
                                </Select.ItemIndicator>
                              </Select.Item>
                            )}
                          </For>
                        </Select.List>
                      </Select.Content>
                    </Select.Positioner>
                  </Select.Control>
                  <Select.HiddenSelect />
                </Select.Root>
              </div>
            </div>
          </section>
        </Show>

        <Show when={EMPTY_TAB_IDS.includes(activeTab())}>
          <div class="settings-placeholder">
            <p class="settings-placeholder__title">{t('settings.placeholder.title')}</p>
            <p class="settings-placeholder__text">
              {t('settings.placeholder.text', { label: activeTabLabel() })}
            </p>
          </div>
        </Show>
      </div>
    </div>
  );
};
