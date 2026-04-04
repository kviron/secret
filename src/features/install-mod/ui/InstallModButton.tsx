import { Component, createSignal } from 'solid-js';
import { Button } from '@/shared/ui/Button';
import { useModStore } from '@/entities/mod';
import { open } from '@tauri-apps/plugin-dialog';

interface InstallModButtonProps {
  gameId: string;
  onInstalled?: () => void;
}

export const InstallModButton: Component<InstallModButtonProps> = (props) => {
  const [isInstalling, setIsInstalling] = createSignal(false);
  const { addMod } = useModStore();

  const handleClick = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Archives', extensions: ['zip', '7z', 'rar'] }],
    });

    if (typeof selected === 'string') {
      setIsInstalling(true);
      try {
        const { modApi } = await import('@/entities/mod');
        const mod = await modApi.installMod(props.gameId, selected);
        addMod(mod);
        props.onInstalled?.();
      } catch (err) {
        console.error('Failed to install mod:', err);
      } finally {
        setIsInstalling(false);
      }
    }
  };

  return (
    <Button onClick={handleClick} isLoading={isInstalling()} variant="primary" size="md">
      Install Mod
    </Button>
  );
};
