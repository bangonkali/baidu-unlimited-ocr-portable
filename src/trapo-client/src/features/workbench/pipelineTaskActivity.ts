import type { DiagnosticPipelineTaskRecord } from '../../api/types';

const ACTIVE_TASK_STATUSES = new Set(['planned', 'queued', 'running']);
const RAG_TASK_KINDS = new Set(['text_index', 'generate_embedding']);

export interface PipelineTaskActivity {
  kind: string;
  label: string;
  runId?: string;
  status: string;
  task: DiagnosticPipelineTaskRecord;
  title: string;
}

export function activePipelineActivities(
  tasks: DiagnosticPipelineTaskRecord[] | undefined,
): PipelineTaskActivity[] {
  return (tasks ?? [])
    .filter((task) => RAG_TASK_KINDS.has(task.task_kind))
    .filter((task) => ACTIVE_TASK_STATUSES.has(task.status))
    .map(taskActivity)
    .sort(activitySort);
}

export function primaryPipelineActivity(
  tasks: DiagnosticPipelineTaskRecord[] | undefined,
): PipelineTaskActivity | undefined {
  return activePipelineActivities(tasks)[0];
}

export function activePipelineActivityForRun(
  tasks: DiagnosticPipelineTaskRecord[] | undefined,
  runId: string | undefined,
): PipelineTaskActivity | undefined {
  if (!runId) {
    return undefined;
  }
  return activePipelineActivities(tasks).find((activity) => activity.runId === runId);
}

function taskActivity(task: DiagnosticPipelineTaskRecord): PipelineTaskActivity {
  const label = taskLabel(task.task_kind);
  const modelId = stringParam(task.params, 'model_id');
  const status = task.status;
  return {
    kind: task.task_kind,
    label,
    runId: task.origin_run_id ?? stringParam(task.params, 'source_run_id'),
    status,
    task,
    title: modelId ? `${label} ${status} with ${modelId}` : `${label} ${status}`,
  };
}

function taskLabel(taskKind: string) {
  if (taskKind === 'text_index') {
    return 'Text Index';
  }
  if (taskKind === 'generate_embedding') {
    return 'Embedding';
  }
  return taskKind.replaceAll('_', ' ');
}

function stringParam(params: Record<string, unknown>, key: string) {
  const value = params[key];
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function activitySort(left: PipelineTaskActivity, right: PipelineTaskActivity) {
  const statusOrder = statusRank(left.status) - statusRank(right.status);
  if (statusOrder !== 0) {
    return statusOrder;
  }
  return Date.parse(left.task.queued_at) - Date.parse(right.task.queued_at);
}

function statusRank(status: string) {
  if (status === 'running') {
    return 0;
  }
  if (status === 'queued') {
    return 1;
  }
  return 2;
}
