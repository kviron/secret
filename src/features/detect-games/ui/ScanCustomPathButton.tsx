import { Component } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';
import { Button } from '@/shared/ui/Button';

export const ScanCustomPathButton: Component = () => {
  const { t } = useI18n();
  const { scanCustomPath, state } = useGameStore();

  const handleClick = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('detect.dialogSelectGameFolder'),
    });

    if (selected && typeof selected === 'string') {
      await scanCustomPath(selected);
    }
  };

  return (
    <Button
      onClick={handleClick}
      isLoading={state.isDetecting}
      disabled={state.isDetecting}
      variant="secondary"
    >
      {t('detect.addFromFolder')}
    </Button>
  );
};