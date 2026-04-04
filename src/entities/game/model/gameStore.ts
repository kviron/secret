import { createStore } from 'solid-js/store';
import type { Game, DetectionProgress, GameDetectionError } from '@/shared/types';
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
        if (exists) {
          return prev.map((g) => (g.id === game.id ? game : g));
        }
        return [...prev, game];
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
      setState('games', games);
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
      setState('games', games);
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
      setState('games', games);
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

  return {
    state,
    loadGames,
    detectGames,
    scanCustomPath,
    selectGame,
    clearDetectionErrors,
    cleanupListeners,
  };
};