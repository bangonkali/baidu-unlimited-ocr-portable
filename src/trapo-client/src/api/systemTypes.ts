export interface ShutdownRequest {
  confirm: string;
}

export interface ShutdownPayload {
  active_download_count: number;
  active_run_ids: string[];
  grace_ms: number;
  message: string;
  source: string;
  state: string;
}
