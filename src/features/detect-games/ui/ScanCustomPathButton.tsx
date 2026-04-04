import { Component } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { useGameStore } from '@/entities/game';
import { Button } from '@/shared/ui/Button';

export const ScanCustomPathButton: Component = () => {
  const { scanCustomPath, state } = useGameStore();

  const handleClick = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select game installation folder',
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
      Add from Folder...
    </Button>
  );
};