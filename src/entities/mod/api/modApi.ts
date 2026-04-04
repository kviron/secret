import { api } from '@/shared/api/client';
import type { Mod, DeploymentState } from '@/shared/types';

export const modApi = {
  getMods: (gameId: string) => api.invoke<Mod[]>('get_mods', { gameId }),
  installMod: (gameId: string, archivePath: string) =>
    api.invoke<Mod>('install_mod', { gameId, archivePath }),
  uninstallMod: (modId: string) => api.invoke<void>('uninstall_mod', { modId }),
  setModEnabled: (modId: string, enabled: boolean) =>
    api.invoke<void>('set_mod_enabled', { modId, enabled }),
  deployMod: (modId: string) => api.invoke<DeploymentState>('deploy_mod', { modId }),
  undeployMod: (modId: string) => api.invoke<void>('undeploy_mod', { modId }),
  deployAll: (gameId: string) => api.invoke<DeploymentState[]>('deploy_all', { gameId }),
};