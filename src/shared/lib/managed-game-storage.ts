const MANAGED_GAME_STORAGE_KEY = 'pantheon.managedGameId';

export function readManagedGameIdFromStorage(): string | null {
  if (typeof window === 'undefined' || !window.localStorage) {
    return null;
  }
  try {
    const v = window.localStorage.getItem(MANAGED_GAME_STORAGE_KEY);
    return v && v.length > 0 ? v : null;
  } catch {
    return null;
  }
}

export function writeManagedGameIdToStorage(gameId: string | null): void {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }
  try {
    if (gameId == null || gameId === '') {
      window.localStorage.removeItem(MANAGED_GAME_STORAGE_KEY);
    } else {
      window.localStorage.setItem(MANAGED_GAME_STORAGE_KEY, gameId);
    }
  } catch {
    /* ignore quota / private mode */
  }
}
