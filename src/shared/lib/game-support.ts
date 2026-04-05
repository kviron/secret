import type { Game } from '@/shared/types';
import type { MessageKey } from '@/shared/lib/i18n';

/**
 * Игры с каталога Gamebryo (.esp / .esm / .esl в `Data`) — аналог проверки `gameSupport.has(gameMode)`
 * в Vortex (`extensions/gamebryo-plugin-management`, вкладка Plugins только для этих id).
 * Для записей БД без `modPlugin` в `supportedModTypes` (старые установки) оставляем тот же набор id.
 */
const LEGACY_GAME_IDS_WITH_GAMEBRYO_PLUGINS: ReadonlySet<string> = new Set([
  'skyrim',
  'skyrimse',
  'skyrimvr',
  'fallout4',
  'fallout4vr',
  'falloutnv',
  'oblivion',
  'starfield',
]);

/** Показывать раздел «Плагины» (список .esp/.esm/.esl) только для поддерживаемых Gamebryo-тайтлов. */
export function gameSupportsGamebryoPlugins(game: Game): boolean {
  if (game.supportedModTypes.includes('modPlugin')) {
    return true;
  }
  return LEGACY_GAME_IDS_WITH_GAMEBRYO_PLUGINS.has(game.id);
}

/** Совпадает с `saves_dir_for_game_id` в `src-tauri/.../game_content.rs` (Documents/My Games/…/Saves). */
const LEGACY_GAME_IDS_WITH_KNOWN_SAVES_PATH: ReadonlySet<string> = new Set([
  'skyrim',
  'skyrimse',
  'skyrimvr',
  'fallout4',
  'fallout4vr',
  'falloutnv',
  'oblivion',
  'starfield',
]);

/** Показывать раздел «Сохранения», только если бэкенд знает папку сохранений для этого `game.id`. */
export function gameSupportsKnownSavesLocation(game: Game): boolean {
  if (game.supportedModTypes.includes('gameSaves')) {
    return true;
  }
  return LEGACY_GAME_IDS_WITH_KNOWN_SAVES_PATH.has(game.id);
}

/** Меньше = выше в списке: full → partial → остальное none → не в каталоге Pantheon. */
export function modSupportSortRank(game: Game): number {
  if (isGameUnsupportedByPantheon(game)) {
    return 3;
  }
  switch (game.modSupport) {
    case 'full':
      return 0;
    case 'partial':
      return 1;
    case 'none':
    default:
      return 2;
  }
}

/** Сортировка для библиотеки: сначала полная поддержка, затем частичная, затем остальные. */
export function sortGamesByModSupport(games: Game[]): Game[] {
  return [...games].sort((a, b) => {
    const d = modSupportSortRank(a) - modSupportSortRank(b);
    if (d !== 0) {
      return d;
    }
    const ma = a.installPathMissing ? 1 : 0;
    const mb = b.installPathMissing ? 1 : 0;
    if (ma !== mb) {
      return ma - mb;
    }
    return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
  });
}

/** Pantheon ещё не поддерживает моды / профиль для этой записи (нет типов и уровня none). */
export function isGameUnsupportedByPantheon(game: Game): boolean {
  return game.modSupport === 'none' && game.supportedModTypes.length === 0;
}

export type GameModStatusVariant = 'unsupported' | 'full' | 'partial' | 'none';

/** Ключ перевода и вариант стиля для бейджа в карточке (рядом с лаунчером). */
export function getGameModStatusBadge(game: Game): {
  labelKey: MessageKey;
  variant: GameModStatusVariant;
} {
  if (isGameUnsupportedByPantheon(game)) {
    return { labelKey: 'dashboard.modBadgeUnsupported', variant: 'unsupported' };
  }
  switch (game.modSupport) {
    case 'full':
      return { labelKey: 'dashboard.modBadgeFull', variant: 'full' };
    case 'partial':
      return { labelKey: 'dashboard.modBadgePartial', variant: 'partial' };
    case 'none':
    default:
      return { labelKey: 'dashboard.modBadgeNone', variant: 'none' };
  }
}
