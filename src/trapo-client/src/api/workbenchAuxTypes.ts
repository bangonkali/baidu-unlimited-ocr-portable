export interface FolderDialogResponse {
  cancelled: boolean;
  selected_path: string;
  manual_path_supported: boolean;
  error?: string;
}

export interface PreviewImagesPayload {
  file_hash: string;
  variants: string[];
  pages: number[];
}

export interface LogRecord {
  timestamp: string;
  level: string;
  component: string;
  message: string;
}

export interface LogsPayload {
  log_path?: string;
  logs: LogRecord[];
}
