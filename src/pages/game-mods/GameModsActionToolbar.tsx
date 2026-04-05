import type { Component } from 'solid-js';
import { Portal } from 'solid-js/web';
import { Menu } from '@ark-ui/solid/menu';
import { Tooltip } from '@ark-ui/solid/tooltip';
import {
  ChevronDown,
  Clapperboard,
  Download,
  FolderOpen,
  GitBranch,
  History,
  Link,
  RefreshCw,
  SquarePlus,
  Tag,
  Undo2,
  Unlink,
} from 'lucide-solid';
import type { LucideIcon } from 'lucide-solid';
import { useI18n } from '@/shared/lib/i18n';
import type { MessageKey } from '@/shared/lib/i18n';

const iconProps = (): Parameters<LucideIcon>[0] => ({
  size: 22,
  strokeWidth: 2,
  class: 'mods-action-toolbar__icon',
});

const noop = () => {
  /* toolbar actions — wired later */
};

const tooltipPositioning = { strategy: 'fixed' as const, placement: 'top' as const };

const ToolbarMenu: Component<{
  tooltipKey: MessageKey;
  icon: LucideIcon;
}> = (props) => {
  const { t } = useI18n();
  const label = () => t(props.tooltipKey);
  const Icon = props.icon;
  return (
    <Menu.Root>
      <Tooltip.Root positioning={tooltipPositioning}>
        <Menu.Trigger
          asChild={(mergeMenuProps) => (
            <Tooltip.Trigger
              {...mergeMenuProps({})}
              type="button"
              class="mods-action-toolbar__btn mods-action-toolbar__btn--menu"
              aria-label={label()}
            >
              <Icon {...iconProps()} />
              <ChevronDown size={17} strokeWidth={2} class="mods-action-toolbar__chevron" aria-hidden />
            </Tooltip.Trigger>
          )}
        />
        <Portal>
          <Tooltip.Positioner>
            <Tooltip.Content class="mods-action-toolbar__tooltip-content">{label()}</Tooltip.Content>
          </Tooltip.Positioner>
        </Portal>
      </Tooltip.Root>
      <Menu.Positioner>
        <Menu.Content class="mods-action-toolbar__menu-content">
          <Menu.Item class="mods-action-toolbar__menu-item" value="placeholder" onSelect={noop}>
            {t('gameMods.toolbar.menuPlaceholder')}
          </Menu.Item>
        </Menu.Content>
      </Menu.Positioner>
    </Menu.Root>
  );
};

export const GameModsActionToolbar: Component = () => {
  const { t } = useI18n();

  const Btn: Component<{ tooltipKey: MessageKey; icon: LucideIcon }> = (p) => {
    const Icon = p.icon;
    const label = () => t(p.tooltipKey);
    return (
      <Tooltip.Root positioning={tooltipPositioning}>
        <Tooltip.Trigger
          type="button"
          class="mods-action-toolbar__btn"
          aria-label={label()}
          onClick={noop}
        >
          <Icon {...iconProps()} />
        </Tooltip.Trigger>
        <Portal>
          <Tooltip.Positioner>
            <Tooltip.Content class="mods-action-toolbar__tooltip-content">{label()}</Tooltip.Content>
          </Tooltip.Positioner>
        </Portal>
      </Tooltip.Root>
    );
  };

  return (
    <div class="mods-action-toolbar" role="toolbar" aria-label={t('gameMods.toolbar.aria')}>
      <div class="mods-action-toolbar__group">
        <Btn tooltipKey="gameMods.toolbar.installFromFile" icon={SquarePlus} />
        <Btn tooltipKey="gameMods.toolbar.checkUpdates" icon={RefreshCw} />
        <Btn tooltipKey="gameMods.toolbar.categories" icon={Tag} />
        <Btn tooltipKey="gameMods.toolbar.manageRules" icon={GitBranch} />
        <Btn tooltipKey="gameMods.toolbar.deployMods" icon={Link} />
        <Btn tooltipKey="gameMods.toolbar.purgeMods" icon={Unlink} />
        <Btn tooltipKey="gameMods.toolbar.resetManifest" icon={Undo2} />
        <ToolbarMenu tooltipKey="gameMods.toolbar.importFrom" icon={Download} />
        <Btn tooltipKey="gameMods.toolbar.history" icon={History} />
        <ToolbarMenu tooltipKey="gameMods.toolbar.open" icon={FolderOpen} />
        <ToolbarMenu tooltipKey="gameMods.toolbar.tutorials" icon={Clapperboard} />
      </div>
    </div>
  );
};
