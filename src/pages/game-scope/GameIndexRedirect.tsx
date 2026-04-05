import { Component, Show } from 'solid-js';
import { Navigate, useParams } from '@solidjs/router';

/** `/game/:id` → `/game/:id/mods` */
export const GameIndexRedirect: Component = () => {
  const params = useParams<{ id: string }>();
  return (
    <Show when={params.id} keyed>
      {(id) => <Navigate href={`/game/${id}/mods`} />}
    </Show>
  );
};
