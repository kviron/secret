import { Component } from 'solid-js';
import { Switch as ArkSwitch } from '@ark-ui/solid/switch';

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
  class?: string;
}

export const Switch: Component<SwitchProps> = (props) => {
  return (
    <ArkSwitch.Root
      checked={props.checked}
      onCheckedChange={(e) => props.onChange(e.checked)}
      disabled={props.disabled}
      class={props.class}
    >
      <ArkSwitch.Control>
        <ArkSwitch.Thumb />
      </ArkSwitch.Control>
      {props.label && <ArkSwitch.Label>{props.label}</ArkSwitch.Label>}
      <ArkSwitch.HiddenInput />
    </ArkSwitch.Root>
  );
};
