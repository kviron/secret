import { Component, JSX } from 'solid-js';
import { Dialog as ArkDialog } from '@ark-ui/solid/dialog';

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  children: JSX.Element;
  actions?: JSX.Element;
}

export const Dialog: Component<DialogProps> = (props) => {
  return (
    <ArkDialog.Root
      open={props.open}
      onOpenChange={(e) => props.onOpenChange(e.open)}
    >
      <ArkDialog.Backdrop />
      <ArkDialog.Positioner>
        <ArkDialog.Content>
          <ArkDialog.Title>{props.title}</ArkDialog.Title>
          {props.description && <ArkDialog.Description>{props.description}</ArkDialog.Description>}
          <div class="dialog-body">{props.children}</div>
          {props.actions && <div class="dialog-actions">{props.actions}</div>}
          <ArkDialog.CloseTrigger class="dialog-close-btn">Close</ArkDialog.CloseTrigger>
        </ArkDialog.Content>
      </ArkDialog.Positioner>
    </ArkDialog.Root>
  );
};
