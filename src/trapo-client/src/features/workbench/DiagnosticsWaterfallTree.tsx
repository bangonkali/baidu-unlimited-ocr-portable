import { Activity, Boxes, FileText, Folder } from 'lucide-react';
import type { CSSProperties } from 'react';

import type { DiagnosticWaterfallPayload, DiagnosticWaterfallRowRecord } from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { formatMs, iconForStatus } from './DiagnosticsPanel.helpers';
import styles from './DiagnosticsWaterfallBars.module.css';

interface WaterfallTraceNodesArgs {
  payload?: DiagnosticWaterfallPayload;
}

interface WaterfallTimeline {
  rangeMs: number;
  startMs: number;
}

export function buildWaterfallRunNodes(args: WaterfallTraceNodesArgs): TreeGridNode[] {
  const rows = args.payload?.rows ?? [];
  if (rows.length === 0) {
    return [];
  }
  const timeline = waterfallTimeline(args.payload);
  const byParent = new Map<string, DiagnosticWaterfallRowRecord[]>();
  const roots: DiagnosticWaterfallRowRecord[] = [];
  for (const row of rows) {
    if (row.parent_row_id) {
      byParent.set(row.parent_row_id, [...(byParent.get(row.parent_row_id) ?? []), row]);
    } else {
      roots.push(row);
    }
  }
  const toNode = (row: DiagnosticWaterfallRowRecord): TreeGridNode => ({
    actions: waterfallBar(row, timeline),
    badge: <span>{formatTimestamp(row.visual_start_ms ?? row.start_ms)}</span>,
    children: sortRows(byParent.get(row.row_id) ?? []).map(toNode),
    icon: iconForRow(row),
    id: row.row_id,
    label: rowLabel(row),
  });
  return sortRows(roots).map(toNode);
}

export function waterfallExpandableIds(nodes: TreeGridNode[]) {
  const ids = new Set<string>();
  const visit = (node: TreeGridNode) => {
    if ((node.children?.length ?? 0) > 0) {
      ids.add(node.id);
      node.children?.forEach(visit);
    }
  };
  nodes.forEach(visit);
  return ids;
}

function sortRows(rows: DiagnosticWaterfallRowRecord[]) {
  return [...rows].sort((left, right) => left.sort_index - right.sort_index);
}

function rowLabel(row: DiagnosticWaterfallRowRecord) {
  if (
    row.row_source === 'run' ||
    row.row_source === 'task_group' ||
    row.row_source === 'file_group'
  ) {
    return sourceLabel(row);
  }
  const parts = [sourceLabel(row)];
  if (row.filename) {
    parts.push(row.filename);
  } else if (row.file_hash) {
    parts.push(shortId(row.file_hash));
  }
  if (row.page_no) {
    parts.push(`p${row.page_no}`);
  }
  return parts.join(' - ');
}

function sourceLabel(row: DiagnosticWaterfallRowRecord) {
  if (row.row_source === 'run') {
    return row.label;
  }
  if (row.row_source === 'task_group') {
    return `Task: ${row.label}`;
  }
  if (row.row_source === 'file_group') {
    return row.label;
  }
  if (row.row_source === 'pipeline_task') {
    return `Task: ${row.label}`;
  }
  if (row.row_source === 'work_unit') {
    return `Work unit: ${row.label}`;
  }
  return row.label;
}

function formatTimestamp(value: number | null | undefined) {
  if (value === undefined || value === null) {
    return '-';
  }
  const date = new Date(value);
  return `${date.getUTCFullYear()}-${padTime(date.getUTCMonth() + 1)}-${padTime(
    date.getUTCDate(),
  )} ${padTime(date.getUTCHours())}:${padTime(date.getUTCMinutes())}:${padTime(
    date.getUTCSeconds(),
  )}.${String(date.getUTCMilliseconds()).padStart(3, '0')}`;
}

function padTime(value: number) {
  return String(value).padStart(2, '0');
}

function iconForRow(row: DiagnosticWaterfallRowRecord) {
  if (row.row_source === 'run' || row.row_source === 'task_group') {
    return <Boxes size={14} />;
  }
  if (row.row_source === 'file_group') {
    return <Folder size={14} />;
  }
  if (row.span_kind === 'task') {
    return <Boxes size={14} />;
  }
  if (row.page_no) {
    return <FileText size={14} />;
  }
  return row.status ? iconForStatus(row.status) : <Activity size={14} />;
}

function waterfallTimeline(payload: DiagnosticWaterfallPayload | undefined): WaterfallTimeline {
  const startMs = payload?.summary.start_ms ?? 0;
  const endMs = payload?.summary.end_ms ?? startMs + 1;
  return { rangeMs: Math.max(endMs - startMs, 1), startMs };
}

function waterfallBar(row: DiagnosticWaterfallRowRecord, timeline: WaterfallTimeline) {
  const envelope = barStyle(row.visual_start_ms, row.visual_end_ms, timeline);
  const operation = barStyle(row.start_ms, row.end_ms, timeline);
  return (
    <span
      className={styles.diagnosticWaterfallBar}
      data-parent={row.child_count > 0}
      data-status={row.status}
      title={`${row.label} ${formatMs(row.visual_duration_ms || row.duration_ms)}`}
    >
      <span className={styles.waterfallLane}>
        <span className={styles.waterfallEnvelope} style={envelope} />
        <span className={styles.waterfallSegment} style={operation ?? envelope} />
      </span>
      <span className={styles.waterfallDuration}>
        {formatMs(row.visual_duration_ms || row.duration_ms)}
      </span>
    </span>
  );
}

function barStyle(
  startMs: number | null | undefined,
  endMs: number | null | undefined,
  timeline: WaterfallTimeline,
): CSSProperties | undefined {
  if (startMs === undefined || startMs === null || endMs === undefined || endMs === null) {
    return undefined;
  }
  const left = clamp(((startMs - timeline.startMs) / timeline.rangeMs) * 100, 0, 99);
  const rawWidth = ((endMs - startMs) / timeline.rangeMs) * 100;
  const width = clamp(Math.max(1.4, rawWidth), 1.4, 100 - left);
  return {
    left: `${left}%`,
    width: `${width}%`,
  };
}

function shortId(value: string) {
  return value.length > 8 ? value.slice(0, 8) : value;
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(Math.max(value, minimum), maximum);
}
