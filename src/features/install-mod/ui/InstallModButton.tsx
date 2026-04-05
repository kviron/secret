import { Component, createSignal } from 'solid-js';
import { Button } from '@/shared/ui/Button';
import { useModStore } from '@/entities/mod';
import { open } from '@tauri-apps/plugin-dialog';
import { useI18n } from '@/shared/lib/i18n';

interface InstallModButtonProps {
  gameId: string;
  onInstalled?: () => void;
  disabled?: boolean;
}

export const InstallModButton: Component<InstallModButtonProps> = (props) => {
  const { t } = useI18n();
  const [isInstalling, setIsInstalling] = createSignal(false);
  const { addMod } = useModStore();

  const handleClick = async () => {
    if (props.disabled) {
      return;
    }
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
    <Button
      onClick={handleClick}
      isLoading={isInstalling()}
      variant="primary"
      size="md"
      disabled={props.disabled}
      title={props.disabled ? t('installMod.titleDisabled') : undefined}
    >
      {t('installMod.button')}
    </Button>
  );
};
