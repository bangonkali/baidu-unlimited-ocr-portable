import { Boxes } from 'lucide-react';

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
  taskDurationMs,
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

export function buildWaterfallRunNodes(args: WaterfallRunNodesArgs): TreeGridNode[] {
  const { runBuckets, runBucket } = createRunBucketResolver(args.runs);
  const maxDurationMs = maxDuration(args);

  for (const run of args.runs) {
    runBucket(run.run_id);
  }
  for (const unit of args.workUnits) {
    addRecordNode(runBucket(unit.run_id), unitLocation(unit), workUnitNode(unit, maxDurationMs));
  }
  for (const task of args.pipelineTasks) {
    addRecordNode(
      runBucket(task.origin_run_id ?? 'unscoped'),
      pipelineTaskLocation(task),
      pipelineTaskNode(task, maxDurationMs),
    );
  }
  for (const span of args.spans) {
    addRecordNode(
      runBucket(span.run_id ?? 'unscoped'),
      diagnosticLocation(span),
      spanNode(span, maxDurationMs),
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

function workUnitNode(unit: DiagnosticWorkUnitRecord, maxDurationMs: number): TreeGridNode {
  return {
    actions: waterfallBar(unit.duration_ms ?? 0, maxDurationMs, unit.status),
    badge: <span>{unit.status}</span>,
    icon: iconForStatus(unit.status),
    id: `work:${unit.work_unit_id}`,
    label: `${unit.phase} - ${unit.engine || unit.provider || unit.model}`,
  };
}

function spanNode(span: DiagnosticSpanRecord, maxDurationMs: number): TreeGridNode {
  return {
    actions: waterfallBar(span.duration_ms, maxDurationMs, span.status),
    badge: <span>{formatMs(span.duration_ms)}</span>,
    icon: iconForStatus(span.status),
    id: `span:${span.span_id}`,
    label: `${span.name}${span.page_no ? ` page ${span.page_no}` : ''}`,
  };
}

function pipelineTaskNode(task: DiagnosticPipelineTaskRecord, maxDurationMs: number): TreeGridNode {
  const duration = taskDurationMs(task);
  return {
    actions: waterfallBar(duration, maxDurationMs, task.status),
    badge: <span>{duration ? formatMs(duration) : task.status}</span>,
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

function maxDuration(args: WaterfallRunNodesArgs) {
  const durations = [
    ...args.workUnits.map((unit) => unit.duration_ms ?? 0),
    ...args.spans.map((span) => span.duration_ms),
    ...args.pipelineTasks.map(taskDurationMs),
  ];
  return Math.max(...durations, 1);
}

function waterfallBar(durationMs: number, maxDurationMs: number, status: string) {
  const width = durationMs > 0 ? Math.max(3, (durationMs / maxDurationMs) * 100) : 3;
  return (
    <span
      className={styles.diagnosticWaterfallBar}
      data-status={status}
      title={formatMs(durationMs)}
    >
      <span style={{ width: `${Math.min(width, 100)}%` }} />
    </span>
  );
}
