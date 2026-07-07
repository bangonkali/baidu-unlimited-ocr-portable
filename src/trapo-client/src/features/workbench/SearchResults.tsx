import type { DocumentSummary, HybridSearchFileResult, HybridSearchHit } from '../../api/types';
import type { SearchResultViewMode } from '../../routeSearch';
import { SearchRankedResults } from './SearchRankedResults';
import { SearchTreeResults } from './SearchTreeResults';
import styles from './SearchView.module.css';

interface SearchResultsProps {
  documents: Map<string, DocumentSummary>;
  files: HybridSearchFileResult[];
  hits: HybridSearchHit[];
  loading: boolean;
  query: string;
  view: SearchResultViewMode;
  onSelectHit: (hit: HybridSearchHit) => void;
}

export function SearchResults({
  documents,
  files,
  hits,
  loading,
  query,
  view,
  onSelectHit,
}: SearchResultsProps) {
  const hasQuery = query.trim().length > 0;
  return (
    <div className={styles.resultList}>
      {loading ? <div className={styles.empty}>Searching...</div> : null}
      {!loading && hasQuery && files.length === 0 ? (
        <div className={styles.empty}>No indexed matches</div>
      ) : null}
      {!hasQuery ? <div className={styles.empty}>Enter a phrase to search.</div> : null}
      {hasQuery && view === 'ranked' ? (
        <SearchRankedResults
          documents={documents}
          hits={hits}
          query={query}
          onSelectHit={onSelectHit}
        />
      ) : null}
      {hasQuery && view === 'tree' ? (
        <SearchTreeResults
          documents={documents}
          files={files}
          query={query}
          onSelectHit={onSelectHit}
        />
      ) : null}
    </div>
  );
}
