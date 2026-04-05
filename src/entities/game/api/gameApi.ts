import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { api } from '@/shared/api/client';
import type {
  Game,
  DetectionProgress,
  GameDetectionError,
  RemoveGameResult,
  SaveFileEntry,
} from '@/shared/types';

export const gameApi = {
  getGames: () => api.invoke<Game[]>('get_games'),
  getGame: (gameId: string) => api.invoke<Game | null>('get_game', { gameId }),
  detectGames: () => api.invoke<Game[]>('detect_games'),
  scanCustomPath: (path: string) => api.invoke<Game[]>('scan_custom_path', { path }),
  registerGame: (game: Game) => api.invoke<Game>('register_game', { game }),
  unregisterGame: (gameId: string) => api.invoke<void>('unregister_game', { gameId }),
  removeGameFromLibrary: (gameId: string) =>
    api.invoke<RemoveGameResult>('remove_game_from_library', { gameId }),

  listGamePlugins: (gameId: string) => api.invoke<string[]>('list_game_plugins', { gameId }),

  listGameSaves: (gameId: string) => api.invoke<SaveFileEntry[]>('list_game_saves', { gameId }),

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