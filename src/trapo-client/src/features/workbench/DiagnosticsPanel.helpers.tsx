import { Boxes, CircleAlert, CircleCheck, Clock3, LoaderCircle } from 'lucide-react';

import type {
  DiagnosticWaterfallRowRecord,
  DiagnosticWorkUnitRecord,
  IngestRunRecord,
  LogRecord,
} from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import styles from './DiagnosticsPanel.module.css';

export function buildProgressNodes(
  workUnits: DiagnosticWorkUnitRecord[],
  onWorkUnitSelect?: (workUnitId: string) => void,
): TreeGridNode[] {
  const byRun = new Map<string, DiagnosticWorkUnitRecord[]>();
  for (const unit of workUnits) {
    byRun.set(unit.run_id, [...(byRun.get(unit.run_id) ?? []), unit]);
  }
  return [...byRun.entries()].map(([runId, units]) => ({
    badge: <span>{units.length}</span>,
    children: units.map((unit) => ({
      badge: <span>{unit.status}</span>,
      icon: iconForStatus(unit.status),
      id: unit.work_unit_id,
      label: `${unit.phase} ${unit.page_no ? `page ${unit.page_no}` : (unit.file_hash ?? '')}`,
      onSelect: () => onWorkUnitSelect?.(unit.work_unit_id),
    })),
    icon: <Boxes size={14} />,
    id: `run:${runId}`,
    label: runId,
  }));
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
    return query
      ? `${log.timestamp} ${log.level} ${log.component} ${log.message}`
          .toLowerCase()
          .includes(query)
      : true;
  });
}

export function filterRuns(runs: IngestRunRecord[], search: DiagnosticsRouteSearch | undefined) {
  const query = normalized(search?.q);
  return runs.filter((run) => {
    if (search?.run && run.run_id !== search.run) {
      return false;
    }
    if (search?.status && search.status !== 'all' && run.status !== search.status) {
      return false;
    }
    return query
      ? `${run.run_id} ${run.status} ${run.root_path} ${run.profile_id ?? ''}`
          .toLowerCase()
          .includes(query)
      : true;
  });
}

export interface DiagnosticFailureSummaryItem {
  id: string;
  message: string;
  source: string;
  detail?: string;
}

export function buildFailureSummary(
  rows: DiagnosticWaterfallRowRecord[],
  workUnits: DiagnosticWorkUnitRecord[],
): DiagnosticFailureSummaryItem[] {
  const seen = new Set<string>();
  const failures: DiagnosticFailureSummaryItem[] = [];
  for (const row of rows) {
    if (!isFailureStatus(row.status)) {
      continue;
    }
    const message = row.error_message || row.status_message;
    if (!message || seen.has(message)) {
      continue;
    }
    seen.add(message);
    failures.push({
      id: row.span_id ?? row.work_unit_id ?? `${row.label}-${failures.length}`,
      message,
      source: row.pipeline_step || row.category || 'diagnostic',
      detail: row.label,
    });
  }
  for (const unit of workUnits) {
    if (!isFailureStatus(unit.status) || !unit.error || seen.has(unit.error)) {
      continue;
    }
    seen.add(unit.error);
    failures.push({
      id: unit.work_unit_id,
      message: unit.error,
      source: unit.phase,
      detail: unit.filename ?? unit.file_hash ?? undefined,
    });
  }
  return failures.slice(0, 5);
}

export function iconForStatus(status: string) {
  if (status === 'completed' || status === 'ok') {
    return <CircleCheck size={14} className={styles.ok} />;
  }
  if (status === 'failed' || status === 'error' || status === 'cancelled') {
    return <CircleAlert size={14} className={styles.bad} />;
  }
  if (status === 'running') {
    return <LoaderCircle size={14} className={styles.pending} />;
  }
  return <Clock3 size={14} className={styles.queued} />;
}

function isFailureStatus(status: string) {
  return status === 'failed' || status === 'error' || status === 'cancelled';
}

export function toggled(current: Set<string>, id: string) {
  const next = new Set(current);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  return next;
}

export function formatMs(value: number) {
  if (value > 0 && value < 1) {
    return '<1ms';
  }
  return value >= 1000 ? `${(value / 1000).toFixed(2)}s` : `${value.toFixed(0)}ms`;
}

function normalized(value: string | undefined) {
  return value?.trim().toLowerCase() ?? '';
}
