import { FileText, ScanText } from 'lucide-react';

import type { IngestPreviewResultRecord } from '../../api/types';
import styles from './EngineResultSwitcher.module.css';

interface EngineResultSwitcherProps {
  results: IngestPreviewResultRecord[];
  selectedRunEngineId?: string;
  onSelect: (runEngineId: string) => void;
}

export function EngineResultSwitcher({
  onSelect,
  results,
  selectedRunEngineId,
}: EngineResultSwitcherProps) {
  if (results.length === 0) {
    return null;
  }
  const selectedId = selectedRunEngineId ?? results[0]?.run_engine_id;
  return (
    <div className={styles.switcher} aria-label="Engine results" role="toolbar">
      {results.map((result) => (
        <button
          aria-pressed={result.run_engine_id === selectedId}
          className={styles.resultButton}
          data-active={result.run_engine_id === selectedId}
          data-runner={result.runner_status}
          key={result.run_engine_id}
          onClick={() => onSelect(result.run_engine_id)}
          title={resultTitle(result)}
          type="button"
        >
          {result.previewer === 'document_markdown' ? (
            <FileText size={14} />
          ) : (
            <ScanText size={14} />
          )}
          <span className={styles.label}>{result.label}</span>
          <span className={styles.meta}>
            {result.page_count}p · {statusLabel(result.status)}
          </span>
          <span className={styles.runner}>{runnerLabel(result)}</span>
        </button>
      ))}
    </div>
  );
}

function resultTitle(result: IngestPreviewResultRecord) {
  const parts = [
    result.label,
    `status: ${statusLabel(result.status)}`,
    `runner: ${runnerLabel(result)}`,
  ];
  if (result.model_id) {
    parts.push(`model: ${result.model_id}`);
  }
  if (result.runtime_id) {
    parts.push(`runtime: ${result.runtime_id}`);
  }
  if (result.runner_detail) {
    parts.push(result.runner_detail);
  }
  return parts.join(' · ');
}

function runnerLabel(result: IngestPreviewResultRecord) {
  if (result.runner_status === 'ready') {
    return result.runner_kind;
  }
  return result.runner_status.replaceAll('_', ' ');
}

function statusLabel(status: string) {
  switch (status) {
    case 'completed':
      return 'ready';
    case 'completed_with_errors':
      return 'partial';
    case 'running':
      return 'running';
    case 'failed':
      return 'failed';
    default:
      return status.replaceAll('_', ' ');
  }
}
