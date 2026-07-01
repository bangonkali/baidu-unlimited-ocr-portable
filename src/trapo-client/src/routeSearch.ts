export interface WorkbenchRouteSearch {
  file?: string;
  follow?: boolean;
  labels?: boolean;
  overlays?: boolean;
  page?: number;
  q?: string;
  region?: string;
}

export type ModelViewMode = 'grid' | 'cards';
export type ModelSortKey =
  | 'name'
  | 'size'
  | 'bits'
  | 'vram'
  | 'status'
  | 'progress'
  | 'speed'
  | 'eta';
export type SortDirection = 'asc' | 'desc';
export type DownloadStatusFilter = 'active' | 'queued' | 'pending' | 'all';

export interface ModelRouteSearch {
  dir?: SortDirection;
  model?: string;
  sort?: ModelSortKey;
  status?: DownloadStatusFilter;
  view?: ModelViewMode;
}

export type SettingsSection = 'appearance' | 'runtime' | 'ocr' | 'storage' | 'models';

export interface SettingsRouteSearch {
  section?: SettingsSection;
}

export interface DiagnosticsRouteSearch {
  component?: string;
  level?: string;
  q?: string;
  run?: string;
  status?: string;
  tab?: 'logs' | 'runs';
}

export interface IngestRouteSearch {
  model?: string;
  profile?: string;
  reprocess?: boolean;
}

export function validateWorkbenchSearch(search: Record<string, unknown>): WorkbenchRouteSearch {
  return {
    file: stringValue(search.file) ?? stringValue(search.file_hash),
    follow: booleanValue(search.follow),
    labels: booleanValue(search.labels),
    overlays: booleanValue(search.overlays),
    page: positiveIntegerValue(search.page) ?? positiveIntegerValue(search.page_no),
    q: stringValue(search.q),
    region: stringValue(search.region) ?? stringValue(search.region_id),
  };
}

export function validateModelSearch(search: Record<string, unknown>): ModelRouteSearch {
  return {
    dir: sortDirectionValue(search.dir),
    model: stringValue(search.model),
    sort: modelSortValue(search.sort),
    status: downloadStatusValue(search.status),
    view: modelViewValue(search.view),
  };
}

export function validateSettingsSearch(search: Record<string, unknown>): SettingsRouteSearch {
  const value = stringValue(search.section);
  return {
    section:
      value === 'appearance' ||
      value === 'runtime' ||
      value === 'ocr' ||
      value === 'storage' ||
      value === 'models'
        ? value
        : undefined,
  };
}

export function validateDiagnosticsSearch(search: Record<string, unknown>): DiagnosticsRouteSearch {
  const tab = stringValue(search.tab);
  return {
    component: stringValue(search.component),
    level: stringValue(search.level),
    q: stringValue(search.q),
    run: stringValue(search.run),
    status: stringValue(search.status),
    tab: tab === 'logs' || tab === 'runs' ? tab : undefined,
  };
}

export function validateIngestSearch(search: Record<string, unknown>): IngestRouteSearch {
  return {
    model: stringValue(search.model),
    profile: stringValue(search.profile),
    reprocess: booleanValue(search.reprocess),
  };
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value : undefined;
}

function positiveIntegerValue(value: unknown): number | undefined {
  const parsed =
    typeof value === 'number' ? value : typeof value === 'string' ? Number(value) : Number.NaN;
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}

function booleanValue(value: unknown): boolean | undefined {
  if (value === true || value === '1' || value === 'true') {
    return true;
  }
  if (value === false || value === '0' || value === 'false') {
    return false;
  }
  return undefined;
}

function sortDirectionValue(value: unknown): SortDirection | undefined {
  return value === 'asc' || value === 'desc' ? value : undefined;
}

function modelViewValue(value: unknown): ModelViewMode | undefined {
  return value === 'grid' || value === 'cards' ? value : undefined;
}

function modelSortValue(value: unknown): ModelSortKey | undefined {
  return value === 'name' ||
    value === 'size' ||
    value === 'bits' ||
    value === 'vram' ||
    value === 'status' ||
    value === 'progress' ||
    value === 'speed' ||
    value === 'eta'
    ? value
    : undefined;
}

function downloadStatusValue(value: unknown): DownloadStatusFilter | undefined {
  return value === 'active' || value === 'queued' || value === 'pending' || value === 'all'
    ? value
    : undefined;
}
