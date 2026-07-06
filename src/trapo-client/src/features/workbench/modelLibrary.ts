import type { ModelAssetRecord } from '../../api/types';
import type {
  DownloadStatusFilter,
  ModelOriginFilter,
  ModelSortKey,
  SortDirection,
} from '../../routeSearch';

export interface ModelLibraryOptions {
  dir?: SortDirection;
  origin?: ModelOriginFilter;
  scope?: 'library' | 'downloads';
  sort?: ModelSortKey;
  status?: DownloadStatusFilter;
}

export function visibleModels(models: ModelAssetRecord[], options: ModelLibraryOptions) {
  const filtered = filterModels(
    models,
    options.scope ?? 'library',
    options.status ?? 'all',
    options.origin ?? 'all',
  );
  return [...filtered].sort((left, right) =>
    compareModels(left, right, options.sort ?? 'status', options.dir ?? 'asc'),
  );
}

export function modelPercent(model: ModelAssetRecord) {
  if (model.status === 'downloaded') {
    return 100;
  }
  const total = model.overall_total_bytes ?? model.total_bytes ?? 0;
  const downloaded = model.overall_downloaded_bytes ?? model.downloaded_bytes ?? 0;
  return total > 0 ? Math.min(100, (downloaded / total) * 100) : (model.overall_percent ?? 0);
}

export function modelRequiredBytes(model: ModelAssetRecord) {
  return model.total_required_bytes ?? model.overall_total_bytes ?? model.total_bytes ?? 0;
}

export function modelDownloadedBytes(model: ModelAssetRecord) {
  return model.overall_downloaded_bytes ?? model.downloaded_bytes ?? 0;
}

function filterModels(
  models: ModelAssetRecord[],
  scope: 'library' | 'downloads',
  status: DownloadStatusFilter,
  origin: ModelOriginFilter,
) {
  return models.filter(
    (model) =>
      originMatches(model, origin) &&
      (scope === 'library' || statusMatches(model.status, scope, status)),
  );
}

function originMatches(model: ModelAssetRecord, origin: ModelOriginFilter) {
  return origin === 'all' || model.routing_origin === origin;
}

function statusMatches(
  status: string,
  scope: 'library' | 'downloads',
  filter: DownloadStatusFilter,
) {
  if (filter === 'all') {
    return scope === 'downloads' ? ['downloading', 'queued', 'cancelling'].includes(status) : true;
  }
  if (filter === 'active') {
    return status === 'downloading' || status === 'cancelling';
  }
  if (filter === 'queued') {
    return status === 'queued';
  }
  return false;
}

function compareModels(
  left: ModelAssetRecord,
  right: ModelAssetRecord,
  sort: ModelSortKey,
  dir: SortDirection,
) {
  const direction = dir === 'desc' ? -1 : 1;
  const compared = compareValue(sortValue(left, sort), sortValue(right, sort));
  return compared * direction || left.display_name.localeCompare(right.display_name);
}

function sortValue(model: ModelAssetRecord, sort: ModelSortKey) {
  switch (sort) {
    case 'bits':
      return model.bits ?? 0;
    case 'eta':
      return model.eta_seconds ?? Number.POSITIVE_INFINITY;
    case 'name':
      return model.display_name;
    case 'progress':
      return modelPercent(model);
    case 'size':
      return modelRequiredBytes(model);
    case 'speed':
      return model.bytes_per_second ?? 0;
    case 'status':
      return statusRank(model.status);
    case 'vram':
      return model.hardware_tier ?? '';
  }
}

function compareValue(left: number | string, right: number | string) {
  if (typeof left === 'number' && typeof right === 'number') {
    return left - right;
  }
  return String(left).localeCompare(String(right), undefined, { numeric: true });
}

function statusRank(status: string) {
  if (status === 'downloading') {
    return 0;
  }
  if (status === 'cancelling') {
    return 1;
  }
  if (status === 'queued') {
    return 2;
  }
  if (status === 'missing') {
    return 3;
  }
  if (status === 'failed') {
    return 4;
  }
  if (status === 'cancelled') {
    return 5;
  }
  if (status === 'downloaded') {
    return 6;
  }
  return 6;
}
