import {
  ArrowDownAZ,
  ArrowUpAZ,
  Cpu,
  DownloadCloud,
  HardDriveDownload,
  Library,
} from 'lucide-react';
import { useState } from 'react';

import type { ModelsPayload, StatusPayload } from '../../api/types';
import type {
  DownloadStatusFilter,
  ModelRouteSearch,
  ModelSortKey,
  ModelViewMode,
  SortDirection,
} from '../../routeSearch';
import { DownloadManager } from './DownloadManager';
import { ModelCards } from './ModelCards';
import { ModelDataGrid } from './ModelDataGrid';
import styles from './ModelManager.module.css';
import { formatBytes } from './modelDownloadFormat';
import { modelRequiredBytes, visibleModels } from './modelLibrary';

interface ModelManagerProps {
  busy?: boolean;
  models?: ModelsPayload;
  routeSearch?: ModelRouteSearch;
  scope?: 'library' | 'downloads';
  status?: StatusPayload;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onRouteSearchChange?: (patch: Partial<ModelRouteSearch>) => void;
  onScopeChange?: (scope: 'library' | 'downloads') => void;
  onSelectModel: (modelId: string) => void;
}

export function ModelManager(props: ModelManagerProps) {
  const library = props.models?.models ?? [];
  const selected =
    library.find((model) => model.selected) ??
    library.find((model) => model.model_id === props.models?.selected_model_id) ??
    library[0];
  const scope = props.scope ?? 'library';
  const view = props.routeSearch?.view ?? 'grid';
  const sort = props.routeSearch?.sort ?? 'status';
  const dir = props.routeSearch?.dir ?? 'asc';
  const statusFilter = props.routeSearch?.status ?? 'all';
  const shown = visibleModels(library, { dir, scope, sort, status: statusFilter });
  const updateSearch = props.onRouteSearchChange ?? (() => undefined);
  const [downloadsOpen, setDownloadsOpen] = useState(scope === 'downloads');
  const changeSort = (nextSort: ModelSortKey) =>
    updateSearch({ dir: nextSort === sort && dir === 'asc' ? 'desc' : 'asc', sort: nextSort });

  return (
    <section className={styles.manager} aria-label="Models" data-tour="models">
      <header className={styles.header}>
        <div className={styles.headerTitle}>
          <Library size={16} />
          <span>{scope === 'downloads' ? 'Active Downloads' : 'Model Library'}</span>
        </div>
        <span className={styles.provider}>{props.models?.provider_repo ?? selected?.repo_id}</span>
      </header>
      <ModelSummary selected={selected} status={props.status} />
      <ModelToolbar
        dir={dir}
        scope={scope}
        sort={sort}
        status={statusFilter}
        view={view}
        onDirChange={(nextDir) => updateSearch({ dir: nextDir })}
        onSortChange={(nextSort) => updateSearch({ sort: nextSort })}
        onStatusChange={(nextStatus) => updateSearch({ status: nextStatus })}
        onViewChange={(nextView) => updateSearch({ view: nextView })}
        onScopeChange={props.onScopeChange ?? (() => undefined)}
        onDownloadsOpen={() => setDownloadsOpen(true)}
      />
      {view === 'cards' ? (
        <ModelCards {...props} models={shown} />
      ) : (
        <ModelDataGrid {...props} dir={dir} models={shown} sort={sort} onSortChange={changeSort} />
      )}
      {downloadsOpen ? (
        <DownloadManager
          busy={props.busy}
          models={library}
          onCancelModel={props.onCancelModel}
          onClose={() => setDownloadsOpen(false)}
        />
      ) : null}
    </section>
  );
}

function ModelSummary({
  selected,
  status,
}: {
  selected?: ModelsPayload['models'][number];
  status?: StatusPayload;
}) {
  return (
    <div className={styles.summary}>
      <div>
        <span className={styles.eyebrow}>Selected model</span>
        <h2>{selected?.display_name ?? 'No model selected'}</h2>
        <p>{selected?.notes ?? 'Choose a model variant and download its required files.'}</p>
      </div>
      <div className={styles.summaryStats}>
        <span>
          <Cpu size={14} />
          {status?.runtime_platform ?? 'windows-x86_64-cuda13'} / {status?.accelerator ?? 'cuda'}
        </span>
        <span>
          <HardDriveDownload size={14} />
          {selected ? formatBytes(modelRequiredBytes(selected)) : '0 B'}
        </span>
      </div>
    </div>
  );
}

function ModelToolbar(props: {
  dir: SortDirection;
  scope: 'library' | 'downloads';
  sort: ModelSortKey;
  status: DownloadStatusFilter;
  view: ModelViewMode;
  onDirChange: (dir: SortDirection) => void;
  onSortChange: (sort: ModelSortKey) => void;
  onScopeChange: (scope: 'library' | 'downloads') => void;
  onStatusChange: (status: DownloadStatusFilter) => void;
  onDownloadsOpen: () => void;
  onViewChange: (view: ModelViewMode) => void;
}) {
  const DirectionIcon = props.dir === 'desc' ? ArrowDownAZ : ArrowUpAZ;
  return (
    <div className={styles.toolbar}>
      <div className={styles.segmented}>
        <button
          aria-pressed={props.view === 'grid'}
          onClick={() => props.onViewChange('grid')}
          type="button"
        >
          Grid
        </button>
        <button
          aria-pressed={props.view === 'cards'}
          onClick={() => props.onViewChange('cards')}
          type="button"
        >
          Cards
        </button>
      </div>
      <label>
        <span>Sort</span>
        <select
          onChange={(event) => props.onSortChange(event.target.value as ModelSortKey)}
          value={props.sort}
        >
          <option value="status">Status</option>
          <option value="progress">Progress</option>
          <option value="size">Size</option>
          <option value="bits">Bits</option>
          <option value="vram">VRAM tier</option>
          <option value="speed">Speed</option>
          <option value="eta">ETA</option>
          <option value="name">Name</option>
        </select>
      </label>
      <button
        className={styles.directionButton}
        onClick={() => props.onDirChange(props.dir === 'asc' ? 'desc' : 'asc')}
        type="button"
      >
        <DirectionIcon size={14} strokeWidth={1.9} />
        {props.dir}
      </button>
      {props.scope === 'downloads' ? (
        <label>
          <span>Status</span>
          <select
            onChange={(event) => props.onStatusChange(event.target.value as DownloadStatusFilter)}
            value={props.status}
          >
            <option value="all">All active</option>
            <option value="active">Active</option>
            <option value="queued">Queued</option>
          </select>
        </label>
      ) : null}
      <button
        className={styles.routeLink}
        onClick={() => props.onScopeChange(props.scope === 'downloads' ? 'library' : 'downloads')}
        type="button"
      >
        {props.scope === 'downloads' ? 'Library' : 'Active Downloads'}
      </button>
      <button
        className={styles.iconButton}
        onClick={props.onDownloadsOpen}
        title="Downloads"
        type="button"
      >
        <DownloadCloud size={15} strokeWidth={1.9} />
      </button>
    </div>
  );
}
