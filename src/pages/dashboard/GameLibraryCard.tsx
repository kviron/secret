import {
  Component,
  Show,
  createMemo,
  createResource,
  createSignal,
} from 'solid-js';
import type { Resource } from 'solid-js';
import { HoverCard } from '@ark-ui/solid/hover-card';
import { gameApi } from '@/entities/game';
import { getGameModStatusBadge, isGameUnsupportedByPantheon } from '@/shared/lib/game-support';
import { formatBytes } from '@/shared/lib/format-bytes';
import type { Locale } from '@/shared/lib/i18n';
import type { TranslateFn } from '@/shared/lib/i18n';
import { launchGame } from '@/shared/lib/launch-game';
import { steamHeaderImageUrl } from '@/shared/lib/steam-art';
import type { Game, GameInstallStats } from '@/shared/types';
import { Button } from '@/shared/ui/Button';
import { Card } from '@/shared/ui/Card';

function formatDateTime(iso: string, locale: Locale): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  const tag = locale === 'ru' ? 'ru-RU' : 'en-US';
  return d.toLocaleString(tag, { dateStyle: 'medium', timeStyle: 'short' });
}

const DetailRow: Component<{ label: string; value: string; valueClass?: string }> = (props) => (
  <div class="game-card-details-row">
    <dt>{props.label}</dt>
    <dd class={props.valueClass}>{props.value}</dd>
  </div>
);

const GameCardCover: Component<{ game: Game }> = (props) => {
  const artUrl = createMemo(() => {
    const g = props.game;
    if (g.details.logo) return g.details.logo;
    if (g.launcher === 'steam' && g.details.steamAppId != null) {
      return steamHeaderImageUrl(g.details.steamAppId);
    }
    return undefined;
  });

  const [artBroken, setArtBroken] = createSignal(false);

  const showArt = () => Boolean(artUrl()) && !artBroken();

  return (
    <div class="game-card-header">
      <Show when={showArt()}>
        <img
          class="game-card-art"
          src={artUrl()!}
          alt=""
          loading="lazy"
          decoding="async"
          onError={() => setArtBroken(true)}
        />
      </Show>
      <Show when={!showArt()}>
        <div class="game-icon-placeholder">{props.game.name.charAt(0)}</div>
      </Show>
    </div>
  );
};

const GameCardDetails: Component<{
  game: Game;
  t: TranslateFn;
  locale: Locale;
  installStats: Resource<GameInstallStats | undefined>;
}> = (props) => {
  const modBadge = () => getGameModStatusBadge(props.game);
  const mergeLabel = () =>
    props.game.mergeMods ? props.t('dashboard.detailYes') : props.t('dashboard.detailNo');

  const versionLine = () => {
    const r = props.installStats;
    if (r.loading) return props.t('dashboard.detailsStatsLoading');
    if (r.state === 'errored') return props.t('dashboard.detailsStatsError');
    return r()?.installedVersionLabel ?? '—';
  };

  const diskLine = () => {
    const r = props.installStats;
    if (r.loading) return props.t('dashboard.detailsStatsLoading');
    if (r.state === 'errored') return props.t('dashboard.detailsStatsError');
    const data = r();
    if (!data) return '—';
    return formatBytes(data.diskUsageBytes);
  };

  const diskLineNoSymlinks = () => {
    const r = props.installStats;
    if (r.loading) return props.t('dashboard.detailsStatsLoading');
    if (r.state === 'errored') return props.t('dashboard.detailsStatsError');
    const data = r();
    if (!data) return '—';
    return formatBytes(data.diskUsageBytesNoSymlinks);
  };

  const steamManifestLine = () => {
    const r = props.installStats;
    if (r.loading || r.state === 'errored') return null;
    const n = r()?.steamSizeOnDiskBytes;
    return n != null ? formatBytes(n) : null;
  };

  return (
    <div class="game-card-details-popover-inner">
      <div class="game-card-details-popover__title">{props.t('dashboard.detailsTitle')}</div>
      <dl class="game-card-details-dl">
        <DetailRow
          label={props.t('dashboard.detailPath')}
          value={props.game.installPath}
          valueClass="game-card-details-path"
        />
        <DetailRow label={props.t('dashboard.detailStore')} value={props.game.launcher} />
        <DetailRow label={props.t('dashboard.detailInstalledVersion')} value={versionLine()} />
        <DetailRow label={props.t('dashboard.detailDiskUsage')} value={diskLine()} />
        <DetailRow
          label={props.t('dashboard.detailDiskUsageNoSymlinks')}
          value={diskLineNoSymlinks()}
        />
        <Show when={steamManifestLine()}>
          <DetailRow
            label={props.t('dashboard.detailSteamManifestSize')}
            value={steamManifestLine()!}
          />
        </Show>
        <DetailRow label={props.t('dashboard.detailSupportPath')} value={props.game.supportPath} />
        <DetailRow label={props.t('dashboard.detailModSupport')} value={props.t(modBadge().labelKey)} />
        <DetailRow label={props.t('dashboard.detailMergeMods')} value={mergeLabel()} />
        <Show when={props.game.extensionId}>
          <DetailRow
            label={props.t('dashboard.detailExtension')}
            value={props.game.extensionId!}
          />
        </Show>
        <Show when={props.game.details.steamAppId != null}>
          <DetailRow
            label={props.t('dashboard.detailSteamAppId')}
            value={String(props.game.details.steamAppId)}
          />
        </Show>
        <DetailRow
          label={props.t('dashboard.detailCreated')}
          value={formatDateTime(props.game.createdAt, props.locale)}
        />
        <DetailRow
          label={props.t('dashboard.detailUpdated')}
          value={formatDateTime(props.game.updatedAt, props.locale)}
        />
      </dl>
    </div>
  );
};

export interface GameLibraryCardProps {
  game: Game;
  managedGameId: string | null;
  t: TranslateFn;
  locale: Locale;
  onManage: (gameId: string) => void;
}

export const GameLibraryCard: Component<GameLibraryCardProps> = (props) => {
  const modBadge = createMemo(() => getGameModStatusBadge(props.game));
  const unsupported = createMemo(() => isGameUnsupportedByPantheon(props.game));

  const [statsId, setStatsId] = createSignal<string | null>(null);
  const [installStats] = createResource(statsId, async (id) => {
    if (!id) return undefined;
    return gameApi.getGameInstallStats(id);
  });

  const handleLaunch = () => {
    void launchGame(props.game).catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(err);
      window.alert(msg);
    });
  };

  return (
    <HoverCard.Root
      closeDelay={180}
      openDelay={220}
      onOpenChange={(d) => {
        if (d.open) {
          setStatsId(props.game.id);
        }
      }}
      positioning={{
        placement: 'right-start',
        gutter: 12,
        flip: true,
        shift: true,
      }}
    >
      <HoverCard.Context>
        {(hc) => (
          <div {...hc().getTriggerProps()} class="game-card-hover-wrap">
            <Card
              class={`game-card${props.game.installPathMissing ? ' game-card--missing' : ''}`}
              hoverable={!unsupported()}
              onClick={unsupported() ? undefined : () => props.onManage(props.game.id)}
            >
              <GameCardCover game={props.game} />
              <div class="game-card-body">
                <h3>{props.game.name}</h3>
                <div class="game-meta">
                  <span class={`launcher-badge launcher-${props.game.launcher}`}>
                    {props.game.launcher}
                  </span>
                  {props.game.installPathMissing && (
                    <span class="game-meta-badge game-meta-badge--missing-path">
                      {props.t('dashboard.installPathMissing')}
                    </span>
                  )}
                  <span class={`game-meta-badge game-meta-badge--${modBadge().variant}`}>
                    {props.t(modBadge().labelKey)}
                  </span>
                </div>
                <Show when={props.game.installPathMissing !== true}>
                  <div
                    class="game-card-actions"
                    onClick={(e) => e.stopPropagation()}
                    role="presentation"
                  >
                    <Show when={!unsupported() && props.managedGameId !== props.game.id}>
                      <Button
                        variant="primary"
                        size="sm"
                        class="game-card-manage"
                        onClick={() => props.onManage(props.game.id)}
                      >
                        {props.t('dashboard.manage')}
                      </Button>
                    </Show>
                    <Button
                      variant={unsupported() ? 'primary' : 'secondary'}
                      size="sm"
                      class="game-card-launch"
                      onClick={() => handleLaunch()}
                    >
                      {props.t('dashboard.launch')}
                    </Button>
                  </div>
                </Show>
              </div>
            </Card>
          </div>
        )}
      </HoverCard.Context>
      <HoverCard.Positioner class="game-card-details-positioner">
        <HoverCard.Content class="game-card-details-popover">
          <GameCardDetails
            game={props.game}
            t={props.t}
            locale={props.locale}
            installStats={installStats}
          />
        </HoverCard.Content>
      </HoverCard.Positioner>
    </HoverCard.Root>
  );
};
