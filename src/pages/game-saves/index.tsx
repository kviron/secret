import { Component, Show, createMemo, createResource, createSignal, For } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { invoke } from '@tauri-apps/api/core';
import { gameApi } from '@/entities/game';
import { useGameStore } from '@/entities/game';
import { useI18n } from '@/shared/lib/i18n';
import { gameSupportsKnownSavesLocation } from '@/shared/lib/game-support';
import { Dialog, Button } from '@/shared/ui';
import type { SaveFileEntry, SaveBackupEntry } from '@/shared/types';
import { FolderOpen, Trash2, Shield, RotateCcw } from 'lucide-solid';

type SortKey = 'name' | 'modified' | 'size';

export const GameSavesPage: Component = () => {
  const { t } = useI18n();
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { state: gameState } = useGameStore();

  const game = () => gameState.games.find((g) => g.id === params.id);

  const supportsSaves = createMemo(() => {
    const g = game();
    return g !== undefined && gameSupportsKnownSavesLocation(g);
  });

  const [saves, { refetch }] = createResource(
    () => {
      const id = params.id;
      const g = gameState.games.find((x) => x.id === id);
      if (!id || !g || !gameSupportsKnownSavesLocation(g)) return undefined;
      return id;
    },
    async (id) => gameApi.listGameSaves(id),
  );

  const [backups, { refetch: refetchBackups }] = createResource(
    () => {
      const id = params.id;
      if (!id || !supportsSaves()) return undefined;
      return id;
    },
    async (id) => gameApi.listSaveBackups(id),
  );

  const [sortKey, setSortKey] = createSignal<SortKey>('name');
  const [sortAsc, setSortAsc] = createSignal(true);
  const [deleteTarget, setDeleteTarget] = createSignal<SaveFileEntry | null>(null);
  const [deleteBusy, setDeleteBusy] = createSignal(false);
  const [backupBusy, setBackupBusy] = createSignal<string | null>(null);
  const [showBackups, setShowBackups] = createSignal(false);
  const [restoreTarget, setRestoreTarget] = createSignal<SaveBackupEntry | null>(null);
  const [restoreBusy, setRestoreBusy] = createSignal(false);
  const [savesDir] = createResource(
    () => supportsSaves() ? params.id : undefined,
    async (id) => {
      if (!id) return null;
      return gameApi.getSavesDirPath(id);
    },
  );

  const sortedSaves = createMemo(() => {
    const list = saves() ?? [];
    const key = sortKey();
    const asc = sortAsc();
    return [...list].sort((a, b) => {
      let cmp = 0;
      if (key === 'name') cmp = a.name.localeCompare(b.name);
      else if (key === 'size') cmp = a.size - b.size;
      else if (key === 'modified') cmp = (a.modified ?? '').localeCompare(b.modified ?? '');
      return asc ? cmp : -cmp;
    });
  });

  const handleSort = (key: SortKey) => {
    if (sortKey() === key) {
      setSortAsc(!sortAsc());
    } else {
      setSortKey(key);
      setSortAsc(true);
    }
  };

  const handleBack = () => navigate(`/game/${params.id}/mods`);

  const handleOpenFolder = () => {
    const dir = savesDir();
    if (!dir) return;
    void invoke('open_folder', { path: dir }).catch((err: unknown) => {
      console.error(err);
    });
  };

  const handleDeleteConfirm = async () => {
    const target = deleteTarget();
    if (!target) return;
    setDeleteBusy(true);
    try {
      await gameApi.deleteSave(params.id, target.path);
      setDeleteTarget(null);
      await refetch();
    } catch (err) {
      console.error(err);
      window.alert(String(err));
    } finally {
      setDeleteBusy(false);
    }
  };

  const handleBackup = async (save: SaveFileEntry) => {
    setBackupBusy(save.path);
    try {
      await gameApi.backupSave(params.id, save.path);
      await refetchBackups();
    } catch (err) {
      console.error(err);
      window.alert(String(err));
    } finally {
      setBackupBusy(null);
    }
  };

  const handleRestoreConfirm = async () => {
    const target = restoreTarget();
    if (!target) return;
    setRestoreBusy(true);
    try {
      await gameApi.restoreSave(params.id, target.path);
      setRestoreTarget(null);
      await refetch();
      await refetchBackups();
    } catch (err) {
      console.error(err);
      window.alert(String(err));
    } finally {
      setRestoreBusy(false);
    }
  };

  const formatDate = (iso?: string) => {
    if (!iso) return '\u2014';
    try {
      const d = new Date(iso + 'Z');
      return d.toLocaleString();
    } catch {
      return iso;
    }
  };

  const SortHeader = (props: { label: string; field: SortKey }) => (
    <th
      class="saves-table-th saves-table-th--sortable"
      onClick={() => handleSort(props.field)}
    >
      {props.label}
      <Show when={sortKey() === props.field}>
        <span class="saves-table-sort">{sortAsc() ? ' \u2191' : ' \u2193'}</span>
      </Show>
    </th>
  );

  return (
    <>
      <header class="top-bar">
        <div class="top-bar-left">
          <button type="button" class="btn-back" onClick={handleBack} title={t('savesPage.backTitle')}>
            ←
          </button>
          <h1 class="page-title">{t('savesPage.title')}</h1>
          <span class="mod-count">{game()?.name ?? ''}</span>
        </div>
        <Show when={supportsSaves()}>
          <div class="saves-toolbar">
            <Show when={savesDir()}>
              <button
                type="button"
                class="saves-toolbar-btn"
                onClick={handleOpenFolder}
                title={t('savesPage.openFolder')}
              >
                <FolderOpen size={16} />
                <span>{t('savesPage.openFolder')}</span>
              </button>
            </Show>
            <button
              type="button"
              class={`saves-toolbar-btn ${showBackups() ? 'saves-toolbar-btn--active' : ''}`}
              onClick={() => setShowBackups(!showBackups())}
              title={t('savesPage.showBackups')}
            >
              <Shield size={16} />
              <span>{t('savesPage.backups')} ({(backups() ?? []).length})</span>
            </button>
          </div>
        </Show>
      </header>

      <Show when={!supportsSaves()}>
        <div class="empty-state">
          <div class="empty-icon">💾</div>
          <h2>{t('savesPage.unsupportedTitle')}</h2>
          <p>{t('savesPage.unsupportedDesc')}</p>
        </div>
      </Show>

      <Show when={supportsSaves() && saves.error}>
        <div class="alert alert-error">
          <span class="alert-icon">⚠️</span>
          {String(saves.error)}
        </div>
      </Show>

      <Show when={supportsSaves() && saves.loading}>
        <p class="text-muted">{t('savesPage.loading')}</p>
      </Show>

      <Show when={showBackups() && supportsSaves()}>
        <div class="saves-backups-panel">
          <h3 class="saves-backups-title">{t('savesPage.backupsTitle')}</h3>
          <Show when={(backups() ?? []).length === 0}>
            <p class="text-muted">{t('savesPage.noBackups')}</p>
          </Show>
          <Show when={(backups() ?? []).length > 0}>
            <table class="saves-table">
              <thead>
                <tr>
                  <th class="saves-table-th">{t('savesPage.colBackupName')}</th>
                  <th class="saves-table-th">{t('savesPage.colDate')}</th>
                  <th class="saves-table-th">{t('savesPage.colSize')}</th>
                  <th class="saves-table-th saves-table-th--actions">{t('savesPage.colActions')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={backups() ?? []}>
                  {(backup) => (
                    <tr class="saves-table-row">
                      <td class="saves-table-td saves-table-td--name">{backup.originalSaveName}</td>
                      <td class="saves-table-td">{formatDate(backup.created)}</td>
                      <td class="saves-table-td saves-table-td--size">{backup.sizeLabel}</td>
                      <td class="saves-table-td saves-table-td--actions">
                        <button
                          type="button"
                          class="saves-action-btn saves-action-btn--restore"
                          onClick={() => setRestoreTarget(backup)}
                          title={t('savesPage.restore')}
                        >
                          <RotateCcw size={14} />
                        </button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </Show>
        </div>
      </Show>

      <Show when={supportsSaves() && !saves.loading && !saves.error}>
        <Show
          when={sortedSaves().length > 0}
          fallback={
            <div class="empty-state">
              <div class="empty-icon">💾</div>
              <h2>{t('savesPage.emptyTitle')}</h2>
              <p>{t('savesPage.emptyDesc')}</p>
            </div>
          }
        >
          <div class="saves-table-wrap">
            <table class="saves-table">
              <thead>
                <tr>
                  <SortHeader label={t('savesPage.colName')} field="name" />
                  <SortHeader label={t('savesPage.colDate')} field="modified" />
                  <SortHeader label={t('savesPage.colSize')} field="size" />
                  <th class="saves-table-th saves-table-th--actions">{t('savesPage.colActions')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={sortedSaves()}>
                  {(save) => (
                    <tr class="saves-table-row">
                      <td class="saves-table-td saves-table-td--name" title={save.path}>{save.name}</td>
                      <td class="saves-table-td">{formatDate(save.modified)}</td>
                      <td class="saves-table-td saves-table-td--size">{save.sizeLabel}</td>
                      <td class="saves-table-td saves-table-td--actions">
                      <button
                        type="button"
                        class="saves-action-btn"
                        onClick={() => {
                          const sep = save.path.includes('\\') ? '\\' : '/';
                          const dir = save.path.substring(0, save.path.lastIndexOf(sep));
                          void invoke('open_folder', { path: dir }).catch(console.error);
                        }}
                        title={t('savesPage.openSaveFolder')}
                      >
                        <FolderOpen size={14} />
                      </button>
                      <button
                        type="button"
                        class="saves-action-btn saves-action-btn--backup"
                        onClick={() => void handleBackup(save)}
                        disabled={backupBusy() === save.path}
                        title={t('savesPage.backup')}
                      >
                        <Shield size={14} />
                      </button>
                      <button
                        type="button"
                        class="saves-action-btn saves-action-btn--delete"
                        onClick={() => setDeleteTarget(save)}
                        title={t('savesPage.delete')}
                      >
                        <Trash2 size={14} />
                      </button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </Show>

      <Dialog
        open={deleteTarget() !== null}
        onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}
        title={t('savesPage.deleteDialogTitle')}
        description={t('savesPage.deleteDialogDesc')}
        actions={
          <>
            <Button variant="secondary" onClick={() => setDeleteTarget(null)}>
              {t('gameDetail.cancel')}
            </Button>
            <Button variant="danger" onClick={() => void handleDeleteConfirm()} isLoading={deleteBusy()}>
              {t('savesPage.delete')}
            </Button>
          </>
        }
      >
        <p class="saves-dialog-file-name">{deleteTarget()?.name}</p>
      </Dialog>

      <Dialog
        open={restoreTarget() !== null}
        onOpenChange={(open) => { if (!open) setRestoreTarget(null); }}
        title={t('savesPage.restoreDialogTitle')}
        description={t('savesPage.restoreDialogDesc')}
        actions={
          <>
            <Button variant="secondary" onClick={() => setRestoreTarget(null)}>
              {t('gameDetail.cancel')}
            </Button>
            <Button variant="primary" onClick={() => void handleRestoreConfirm()} isLoading={restoreBusy()}>
              {t('savesPage.restore')}
            </Button>
          </>
        }
      >
        <p class="saves-dialog-file-name">{restoreTarget()?.originalSaveName}</p>
      </Dialog>
    </>
  );
};
