import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { api } from '@/shared/api/client';
import type {
  Game,
  DetectionProgress,
  GameDetectionError,
  GameInstallStats,
  RemoveGameResult,
  SaveFileEntry,
  SaveBackupEntry,
} from '@/shared/types';

export const gameApi = {
  getGames: () => api.invoke<Game[]>('get_games'),
  getGame: (gameId: string) => api.invoke<Game | null>('get_game', { gameId }),
  getGameInstallStats: (gameId: string) =>
    api.invoke<GameInstallStats>('get_game_install_stats', { gameId }),
  detectGames: () => api.invoke<Game[]>('detect_games'),
  scanCustomPath: (path: string) => api.invoke<Game[]>('scan_custom_path', { path }),
  registerGame: (game: Game) => api.invoke<Game>('register_game', { game }),
  unregisterGame: (gameId: string) => api.invoke<void>('unregister_game', { gameId }),
  removeGameFromLibrary: (gameId: string) =>
    api.invoke<RemoveGameResult>('remove_game_from_library', { gameId }),

  listGamePlugins: (gameId: string) => api.invoke<string[]>('list_game_plugins', { gameId }),

  listGameSaves: (gameId: string) => api.invoke<SaveFileEntry[]>('list_game_saves', { gameId }),
  deleteSave: (gameId: string, savePath: string) =>
    api.invoke<void>('delete_save', { gameId, savePath }),
  backupSave: (gameId: string, savePath: string) =>
    api.invoke<string>('backup_save', { gameId, savePath }),
  restoreSave: (gameId: string, backupPath: string) =>
    api.invoke<void>('restore_save', { gameId, backupPath }),
  listSaveBackups: (gameId: string) =>
    api.invoke<SaveBackupEntry[]>('list_save_backups', { gameId }),
  getSavesDirPath: (gameId: string) =>
    api.invoke<string | null>('get_saves_dir_path', { gameId }),

  onDetectionStarted: (cb: () => void): Promise<UnlistenFn> =>
    listen('game_detection_started', () => cb()),

  onDetectionProgress: (cb: (progress: DetectionProgress) => void): Promise<UnlistenFn> =>
    listen<DetectionProgress>('game_detection_progress', (e) => cb(e.payload)),

  onGameDetected: (cb: (game: Game) => void): Promise<UnlistenFn> =>
    listen<Game>('game_detected', (e) => cb(e.payload)),

  onDetectionError: (cb: (error: GameDetectionError) => void): Promise<UnlistenFn> =>
    listen<GameDetectionError>('game_detection_error', (e) => cb(e.payload)),

  onDetectionCompleted: (cb: (result: { count: number }) => void): Promise<UnlistenFn> =>
    listen<{ count: number }>('game_detection_completed', (e) => cb(e.payload)),
};
