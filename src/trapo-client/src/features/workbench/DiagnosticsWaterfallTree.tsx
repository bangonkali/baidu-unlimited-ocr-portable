import { Boxes } from 'lucide-react';
import type { CSSProperties } from 'react';

import type {
  DiagnosticEventRecord,
  DiagnosticPipelineTaskRecord,
  DiagnosticSpanRecord,
  DiagnosticWorkUnitRecord,
  IngestRunRecord,
} from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { formatMs, iconForStatus } from './DiagnosticsPanel.helpers';
import styles from './DiagnosticsPanel.module.css';
import { DiagnosticsRunActions, isActiveRunStatus } from './DiagnosticsRunActions';
import type { RunBucket } from './DiagnosticsWaterfallBuckets';
import {
  addRecordNode,
  createRunBucketResolver,
  diagnosticLocation,
  folderChildren,
  pipelineTaskLocation,
  shortId,
  unitLocation,
} from './DiagnosticsWaterfallBuckets';

interface WaterfallRunNodesArgs {
  activeRunId?: string | null;
  events: DiagnosticEventRecord[];
  runs: IngestRunRecord[];
  spans: DiagnosticSpanRecord[];
  pipelineTasks: DiagnosticPipelineTaskRecord[];
  workUnits: DiagnosticWorkUnitRecord[];
  onResumeRun?: (runId: string) => void;
  onRestartRun?: (run: IngestRunRecord) => void;
  onStopRun?: (runId?: string) => void;
}

interface WaterfallTimeline {
  endMs: number;
  rangeMs: number;
  startMs: number;
}

interface WaterfallTiming {
  anchored: boolean;
  durationMs: number;
  endMs: number;
  startMs: number;
}

export function buildWaterfallRunNodes(args: WaterfallRunNodesArgs): TreeGridNode[] {
  const { runBuckets, runBucket } = createRunBucketResolver(args.runs);
  const nowMs = Date.now();
  const timeline = waterfallTimeline(args, nowMs);

  for (const run of args.runs) {
    runBucket(run.run_id);
  }
  for (const unit of args.workUnits) {
    addRecordNode(runBucket(unit.run_id), unitLocation(unit), workUnitNode(unit, timeline, nowMs));
  }
  for (const task of args.pipelineTasks) {
    addRecordNode(
      runBucket(task.origin_run_id ?? 'unscoped'),
      pipelineTaskLocation(task),
      pipelineTaskNode(task, timeline, nowMs),
    );
  }
  for (const span of args.spans) {
    addRecordNode(
      runBucket(span.run_id ?? 'unscoped'),
      diagnosticLocation(span),
      spanNode(span, timeline, nowMs),
    );
  }
  for (const event of args.events) {
    addRecordNode(
      runBucket(event.run_id ?? 'unscoped'),
      diagnosticLocation(event),
      eventNode(event),
    );
  }

  const hasActiveRun =
    Boolean(args.activeRunId) || args.runs.some((run) => isActiveRunStatus(run.status));
  return [...runBuckets.values()].map((bucket) => runNode(bucket, args, hasActiveRun));
}

function runNode(
  bucket: RunBucket,
  args: WaterfallRunNodesArgs,
  hasActiveRun: boolean,
): TreeGridNode {
  const run = bucket.run;
  return {
    actions: run ? (
      <DiagnosticsRunActions
        activeRunId={args.activeRunId}
        hasActiveRun={hasActiveRun}
        onResumeRun={args.onResumeRun}
        onRestartRun={args.onRestartRun}
        onStopRun={args.onStopRun}
        run={run}
      />
    ) : undefined,
    badge: <span>{run?.status ?? 'unknown'}</span>,
    children: folderChildren(bucket.root),
    icon: <Boxes size={14} />,
    id: `run:${bucket.runId}`,
    label: run ? `run ${shortId(run.run_id)} - ${run.root_path}` : `run ${bucket.runId}`,
  };
}

function workUnitNode(
  unit: DiagnosticWorkUnitRecord,
  timeline: WaterfallTimeline,
  nowMs: number,
): TreeGridNode {
  const timing = workUnitTiming(unit, nowMs);
  return {
    actions: waterfallBar(timing, timeline, unit.status),
    badge: <span>{formatMs(timing.durationMs)}</span>,
    icon: iconForStatus(unit.status),
    id: `work:${unit.work_unit_id}`,
    label: `${unit.phase} - ${unit.engine || unit.provider || unit.model}`,
  };
}

function spanNode(
  span: DiagnosticSpanRecord,
  timeline: WaterfallTimeline,
  nowMs: number,
): TreeGridNode {
  const timing = spanTiming(span, nowMs);
  return {
    actions: waterfallBar(timing, timeline, span.status),
    badge: <span>{formatMs(timing.durationMs)}</span>,
    icon: iconForStatus(span.status),
    id: `span:${span.span_id}`,
    label: `${span.name}${span.page_no ? ` page ${span.page_no}` : ''}`,
  };
}

function pipelineTaskNode(
  task: DiagnosticPipelineTaskRecord,
  timeline: WaterfallTimeline,
  nowMs: number,
): TreeGridNode {
  const timing = pipelineTaskTiming(task, nowMs);
  return {
    actions: waterfallBar(timing, timeline, task.status),
    badge: <span>{timing.durationMs ? formatMs(timing.durationMs) : task.status}</span>,
    icon: iconForStatus(task.status),
    id: `task:${task.task_id}`,
    label: `${task.task_kind} - ${task.status}`,
  };
}

function eventNode(event: DiagnosticEventRecord): TreeGridNode {
  return {
    badge: <span>{event.severity}</span>,
    icon: iconForStatus(statusFromSeverity(event.severity)),
    id: `event:${event.event_id}`,
    label: `${event.name} - ${event.message}`,
  };
}

function statusFromSeverity(severity: string) {
  return severity === 'error' || severity === 'fatal' ? 'error' : 'ok';
}

function waterfallTimeline(args: WaterfallRunNodesArgs, nowMs: number): WaterfallTimeline {
  const timings = [
    ...args.workUnits.map((unit) => workUnitTiming(unit, nowMs)),
    ...args.spans.map((span) => spanTiming(span, nowMs)),
    ...args.pipelineTasks.map((task) => pipelineTaskTiming(task, nowMs)),
  ];
  const anchored = timings.filter((timing) => timing.anchored);
  if (anchored.length === 0) {
    const maxDuration = Math.max(...timings.map((timing) => timing.durationMs), 1);
    return { endMs: maxDuration, rangeMs: maxDuration, startMs: 0 };
  }
  const startMs = Math.min(...anchored.map((timing) => timing.startMs));
  const endMs = Math.max(...anchored.map((timing) => timing.endMs));
  return { endMs, rangeMs: Math.max(endMs - startMs, 1), startMs };
}

function workUnitTiming(unit: DiagnosticWorkUnitRecord, nowMs: number): WaterfallTiming {
  return timingFromDates(
    unit.started_at,
    unit.finished_at,
    unit.status,
    unit.duration_ms ?? 0,
    nowMs,
  );
}

function spanTiming(span: DiagnosticSpanRecord, nowMs: number): WaterfallTiming {
  return timingFromDates(span.started_at, span.ended_at, span.status, span.duration_ms, nowMs);
}

function pipelineTaskTiming(task: DiagnosticPipelineTaskRecord, nowMs: number): WaterfallTiming {
  return timingFromDates(
    task.started_at ?? task.queued_at,
    task.finished_at,
    task.status,
    0,
    nowMs,
  );
}

function timingFromDates(
  startValue: string | null | undefined,
  endValue: string | null | undefined,
  status: string,
  fallbackDurationMs: number,
  nowMs: number,
): WaterfallTiming {
  let startMs = parseTimestamp(startValue);
  let endMs = parseTimestamp(endValue);
  if (startMs !== undefined && endMs === undefined && isInProgressStatus(status)) {
    endMs = nowMs;
  }
  if (startMs === undefined && endMs !== undefined) {
    startMs = Math.max(0, endMs - fallbackDurationMs);
  }
  if (startMs !== undefined && endMs === undefined) {
    endMs = startMs + fallbackDurationMs;
  }
  if (startMs !== undefined && endMs !== undefined && endMs >= startMs) {
    return {
      anchored: true,
      durationMs: endMs - startMs,
      endMs,
      startMs,
    };
  }
  return {
    anchored: false,
    durationMs: Math.max(fallbackDurationMs, 0),
    endMs: Math.max(fallbackDurationMs, 0),
    startMs: 0,
  };
}

function parseTimestamp(value: string | null | undefined) {
  if (!value) {
    return undefined;
  }
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function isInProgressStatus(status: string) {
  return status === 'running' || status === 'queued' || status === 'planned';
}

function waterfallBar(timing: WaterfallTiming, timeline: WaterfallTimeline, status: string) {
  const left = timing.anchored
    ? clamp(((timing.startMs - timeline.startMs) / timeline.rangeMs) * 100, 0, 99)
    : 0;
  const rawWidth = timing.durationMs > 0 ? (timing.durationMs / timeline.rangeMs) * 100 : 1.4;
  const width = clamp(Math.max(1.4, rawWidth), 1.4, 100 - left);
  const style = {
    left: `${left}%`,
    width: `${width}%`,
  } satisfies CSSProperties;
  return (
    <span
      className={styles.diagnosticWaterfallBar}
      data-status={status}
      title={formatMs(timing.durationMs)}
    >
      <span className={styles.waterfallLane}>
        <span className={styles.waterfallSegment} style={style} />
      </span>
      <span className={styles.waterfallDuration}>{formatMs(timing.durationMs)}</span>
    </span>
  );
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(Math.max(value, minimum), maximum);
}
