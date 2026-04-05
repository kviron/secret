import type { Game } from '@/shared/types';

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

/** Текст и вариант стиля для бейджа в карточке (рядом с лаунчером). */
export function getGameModStatusBadge(game: Game): {
  label: string;
  variant: GameModStatusVariant;
} {
  if (isGameUnsupportedByPantheon(game)) {
    return { label: 'Не поддерживается', variant: 'unsupported' };
  }
  switch (game.modSupport) {
    case 'full':
      return { label: 'Полная поддержка модов', variant: 'full' };
    case 'partial':
      return { label: 'Ограниченная поддержка модов', variant: 'partial' };
    case 'none':
    default:
      return { label: 'Без поддержки модов', variant: 'none' };
  }
}
