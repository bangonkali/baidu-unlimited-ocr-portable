import { CircleAlert, CircleCheck, FileText, LoaderCircle } from 'lucide-react';

import type {
  IngestRunRecord,
  LogRecord,
  OcrMetricsTreeNode,
  OcrMetricsTreePayload,
} from '../../api/types';
import type { TreeDataGridColumn } from '../../components/TreeDataGrid';
import { TreeDataGrid } from '../../components/TreeDataGrid';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import styles from './DiagnosticsPanel.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

interface DiagnosticsPanelProps {
  logs: LogRecord[];
  metrics: OcrMetricsTreePayload;
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}

export function DiagnosticsPanel({
  logs,
  metrics,
  onSearchChange,
  runs,
  search,
}: DiagnosticsPanelProps) {
  const tab = search?.tab ?? 'runs';
  const filteredLogs = filterLogs(logs, search);
  const filteredMetrics = filterMetricNodes(metrics.nodes, search);
  const filteredRuns = filterRuns(runs, search);
  const metricRows = flattenMetricNodes(metrics.nodes);
  return (
    <section className={styles.panel} aria-label="Diagnostics" data-tour="diagnostics">
      <div className={styles.header}>Diagnostics</div>
      <div className={styles.tabs}>
        <button
          className={styles.tab}
          data-active={tab === 'runs'}
          onClick={() => onSearchChange?.({ tab: 'runs' })}
          type="button"
        >
          Runs
        </button>
        <button
          className={styles.tab}
          data-active={tab === 'metrics'}
          onClick={() => onSearchChange?.({ tab: 'metrics' })}
          type="button"
        >
          Metrics
        </button>
        <button
          className={styles.tab}
          data-active={tab === 'logs'}
          onClick={() => onSearchChange?.({ tab: 'logs' })}
          type="button"
        >
          Logs
        </button>
      </div>
      <DiagnosticsFilters
        logs={logs}
        metrics={metricRows}
        runs={runs}
        search={search}
        tab={tab}
        onSearchChange={onSearchChange}
      />
      <div className={styles.body}>
        {tab === 'runs' ? <RunList runs={filteredRuns} /> : null}
        {tab === 'metrics' ? <MetricsTree nodes={filteredMetrics} /> : null}
        {tab === 'logs' ? <LogList logs={filteredLogs} /> : null}
      </div>
    </section>
  );
}

export function filterLogs(logs: LogRecord[], search: DiagnosticsRouteSearch | undefined) {
  const query = normalized(search?.q);
  return logs.filter((log) => {
    if (search?.level && log.level !== search.level) {
      return false;
    }
    if (search?.component && log.component !== search.component) {
      return false;
    }
    if (!query) {
      return true;
    }
    return `${log.timestamp} ${log.level} ${log.component} ${log.message}`
      .toLowerCase()
      .includes(query);
  });
}

export function filterRuns(runs: IngestRunRecord[], search: DiagnosticsRouteSearch | undefined) {
  const query = normalized(search?.q);
  return runs.filter((run) => {
    if (search?.run && run.run_id !== search.run) {
      return false;
    }
    if (search?.status && run.status !== search.status) {
      return false;
    }
    if (!query) {
      return true;
    }
    return `${run.run_id} ${run.status} ${run.root_path} ${run.profile_id ?? ''}`
      .toLowerCase()
      .includes(query);
  });
}

function DiagnosticsFilters({
  logs,
  metrics,
  onSearchChange,
  runs,
  search,
  tab,
}: {
  logs: LogRecord[];
  metrics: OcrMetricsTreeNode[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  tab: 'logs' | 'metrics' | 'runs';
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}) {
  return (
    <div className={styles.filters}>
      <input
        aria-label="Filter diagnostics"
        onChange={(event) => onSearchChange?.({ q: event.target.value || undefined })}
        placeholder="Filter diagnostics"
        value={search?.q ?? ''}
      />
      {tab === 'logs' ? (
        <>
          <FilterSelect
            label="Level"
            onChange={(level) => onSearchChange?.({ level })}
            options={uniqueValues(logs.map((log) => log.level))}
            value={search?.level}
          />
          <FilterSelect
            label="Component"
            onChange={(component) => onSearchChange?.({ component })}
            options={uniqueValues(logs.map((log) => log.component))}
            value={search?.component}
          />
        </>
      ) : tab === 'metrics' ? (
        <>
          <FilterSelect
            label="Status"
            onChange={(status) => onSearchChange?.({ status })}
            options={uniqueValues(metrics.map((item) => item.status))}
            value={search?.status}
          />
          <FilterSelect
            label="Run"
            onChange={(run) => onSearchChange?.({ run })}
            options={uniqueValues(metrics.map((item) => item.run_id))}
            value={search?.run}
          />
        </>
      ) : (
        <>
          <FilterSelect
            label="Status"
            onChange={(status) => onSearchChange?.({ status })}
            options={uniqueValues(runs.map((run) => String(run.status)))}
            value={search?.status}
          />
          <FilterSelect
            label="Run"
            onChange={(run) => onSearchChange?.({ run })}
            options={uniqueValues(runs.map((item) => item.run_id))}
            value={search?.run}
          />
        </>
      )}
    </div>
  );
}

const metricColumns: Array<TreeDataGridColumn<OcrMetricsTreeNode>> = [
  {
    header: 'Name',
    id: 'name',
    render: (node) => (
      <span className={styles.metricName}>
        <span className={styles.kindBadge}>{node.kind}</span>
        <span>{node.label}</span>
      </span>
    ),
    width: 'minmax(240px, 1.6fr)',
  },
  {
    header: 'Status',
    id: 'status',
    render: (node) => <strong data-status={node.status}>{node.status}</strong>,
    width: '112px',
  },
  {
    header: 'Model',
    id: 'model',
    render: (node) => node.model_id ?? '',
    width: 'minmax(150px, 1fr)',
  },
  {
    header: 'Runtime',
    id: 'runtime',
    render: (node) => node.runtime_platform || node.runtime_id || '',
    width: 'minmax(150px, 1fr)',
  },
  {
    align: 'right',
    header: 'Tokens',
    id: 'tokens',
    render: (node) => integerLabel(node.token_count),
    width: '84px',
  },
  {
    align: 'right',
    header: 'Avg tok/s',
    id: 'avg',
    render: (node) => tpsLabel(node.avg_tps),
    width: '88px',
  },
  {
    align: 'right',
    header: 'Min',
    id: 'min',
    render: (node) => tpsLabel(node.min_tps),
    width: '70px',
  },
  {
    align: 'right',
    header: 'Max',
    id: 'max',
    render: (node) => tpsLabel(node.max_tps),
    width: '70px',
  },
  {
    align: 'right',
    header: 'Duration',
    id: 'duration',
    render: (node) => durationLabel(node.generation_duration_ms),
    width: '82px',
  },
];

function MetricsTree({ nodes }: { nodes: OcrMetricsTreeNode[] }) {
  return (
    <TreeDataGrid
      ariaLabel="OCR metrics"
      columns={metricColumns}
      defaultExpandedDepth={2}
      emptyLabel="No OCR metrics"
      nodes={nodes}
    />
  );
}

function FilterSelect({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  options: string[];
  value?: string;
  onChange: (value: string | undefined) => void;
}) {
  return (
    <label>
      <span className={styles.filterLabelText}>{label}</span>
      <select onChange={(event) => onChange(event.target.value || undefined)} value={value ?? ''}>
        <option value="">All</option>
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function RunList({ runs }: { runs: IngestRunRecord[] }) {
  return (
    <>
      {runs.length === 0 ? <div className={styles.empty}>No runs</div> : null}
      {runs.map((run) => (
        <div className={styles.runRow} key={run.run_id}>
          {iconForStatus(run.status)}
          <span>{run.run_id}</span>
          <strong>{run.status}</strong>
          <small>
            {runPageLabel(run)} · {percentLabel(run.progress_percent)}
          </small>
          <span
            aria-label={`Run ${run.run_id} progress ${percentLabel(run.progress_percent)}`}
            aria-valuemax={100}
            aria-valuemin={0}
            aria-valuenow={Math.round(clampProgress(run.progress_percent))}
            className={styles.progressTrack}
            role="progressbar"
          >
            <span style={{ width: `${clampProgress(run.progress_percent)}%` }} />
          </span>
        </div>
      ))}
    </>
  );
}

function LogList({ logs }: { logs: LogRecord[] }) {
  return (
    <>
      {logs.length === 0 ? <div className={styles.empty}>No logs</div> : null}
      {logs.map((log) => (
        <div
          className={styles.logRow}
          key={`${log.timestamp}-${log.level}-${log.component}-${log.message}`}
        >
          <FileText size={14} />
          <span>{log.timestamp}</span>
          <strong data-level={log.level}>{log.level}</strong>
          <em>{log.component}</em>
          <p>{log.message}</p>
        </div>
      ))}
    </>
  );
}

function iconForStatus(status: string) {
  if (status === 'completed') {
    return <CircleCheck size={14} className={styles.ok} />;
  }
  if (status === 'failed' || status === 'cancelled') {
    return <CircleAlert size={14} className={styles.bad} />;
  }
  return <LoaderCircle size={14} className={styles.pending} />;
}

function normalized(value: string | undefined) {
  return value?.trim().toLowerCase() ?? '';
}

function uniqueValues(values: string[]) {
  return [...new Set(values.filter(Boolean))].sort((left, right) => left.localeCompare(right));
}

function filterMetricNodes(
  nodes: OcrMetricsTreeNode[],
  search: DiagnosticsRouteSearch | undefined,
): OcrMetricsTreeNode[] {
  const query = normalized(search?.q);
  return nodes
    .map((node) => filterMetricNode(node, search, query))
    .filter((node): node is OcrMetricsTreeNode => Boolean(node));
}

function filterMetricNode(
  node: OcrMetricsTreeNode,
  search: DiagnosticsRouteSearch | undefined,
  query: string,
): OcrMetricsTreeNode | null {
  const children = node.children
    ?.map((child) => filterMetricNode(child, search, query))
    .filter((child): child is OcrMetricsTreeNode => Boolean(child));
  const matchesRun = !search?.run || node.run_id === search.run;
  const matchesStatus = !search?.status || node.status === search.status;
  const haystack =
    `${node.label} ${node.run_id} ${node.file_hash ?? ''} ${node.model_id ?? ''} ${node.runtime_id ?? ''} ${
      node.runtime_platform ?? ''
    } ${node.accelerator ?? ''}`.toLowerCase();
  const matchesQuery = !query || haystack.includes(query);
  if ((matchesRun && matchesStatus && matchesQuery) || (children?.length ?? 0) > 0) {
    return { ...node, children };
  }
  return null;
}

function flattenMetricNodes(nodes: OcrMetricsTreeNode[]): OcrMetricsTreeNode[] {
  const flat: OcrMetricsTreeNode[] = [];
  const visit = (items: OcrMetricsTreeNode[]) => {
    for (const node of items) {
      flat.push(node);
      if (node.children?.length) {
        visit(node.children);
      }
    }
  };
  visit(nodes);
  return flat;
}

function integerLabel(value: number | undefined) {
  return Math.round(value ?? 0).toLocaleString();
}

function tpsLabel(value: number | undefined) {
  return value && value > 0 ? value.toFixed(1) : '';
}

function durationLabel(ms: number | undefined) {
  if (!ms) {
    return '';
  }
  if (ms < 1000) {
    return `${ms} ms`;
  }
  return `${(ms / 1000).toFixed(1)} s`;
}
