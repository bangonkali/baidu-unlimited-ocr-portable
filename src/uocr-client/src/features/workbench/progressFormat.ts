import type { DocumentSummary, IngestRunRecord } from '../../api/types';

export function clampProgress(value?: number) {
  if (!Number.isFinite(value)) {
    return 0;
  }
  return Math.max(0, Math.min(100, value ?? 0));
}

export function percentLabel(value?: number) {
  return `${Math.round(clampProgress(value))}%`;
}

export function documentPageLabel(document: DocumentSummary) {
  const total = document.total_pages ?? document.page_count ?? 0;
  const current = document.current_page ?? (total > 0 ? 1 : 0);
  return total > 0 ? `Page ${current}/${total}` : 'No pages';
}

export function runPageLabel(run?: IngestRunRecord) {
  if (!run) {
    return 'No active run';
  }
  const total = run.total_pages ?? 0;
  const current = run.current_page ?? Math.min((run.processed_pages ?? 0) + 1, total);
  return total > 0 ? `Page ${current}/${total}` : 'Preparing pages';
}
