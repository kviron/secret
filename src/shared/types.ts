export type GameLauncher = 'steam' | 'gog' | 'epic' | 'xbox' | 'origin' | 'manual';

export type ModType = 
  | 'simple' 
  | 'fomod' 
  | 'foomad' 
  | 'bsat' 
  | 'bepinex' 
  | 'dazip' 
  | 'enb' 
  | 'scriptExtender' 
  | 'modPlugin';

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
  launcher: GameLauncher;
  extensionId: string | null;
  supportedModTypes: ModType[];
  mergeMods: boolean;
  details: GameDetails;
  createdAt: string;
  updatedAt: string;
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