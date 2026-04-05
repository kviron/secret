import { openPath, openUrl } from '@tauri-apps/plugin-opener';
import type { Game } from '@/shared/types';

function joinInstallExe(installPath: string, relativeExe: string): string {
  const base = installPath.replace(/[/\\]+$/, '');
  const rel = relativeExe.replace(/^[/\\]+/, '');
  const winLike = /^[a-zA-Z]:/.test(base) || base.startsWith('\\\\');
  const sep = winLike ? '\\' : '/';
  const normalized = rel.split(/[/\\]/).filter(Boolean).join(sep);
  return `${base}${sep}${normalized}`;
}

/**
 * Запуск игры: Steam через `steam://run/`, иначе первый exe из `requiredFiles`.
 */
export async function launchGame(game: Game): Promise<void> {
  if (game.installPathMissing === true) {
    throw new Error('Папка установки не найдена');
  }

  if (game.launcher === 'steam' && game.details.steamAppId != null) {
    await openUrl(`steam://run/${game.details.steamAppId}`);
    return;
  }

  const files = game.details.requiredFiles;
  if (!files?.length) {
    throw new Error('Не указан исполняемый файл');
  }

  await openPath(joinInstallExe(game.installPath, files[0]));
}
