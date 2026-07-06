import { Boxes, FileText, Folder } from 'lucide-react';

import type {
  DiagnosticEventRecord,
  DiagnosticSpanRecord,
  DiagnosticWorkUnitRecord,
  IngestRunRecord,
} from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { formatMs, iconForStatus } from './DiagnosticsPanel.helpers';
import { DiagnosticsRunActions, isActiveRunStatus } from './DiagnosticsRunActions';

interface WaterfallRunNodesArgs {
  activeRunId?: string | null;
  events: DiagnosticEventRecord[];
  runs: IngestRunRecord[];
  spans: DiagnosticSpanRecord[];
  workUnits: DiagnosticWorkUnitRecord[];
  onResumeRun?: (runId: string) => void;
  onRestartRun?: (run: IngestRunRecord) => void;
  onStopRun?: (runId?: string) => void;
}

interface RunBucket {
  run?: IngestRunRecord;
  runId: string;
  root: FolderBucket;
}

interface FolderBucket {
  files: Map<string, FileBucket>;
  folders: Map<string, FolderBucket>;
  id: string;
  label: string;
}

interface FileBucket {
  details: TreeGridNode[];
  id: string;
  label: string;
  pages: Map<number, TreeGridNode[]>;
}

interface RecordLocation {
  fileKey: string;
  fileLabel: string;
  pageNo?: number | null;
  pathParts: string[];
}

export function buildWaterfallRunNodes(args: WaterfallRunNodesArgs): TreeGridNode[] {
  const knownRuns = new Map(args.runs.map((run) => [run.run_id, run]));
  const runBuckets = new Map<string, RunBucket>();
  const runBucket = (runId: string): RunBucket => {
    const existing = runBuckets.get(runId);
    if (existing) {
      return existing;
    }
    const bucket = {
      root: folderBucket(`run:${runId}:root`, ''),
      run: knownRuns.get(runId),
      runId,
    };
    runBuckets.set(runId, bucket);
    return bucket;
  };

  for (const run of args.runs) {
    runBucket(run.run_id);
  }
  for (const unit of args.workUnits) {
    addRecordNode(runBucket(unit.run_id), unitLocation(unit), workUnitNode(unit));
  }
  for (const span of args.spans) {
    addRecordNode(runBucket(span.run_id ?? 'unscoped'), diagnosticLocation(span), spanNode(span));
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

function addRecordNode(bucket: RunBucket, location: RecordLocation, node: TreeGridNode) {
  const parent = fileBucket(bucket.root, location.pathParts, location.fileKey, location.fileLabel);
  if (location.pageNo && location.pageNo > 0) {
    parent.pages.set(location.pageNo, [...(parent.pages.get(location.pageNo) ?? []), node]);
    return;
  }
  parent.details.push(node);
}

function unitLocation(unit: DiagnosticWorkUnitRecord): RecordLocation {
  const path = metadataString(unit.metadata, 'relative_path') ?? unit.source_path ?? unit.filename;
  return recordLocation(path, unit.file_hash, unit.filename, unit.page_no);
}

function diagnosticLocation(
  record: Pick<
    DiagnosticSpanRecord | DiagnosticEventRecord,
    'attributes' | 'file_hash' | 'page_no'
  >,
): RecordLocation {
  const path =
    metadataString(record.attributes, 'relative_path') ??
    metadataString(record.attributes, 'source_path') ??
    metadataString(record.attributes, 'filename');
  return recordLocation(
    path,
    record.file_hash,
    metadataString(record.attributes, 'filename'),
    record.page_no,
  );
}

function recordLocation(
  path: string | null | undefined,
  fileHash: string | null | undefined,
  filename: string | null | undefined,
  pageNo: number | null | undefined,
): RecordLocation {
  const parts = splitPath(path ?? filename ?? fileHash ?? 'unknown');
  const fileLabel = filename ?? parts.at(-1) ?? fileHash ?? 'unknown';
  return {
    fileKey: fileHash ?? parts.join('/'),
    fileLabel,
    pageNo,
    pathParts: parts.length > 1 ? parts.slice(0, -1) : [],
  };
}

function fileBucket(
  root: FolderBucket,
  pathParts: string[],
  fileKey: string,
  fileLabel: string,
): FileBucket {
  let folder = root;
  for (const part of pathParts) {
    const id = `${folder.id}/folder:${part}`;
    const existing = folder.folders.get(part) ?? folderBucket(id, part);
    folder.folders.set(part, existing);
    folder = existing;
  }
  const existing = folder.files.get(fileKey);
  if (existing) {
    return existing;
  }
  const next = {
    details: [],
    id: `${folder.id}/file:${fileKey}`,
    label: fileLabel,
    pages: new Map<number, TreeGridNode[]>(),
  };
  folder.files.set(fileKey, next);
  return next;
}

function folderBucket(id: string, label: string): FolderBucket {
  return { files: new Map(), folders: new Map(), id, label };
}

function folderChildren(folder: FolderBucket): TreeGridNode[] {
  const folders = [...folder.folders.values()].sort(labelSort).map((child) => ({
    badge: <span>{child.files.size + child.folders.size}</span>,
    children: folderChildren(child),
    icon: <Folder size={14} />,
    id: child.id,
    label: child.label,
  }));
  return [...folders, ...[...folder.files.values()].sort(labelSort).map(fileNode)];
}

function fileNode(file: FileBucket): TreeGridNode {
  const pages = [...file.pages.entries()]
    .sort(([left], [right]) => left - right)
    .map(([pageNo, details]) => ({
      badge: <span>{details.length}</span>,
      children: details,
      icon: <FileText size={14} />,
      id: `${file.id}/page:${pageNo}`,
      label: `page ${pageNo}`,
    }));
  return {
    badge: <span>{file.details.length + pages.length}</span>,
    children: [...pages, ...file.details],
    icon: <FileText size={14} />,
    id: file.id,
    label: file.label,
  };
}

function workUnitNode(unit: DiagnosticWorkUnitRecord): TreeGridNode {
  return {
    badge: <span>{unit.status}</span>,
    icon: iconForStatus(unit.status),
    id: `work:${unit.work_unit_id}`,
    label: `${unit.phase} - ${unit.engine || unit.provider || unit.model}`,
  };
}

function spanNode(span: DiagnosticSpanRecord): TreeGridNode {
  return {
    badge: <span>{formatMs(span.duration_ms)}</span>,
    icon: iconForStatus(span.status),
    id: `span:${span.span_id}`,
    label: `${span.name}${span.page_no ? ` page ${span.page_no}` : ''}`,
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

function splitPath(value: string) {
  return value
    .split(/[\\/]+/)
    .map((part) => part.trim())
    .filter(Boolean);
}

function metadataString(metadata: Record<string, unknown>, key: string) {
  const value = metadata[key];
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function labelSort<T extends { label: string }>(left: T, right: T) {
  return left.label.localeCompare(right.label);
}

function shortId(value: string) {
  return value.length > 8 ? value.slice(0, 8) : value;
}
