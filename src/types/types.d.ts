export interface GameExecutable {
  is_launcher: boolean;
  name: string;
  os: string;
  filename?: string;
  path?: string;
  segments?: number;
  is_running?: boolean;
  is_installed?: boolean;
}

export interface Game {
  uid?: string;
  id: string;
  name: string;
  executables: GameExecutable[];
  aliases?: string[];
  themes?: string[];
  is_running?: boolean;
  is_installed?: boolean;
}

export interface GameActionsProvider {
  canPlayGame: (game: Game | null) => boolean;
  isGameInstalled: (game: Game | null) => boolean;
  isExecutableRunning: (executable: GameExecutable) => boolean;
  isGameExecutableInstalled: (executable: GameExecutable) => boolean;
}

export interface SessionState {
  session_id: string;
  app_id: number;
  game_name: string;
  executable_name: string;
  start_timestamp: number;
  is_active: boolean;
  last_sync_timestamp: number;
}

export interface AppSettings {
  minimize_to_tray: boolean;
}

export interface SystemResumePayload {
  resumed_at: number;
  gap_seconds: number;
  session_start_timestamp?: number;
  elapsed_seconds?: number;
  game_name?: string;
  app_id?: number;
}
