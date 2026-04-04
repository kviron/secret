import { invoke } from '@tauri-apps/api/core';

export const api = {
  invoke: <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    return invoke<T>(command, args);
  },
};
