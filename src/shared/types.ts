export type GameLauncher =
  | 'steam'
  | 'gog'
  | 'epic'
  | 'xbox'
  | 'origin'
  | 'ubisoft'
  | 'battlenet'
  | 'amazon'
  | 'microsoftstore'
  | 'manual';

export type ModSupportLevel = 'full' | 'partial' | 'none';

export type ModType = 
  | 'simple' 
  | 'fomod' 
  | 'foomad' 
  | 'bsat' 
  | 'bepinex' 
  | 'dazip' 
  | 'enb' 
  | 'scriptExtender' 
  | 'modPlugin'
  /** Известный путь к сохранениям в MVP (`list_game_saves` по `game.id`). */
  | 'gameSaves';

export interface GameDetails {
  steamAppId: number | null;
  gogId: string | null;
  epicAppId: string | null;
  logo: string | null;
  requiredFiles: string[];
  environment: Record<string, string>;
}

export interface Game {
  id: string;
  name: string;
  installPath: string;
  supportPath: string;
  /** Не из БД: true, если папка установки отсутствует на диске. */
  installPathMissing?: boolean;
  launcher: GameLauncher;
  extensionId: string | null;
  supportedModTypes: ModType[];
  mergeMods: boolean;
  modSupport: ModSupportLevel;
  details: GameDetails;
  createdAt: string;
  updatedAt: string;
}

export interface RemoveGameResult {
  deletedMods: number;
}

/** Ответ `get_game_install_stats`: размер папки и поля Steam `appmanifest` (buildid, SizeOnDisk). */
export interface GameInstallStats {
  /** С учётом симлинков (основная строка «Space used»). */
  diskUsageBytes: number;
  /** Без обхода симлинков (как «Space used (no symlinks)» в Vortex). */
  diskUsageBytesNoSymlinks: number;
  steamSizeOnDiskBytes: number | null;
  steamBuildId: string | null;
  installedVersionLabel: string | null;
}

export interface Mod {
  id: string;
  gameId: string;
  name: string;
  version: string | null;
  modType: string;
  installPath: string;
  enabled: boolean;
}

export interface DeploymentState {
  modId: string;
  gameId: string;
  status: string;
  strategy: string;
  deployedFiles: DeployedFile[];
  deployedAt: string | null;
}

export interface DeployedFile {
  source: string;
  target: string;
  size: number;
}

export interface DetectionProgress {
  message: string;
  found: number;
  total: number;
  currentGame: string | null;
}

export interface GameDetectionError {
  gameId: string;
  gameName: string;
  error: string;
  recoverable: boolean;
}

/** Save file row from `list_game_saves` (Tauri). */
export interface SaveFileEntry {
  name: string;
  path: string;
}