import { Component } from 'solid-js';
import type { Mod } from '@/shared/types';
import { modApi } from '@/entities/mod';
import { Switch } from '@/shared/ui/Switch';

interface ToggleModProps {
  mod: Mod;
  onToggle?: () => void;
}

export const ToggleMod: Component<ToggleModProps> = (props) => {
  const handleChange = async (checked: boolean) => {
    try {
      await modApi.setModEnabled(props.mod.id, checked);
      props.onToggle?.();
    } catch (err) {
      console.error('Failed to toggle mod:', err);
    }
  };

  return (
    <Switch
      checked={props.mod.enabled}
      onChange={handleChange}
      label={props.mod.enabled ? 'Enabled' : 'Disabled'}
    />
  );
};
