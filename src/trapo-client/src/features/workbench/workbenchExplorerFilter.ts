import type { DocumentSummary, IngestRunRecord } from '../../api/types';
import type { WorkbenchRouteSearch } from '../../routeSearch';

export type WorkbenchExplorerScope = 'run' | 'all';

export interface WorkbenchExplorerFilter {
  scope: WorkbenchExplorerScope;
  runId?: string;
}

export function explorerScopeFromSearch(search?: WorkbenchRouteSearch): WorkbenchExplorerScope {
  return search?.run_scope === 'all' ? 'all' : 'run';
}

export function latestRunIdFromRuns(runs: IngestRunRecord[] | undefined) {
  return runs?.[0]?.run_id;
}

export function explorerFilterFromSearch(
  search: WorkbenchRouteSearch | undefined,
  runs: IngestRunRecord[],
): WorkbenchExplorerFilter {
  return {
    runId: search?.run ?? latestRunIdFromRuns(runs),
    scope: explorerScopeFromSearch(search),
  };
}

export function firstDocumentForRun(
  documents: DocumentSummary[],
  runs: IngestRunRecord[],
  runId: string | undefined,
) {
  if (!runId) {
    return documents[0];
  }
  const run = runs.find((item) => item.run_id === runId); // skylos: ignore[SKY-D253] run_id is public route/UI state, not a secret token.
  const hashes = run?.file_hashes ?? [];
  for (const hash of hashes) {
    const document = documents.find((item) => item.file_hash === hash); // skylos: ignore[SKY-D253] file_hash is public run membership metadata, not a secret token.
    if (document) {
      return document;
    }
  }
  return undefined;
}
