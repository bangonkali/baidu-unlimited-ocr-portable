import type { IngestRunRecord } from '../../api/types';
import type { WorkbenchPageProps } from './useWorkbenchPageController';

export function activeRunIdFromRuns(runs: IngestRunRecord[] | undefined) {
  return runs?.find((run) => ['queued', 'running'].includes(String(run.status)))?.run_id ?? null;
}

export function isActiveDocumentStatus(status: string) {
  return status === 'queued' || status === 'rendering' || status === 'running';
}

export function routeSearchText(props: WorkbenchPageProps) {
  return props.workbenchSearch?.q ?? props.diagnosticsSearch?.q ?? '';
}
