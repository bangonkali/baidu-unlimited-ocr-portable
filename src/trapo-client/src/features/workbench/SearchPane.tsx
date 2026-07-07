import { Search } from 'lucide-react';

import type {
  DocumentSummary,
  HybridSearchFileResult,
  HybridSearchHit,
  IngestRunRecord,
  UsedEmbeddingModelRecord,
} from '../../api/types';
import type { SearchResultViewMode, SearchRouteSearch } from '../../routeSearch';
import { SearchResults } from './SearchResults';
import styles from './SearchView.module.css';

interface SearchPaneProps {
  documents: Map<string, DocumentSummary>;
  files: HybridSearchFileResult[];
  hits: HybridSearchHit[];
  loading: boolean;
  models: UsedEmbeddingModelRecord[];
  query: string;
  runId?: string;
  runs: IngestRunRecord[];
  selectedModelId: string;
  view: SearchResultViewMode;
  onChange: (patch: Partial<SearchRouteSearch>) => void;
  onSelectHit: (hit: HybridSearchHit) => void;
}

export function SearchPane({
  documents,
  files,
  hits,
  loading,
  models,
  query,
  runId,
  runs,
  selectedModelId,
  view,
  onChange,
  onSelectHit,
}: SearchPaneProps) {
  return (
    <aside className={styles.searchPane} aria-label="Search">
      <header className={styles.header}>
        <Search size={15} />
        <span>Search</span>
      </header>
      <div className={styles.controls}>
        <select
          aria-label="Result view"
          onChange={(event) => onChange({ view: viewFromValue(event.currentTarget.value) })}
          value={view}
        >
          <option value="tree">Tree by file</option>
          <option value="ranked">Ranked hits</option>
        </select>
        <input
          aria-label="Search phrase"
          autoComplete="off"
          onChange={(event) => onChange({ q: event.target.value || undefined })}
          placeholder="Search text and embeddings"
          value={query}
        />
        <select
          aria-label="Embedding model"
          onChange={(event) => onChange({ model: event.target.value || undefined })}
          value={selectedModelId}
        >
          <option value="">FTS only</option>
          {models.map((model) => (
            <option key={model.model_id} value={model.model_id}>
              {model.display_name}
            </option>
          ))}
        </select>
        <select
          aria-label="Run"
          onChange={(event) => onChange({ run: event.target.value || undefined })}
          value={runId ?? ''}
        >
          <option value="">All indexed runs</option>
          {runs.map((run) => (
            <option key={run.run_id} value={run.run_id}>
              {shortRunLabel(run.run_id)}
            </option>
          ))}
        </select>
      </div>
      <SearchResults
        documents={documents}
        files={files}
        hits={hits}
        loading={loading}
        query={query}
        view={view}
        onSelectHit={onSelectHit}
      />
    </aside>
  );
}

function viewFromValue(value: string): SearchResultViewMode | undefined {
  return value === 'ranked' ? 'ranked' : undefined;
}

function shortRunLabel(runId: string) {
  return runId.length > 8 ? runId.slice(0, 8) : runId;
}
