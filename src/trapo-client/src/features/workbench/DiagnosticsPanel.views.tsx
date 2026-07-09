import { Activity, Clipboard, FileText } from 'lucide-react';
import { useState } from 'react';

import type { useDiagnosticAnalytics } from '../../api/hooks';
import { getText } from '../../api/http';
import type { DiagnosticModelLeaseRecord, LogRecord } from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { TreeGrid } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import logStyles from './DiagnosticsLogs.module.css';
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
  component: string;
  components: string[];
  level: string;
  levels: string[];
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
        aria-label="Log level"
        onChange={(event) => props.onChange?.({ level: event.target.value || undefined })}
        value={props.level}
      >
        <option value="">All levels</option>
        {props.levels.map((level) => (
          <option key={level} value={level}>
            {level}
          </option>
        ))}
      </select>
      <select
        aria-label="Log component"
        onChange={(event) => props.onChange?.({ component: event.target.value || undefined })}
        value={props.component}
      >
        <option value="">All components</option>
        {props.components.map((component) => (
          <option key={component} value={component}>
            {component}
          </option>
        ))}
      </select>
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
  const [copyStatus, setCopyStatus] = useState('');
  const copyLog = (value: string, status: string) => {
    void copyToClipboard(value).then(() => {
      setCopyStatus(status);
      window.setTimeout(() => setCopyStatus(''), 1400);
    });
  };
  const copyAll = () => {
    void getText('/api/logs/export').then((text) => copyLog(text, 'Copied all logs'));
  };
  return (
    <div className={logStyles.logStack}>
      <div className={logStyles.logToolbar}>
        <button className={logStyles.logCopyButton} onClick={copyAll} type="button">
          <Clipboard size={13} />
          Copy All
        </button>
        {copyStatus ? <span>{copyStatus}</span> : null}
      </div>
      <div className={logStyles.logHeader}>
        <span>Time</span>
        <span>Level</span>
        <span>Component</span>
        <span>Message</span>
      </div>
      <div className={styles.list}>
        {logs.length === 0 ? <div className={styles.empty}>No logs</div> : null}
        {logs.map((log) => (
          <button
            className={logStyles.logRow}
            data-level={log.level}
            key={`${log.timestamp}-${log.message}`}
            onClick={() => copyLog(formatLogLine(log), 'Copied row')}
            title="Copy log row"
            type="button"
          >
            <FileText size={14} />
            <time>{formatLogTime(log.timestamp)}</time>
            <strong>{log.level}</strong>
            <small>{log.component}</small>
            <span>{log.message}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

function formatLogLine(log: LogRecord) {
  return `${log.timestamp} ${log.level} ${log.component} ${log.message}`;
}

function formatLogTime(value: string) {
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return value || 'unknown';
  }
  return new Intl.DateTimeFormat(undefined, {
    day: '2-digit',
    hour: '2-digit',
    hour12: false,
    minute: '2-digit',
    month: '2-digit',
    second: '2-digit',
  }).format(parsed);
}

async function copyToClipboard(value: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(value);
    return;
  }
  const textArea = document.createElement('textarea');
  textArea.value = value;
  textArea.style.position = 'fixed';
  textArea.style.left = '-9999px';
  document.body.append(textArea);
  textArea.select();
  document.execCommand('copy');
  textArea.remove();
}
