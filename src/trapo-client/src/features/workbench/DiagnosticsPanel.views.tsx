import { Activity, FileText } from 'lucide-react';

import type { useDiagnosticAnalytics } from '../../api/hooks';
import type { DiagnosticModelLeaseRecord, LogRecord } from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { TreeGrid } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import { formatMs, iconForStatus } from './DiagnosticsPanel.helpers';
import styles from './DiagnosticsPanel.module.css';

export function TabBar({
  active,
  onChange,
}: {
  active: NonNullable<DiagnosticsRouteSearch['tab']>;
  onChange: (tab: NonNullable<DiagnosticsRouteSearch['tab']>) => void;
}) {
  const tabs: Array<NonNullable<DiagnosticsRouteSearch['tab']>> = [
    'waterfall',
    'progress',
    'analytics',
    'models',
    'logs',
  ];
  return (
    <div className={styles.tabs}>
      {tabs.map((tab) => (
        <button
          className={styles.tab}
          data-active={active === tab}
          key={tab}
          onClick={() => onChange(tab)}
          type="button"
        >
          {tab}
        </button>
      ))}
    </div>
  );
}

export function DiagnosticsToolbar(props: {
  query: string;
  runId?: string;
  runs: string[];
  status: string;
  onChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}) {
  return (
    <div className={styles.filters}>
      <input
        aria-label="Filter diagnostics"
        onChange={(event) => props.onChange?.({ q: event.target.value || undefined })}
        placeholder="Filter spans, pages, logs"
        value={props.query}
      />
      <select
        aria-label="Run"
        onChange={(event) => props.onChange?.({ run: event.target.value || undefined })}
        value={props.runId ?? ''}
      >
        <option value="">Latest</option>
        {props.runs.map((runId) => (
          <option key={runId} value={runId}>
            {runId}
          </option>
        ))}
      </select>
      <select
        aria-label="Status"
        onChange={(event) => props.onChange?.({ status: event.target.value || undefined })}
        value={props.status}
      >
        <option value="all">All</option>
        <option value="ok">OK</option>
        <option value="failed">Failed</option>
        <option value="error">Error</option>
      </select>
    </div>
  );
}

export function ProgressSummary({
  nodes,
  summary,
}: {
  nodes: TreeGridNode[];
  summary?: {
    total_work_units: number;
    queued: number;
    running: number;
    completed: number;
    failed: number;
    cancelled: number;
  };
}) {
  return (
    <div className={styles.diagnosticStack}>
      <div className={styles.metrics}>
        {['total_work_units', 'queued', 'running', 'completed', 'failed', 'cancelled'].map(
          (key) => (
            <span key={key}>
              <strong>{summary?.[key as keyof typeof summary] ?? 0}</strong>
              {key.replaceAll('_', ' ')}
            </span>
          ),
        )}
      </div>
      <TreeGrid
        className={styles.tree}
        expandedIds={new Set(nodes.map((node) => node.id))}
        nodes={nodes}
        onToggle={() => undefined}
      />
    </div>
  );
}

export function AnalyticsView({
  data,
}: {
  data?: ReturnType<typeof useDiagnosticAnalytics>['data'];
}) {
  return (
    <div className={styles.diagnosticStack}>
      <div className={styles.metrics}>
        <span>
          <strong>{data?.summary.span_count ?? 0}</strong>spans
        </span>
        <span>
          <strong>{formatMs(data?.summary.total_duration_ms ?? 0)}</strong>duration
        </span>
        <span>
          <strong>{data?.summary.error_count ?? 0}</strong>errors
        </span>
      </div>
      <div className={styles.list}>
        {(data?.slow_spans ?? []).map((span) => (
          <div className={styles.compactRow} key={span.span_id}>
            <Activity size={14} />
            <span>{span.name}</span>
            <strong>{span.pipeline_step}</strong>
            <small>{formatMs(span.duration_ms)}</small>
          </div>
        ))}
      </div>
    </div>
  );
}

export function ModelLeaseList({ leases }: { leases: DiagnosticModelLeaseRecord[] }) {
  return (
    <div className={styles.list}>
      {leases.length === 0 ? <div className={styles.empty}>No model leases</div> : null}
      {leases.map((lease) => (
        <div className={styles.compactRow} key={lease.lease_id}>
          {iconForStatus(lease.status)}
          <span>{lease.model}</span>
          <strong>{lease.status}</strong>
          <small>{lease.provider}</small>
        </div>
      ))}
    </div>
  );
}

export function LogList({ logs }: { logs: LogRecord[] }) {
  return (
    <div className={styles.list}>
      {logs.length === 0 ? <div className={styles.empty}>No logs</div> : null}
      {logs.map((log) => (
        <div className={styles.compactRow} key={`${log.timestamp}-${log.message}`}>
          <FileText size={14} />
          <span>{log.message}</span>
          <strong>{log.level}</strong>
          <small>{log.component}</small>
        </div>
      ))}
    </div>
  );
}
