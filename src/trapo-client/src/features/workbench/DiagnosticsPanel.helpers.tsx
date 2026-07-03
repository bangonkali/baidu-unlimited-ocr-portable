import { Boxes, CircleAlert, CircleCheck, Clock3, LoaderCircle } from 'lucide-react';

import type {
  DiagnosticSpanRecord,
  DiagnosticWorkUnitRecord,
  IngestRunRecord,
  LogRecord,
} from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import styles from './DiagnosticsPanel.module.css';

export function buildSpanNodes(spans: DiagnosticSpanRecord[]): TreeGridNode[] {
  const children = new Map<string | null, DiagnosticSpanRecord[]>();
  for (const span of spans) {
    const parent = span.parent_span_id ?? null;
    children.set(parent, [...(children.get(parent) ?? []), span]);
  }
  const toNode = (span: DiagnosticSpanRecord): TreeGridNode => ({
    badge: <span>{formatMs(span.duration_ms)}</span>,
    children: (children.get(span.span_id) ?? []).map(toNode),
    icon: iconForStatus(span.status),
    id: span.span_id,
    label: spanLabel(span),
  });
  return (children.get(null) ?? spans.filter((span) => !span.parent_span_id)).map(toNode);
}

export function buildProgressNodes(workUnits: DiagnosticWorkUnitRecord[]): TreeGridNode[] {
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
    if (search?.status && run.status !== search.status) {
      return false;
    }
    return query
      ? `${run.run_id} ${run.status} ${run.root_path} ${run.profile_id ?? ''}`
          .toLowerCase()
          .includes(query)
      : true;
  });
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
  return value >= 1000 ? `${(value / 1000).toFixed(2)}s` : `${value.toFixed(0)}ms`;
}

function spanLabel(span: DiagnosticSpanRecord) {
  const page = span.page_no ? ` page ${span.page_no}` : '';
  return `${span.name}${page}`;
}

function normalized(value: string | undefined) {
  return value?.trim().toLowerCase() ?? '';
}
