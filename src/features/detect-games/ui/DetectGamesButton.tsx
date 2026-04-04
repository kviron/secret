import { Component } from 'solid-js';
import { Button } from '@/shared/ui/Button';
import { useGameStore } from '@/entities/game';

interface DetectGamesButtonProps {
  onDetected?: () => void;
}

export const DetectGamesButton: Component<DetectGamesButtonProps> = (props) => {
  const { detectGames, state } = useGameStore();

  const handleClick = async () => {
    await detectGames();
    props.onDetected?.();
  };

  return (
    <Button
      onClick={handleClick}
      isLoading={state.isDetecting}
      disabled={state.isDetecting}
    >
      {state.isDetecting ? 'Scanning...' : 'Detect Games'}
    </Button>
  );
};