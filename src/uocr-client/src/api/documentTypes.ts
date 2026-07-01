export interface DocumentSummary {
  file_hash: string;
  display_name: string;
  relative_path?: string;
  status: string;
  page_count: number;
  processed_pages?: number;
  total_pages?: number;
  current_page?: number;
  progress_percent?: number;
  regions?: number;
  error?: string;
}

export interface DocumentsPayload {
  documents: DocumentSummary[];
}

export interface DocumentRegionsPayload {
  file_hash: string;
  boxes: OverlayBox[];
}

export interface OverlayBox {
  region_id: string;
  label: string;
  content_markdown?: string;
  content_html?: string | null;
  page_no: number;
  left_percent: number;
  top_percent: number;
  width_percent: number;
  height_percent: number;
  hidden?: boolean;
}

export interface TextRegionSpan {
  region_id: string;
  page_no: number;
  start: number;
  end: number;
}

export interface PageTextRecord {
  page_no: number;
  text: string;
  spans: TextRegionSpan[];
}

export interface DocumentTextPayload {
  file_hash: string;
  pages: PageTextRecord[];
}

export interface FolderDialogResponse {
  cancelled: boolean;
  selected_path: string;
  manual_path_supported: boolean;
  error?: string;
}

export interface AnnotationSettingsPayload {
  show_boxes: boolean;
  show_labels: boolean;
  box_color: string;
  active_box_color: string;
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
