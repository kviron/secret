import { Component, Show } from 'solid-js';
import { useGameStore } from '@/entities/game';

export const DetectionProgress: Component = () => {
  const { state } = useGameStore();

  const progressPercent = () => {
    const p = state.detectionProgress;
    if (!p || p.total === 0) return 0;
    return Math.round((p.found / p.total) * 100);
  };

  return (
    <Show when={state.isDetecting}>
      <div class="detection-progress">
        <div class="progress-header">
          <span class="progress-message">{state.detectionProgress?.message}</span>
          <span class="progress-count">
            {state.detectionProgress?.found ?? 0} / {state.detectionProgress?.total ?? 0}
          </span>
        </div>
        <div class="progress-bar-container">
          <div
            class="progress-bar-fill"
            style={{ width: `${progressPercent()}%` }}
          />
        </div>
        <Show when={state.detectionErrors.length > 0}>
          <div class="progress-errors">
            {state.detectionErrors.map((err) => (
              <div class={`error-item ${err.recoverable ? 'warning' : 'error'}`}>
                <span class="error-game">{err.gameName}</span>
                <span class="error-message">{err.error}</span>
              </div>
            ))}
          </div>
        </Show>
      </div>
    </Show>
  );
};