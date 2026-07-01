import { CircleAlert, CircleCheck, FileText, LoaderCircle } from 'lucide-react';

import type { IngestRunRecord, LogRecord } from '../../api/types';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import styles from './DiagnosticsPanel.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

interface DiagnosticsPanelProps {
  logs: LogRecord[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}

export function DiagnosticsPanel({ logs, onSearchChange, runs, search }: DiagnosticsPanelProps) {
  const tab = search?.tab ?? 'runs';
  const filteredLogs = filterLogs(logs, search);
  const filteredRuns = filterRuns(runs, search);
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
          data-active={tab === 'logs'}
          onClick={() => onSearchChange?.({ tab: 'logs' })}
          type="button"
        >
          Logs
        </button>
      </div>
      <DiagnosticsFilters
        logs={logs}
        runs={runs}
        search={search}
        tab={tab}
        onSearchChange={onSearchChange}
      />
      <div className={styles.body}>
        {tab === 'runs' ? <RunList runs={filteredRuns} /> : <LogList logs={filteredLogs} />}
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
  onSearchChange,
  runs,
  search,
  tab,
}: {
  logs: LogRecord[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  tab: 'logs' | 'runs';
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
