import { Component } from 'solid-js';
import { Button } from '@/shared/ui/Button';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';

interface DetectGamesButtonProps {
  onDetected?: () => void;
}

export const DetectGamesButton: Component<DetectGamesButtonProps> = (props) => {
  const { t } = useI18n();
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
      {state.isDetecting ? t('detect.scanning') : t('detect.detectGames')}
    </Button>
  );
};