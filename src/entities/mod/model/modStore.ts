import { createStore } from 'solid-js/store';
import type { Mod } from '@/shared/types';
import { modApi } from '../api/modApi';

interface ModStoreState {
  mods: Mod[];
  isLoading: boolean;
  error: string | null;
}

const [state, setState] = createStore<ModStoreState>({
  mods: [],
  isLoading: false,
  error: null,
});

export const useModStore = () => {
  const loadMods = async (gameId: string) => {
    setState('isLoading', true);
    setState('error', null);
    try {
      const mods = await modApi.getMods(gameId);
      setState('mods', mods);
    } catch (err) {
      setState('error', String(err));
    } finally {
      setState('isLoading', false);
    }
  };

  const addMod = (mod: Mod) => {
    setState('mods', (mods) => [...mods, mod]);
  };

  const removeMod = (modId: string) => {
    setState('mods', (mods) => mods.filter((m) => m.id !== modId));
  };

  const updateMod = (modId: string, updates: Partial<Mod>) => {
    setState('mods', (mod) => mod.id === modId, updates);
  };

  return { state, loadMods, addMod, removeMod, updateMod };
};