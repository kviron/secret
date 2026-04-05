import { Component, JSX, createEffect } from 'solid-js';
import { useParams } from '@solidjs/router';
import { useGameStore } from '@/entities/game';

/** Syncs URL `/game/:id/*` with `managedGameId` in the global store. */
export const GameScopeLayout: Component<{ children?: JSX.Element }> = (props) => {
  const params = useParams<{ id: string }>();
  const { setManagedGame } = useGameStore();

  createEffect(() => {
    const id = params.id;
    if (id) {
      setManagedGame(id);
    }
  });

  return <>{props.children}</>;
};
