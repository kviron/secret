import { createStore } from 'solid-js/store';
import type { Game, DetectionProgress, GameDetectionError, RemoveGameResult } from '@/shared/types';
import { sortGamesByModSupport } from '@/shared/lib/game-support';
import { gameApi } from '../api/gameApi';

interface GameStoreState {
  games: Game[];
  selectedGameId: string | null;
  isLoading: boolean;
  isDetecting: boolean;
  detectionProgress: DetectionProgress | null;
  detectionErrors: GameDetectionError[];
  error: string | null;
}

const [state, setState] = createStore<GameStoreState>({
  games: [],
  selectedGameId: null,
  isLoading: false,
  isDetecting: false,
  detectionProgress: null,
  detectionErrors: [],
  error: null,
});

let unlistenProgress: (() => void) | null = null;
let unlistenError: (() => void) | null = null;
let unlistenDetected: (() => void) | null = null;
let unlistenCompleted: (() => void) | null = null;

export const useGameStore = () => {
  const setupListeners = async () => {
    if (unlistenProgress) return;

    unlistenProgress = await gameApi.onDetectionProgress((progress) => {
      setState('detectionProgress', progress);
    });

    unlistenError = await gameApi.onDetectionError((error) => {
      setState('detectionErrors', (prev) => [...prev, error]);
    });

    unlistenDetected = await gameApi.onGameDetected((game) => {
      setState('games', (prev) => {
        const exists = prev.some((g) => g.id === game.id);
        const next = exists
          ? prev.map((g) => (g.id === game.id ? game : g))
          : [...prev, game];
        return sortGamesByModSupport(next);
      });
    });

    unlistenCompleted = await gameApi.onDetectionCompleted(() => {
      setState('isDetecting', false);
      setState('detectionProgress', null);
    });
  };

  const cleanupListeners = () => {
    unlistenProgress?.();
    unlistenError?.();
    unlistenDetected?.();
    unlistenCompleted?.();
    unlistenProgress = null;
    unlistenError = null;
    unlistenDetected = null;
    unlistenCompleted = null;
  };

  const loadGames = async () => {
    setState('isLoading', true);
    setState('error', null);
    try {
      await setupListeners();
      const games = await gameApi.getGames();
      setState('games', sortGamesByModSupport(games));
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isLoading', false);
    }
  };

  const detectGames = async () => {
    setState('isDetecting', true);
    setState('detectionErrors', []);
    setState('detectionProgress', null);
    setState('error', null);
    try {
      await setupListeners();
      const games = await gameApi.detectGames();
      setState('games', sortGamesByModSupport(games));
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isDetecting', false);
    }
  };

  const scanCustomPath = async (path: string) => {
    setState('isDetecting', true);
    setState('detectionErrors', []);
    setState('detectionProgress', null);
    setState('error', null);
    try {
      await setupListeners();
      const games = await gameApi.scanCustomPath(path);
      setState('games', sortGamesByModSupport(games));
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isDetecting', false);
    }
  };

  const selectGame = (gameId: string) => {
    setState('selectedGameId', gameId);
  };

  const clearDetectionErrors = () => {
    setState('detectionErrors', []);
  };

  const refreshGame = async (gameId: string) => {
    try {
      const g = await gameApi.getGame(gameId);
      if (!g) {
        setState('games', (prev) => prev.filter((x) => x.id !== gameId));
        return;
      }
      setState('games', (prev) => {
        const idx = prev.findIndex((x) => x.id === gameId);
        const next =
          idx === -1 ? [...prev, g] : prev.map((x) => (x.id === gameId ? g : x));
        return sortGamesByModSupport(next);
      });
    } catch (err) {
      setState('error', String(err));
    }
  };

  const removeGameFromLibrary = async (gameId: string): Promise<RemoveGameResult> => {
    const result = await gameApi.removeGameFromLibrary(gameId);
    setState('games', (prev) => prev.filter((g) => g.id !== gameId));
    return result;
  };

  return {
    state,
    loadGames,
    detectGames,
    scanCustomPath,
    selectGame,
    clearDetectionErrors,
    cleanupListeners,
    refreshGame,
    removeGameFromLibrary,
  };
};