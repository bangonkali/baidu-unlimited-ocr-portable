import { CircleAlert, LoaderCircle } from 'lucide-react';

import type {
  DiagnosticWorkUnitRecord,
  DocumentSummary,
  IngestPreviewResultRecord,
} from '../../api/types';
import styles from './ExplorerTree.module.css';

export interface ExplorerEngineStatusContext {
  diagnosticWorkUnits?: DiagnosticWorkUnitRecord[];
  document: DocumentSummary;
  pageNo: number;
  previewResults?: IngestPreviewResultRecord[];
  runId?: string;
  selectedRunEngineId?: string;
}

export function engineBadge(
  result: IngestPreviewResultRecord,
  context: ExplorerEngineStatusContext,
) {
  const workUnit = enginePageWorkUnit(result, context);
  if (workUnit?.status === 'running') {
    return <LoaderCircle className={styles.spin} size={12} />;
  }
  if (isErrorResult(result, workUnit)) {
    return <CircleAlert className={styles.bad} size={12} />;
  }
  return undefined;
}

export function runningEngineForPage(context: ExplorerEngineStatusContext, pageSelected: boolean) {
  const selectedResult = (context.previewResults ?? []).find(
    (result) => result.run_engine_id === context.selectedRunEngineId,
  );
  return pageSelected && selectedResult?.status === 'running' ? selectedResult : undefined;
}

function enginePageWorkUnit(
  result: IngestPreviewResultRecord,
  context: ExplorerEngineStatusContext,
) {
  return (context.diagnosticWorkUnits ?? []).find(
    (unit) =>
      unit.phase === 'ocr' &&
      unit.run_id === context.runId &&
      unit.file_hash === context.document.file_hash && // skylos: ignore[SKY-D253] file_hash is a public document identifier for UI status matching.
      unit.page_no === context.pageNo &&
      unit.engine === result.engine_id,
  );
}

function isErrorResult(
  result: IngestPreviewResultRecord,
  workUnit: DiagnosticWorkUnitRecord | undefined,
) {
  return (
    workUnit?.status === 'failed' ||
    Boolean(workUnit?.error) ||
    result.status === 'failed' ||
    result.status === 'completed_with_errors' ||
    Boolean(result.error)
  );
}
