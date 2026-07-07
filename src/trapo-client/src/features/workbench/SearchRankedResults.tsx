import type { DocumentSummary, HybridSearchHit } from '../../api/types';
import { SearchHitButton } from './SearchHitButton';
import styles from './SearchView.module.css';

interface SearchRankedResultsProps {
  documents: Map<string, DocumentSummary>;
  hits: HybridSearchHit[];
  query: string;
  onSelectHit: (hit: HybridSearchHit) => void;
}

export function SearchRankedResults({
  documents,
  hits,
  query,
  onSelectHit,
}: SearchRankedResultsProps) {
  return (
    <div className={styles.rankedResults}>
      {hits.map((hit) => (
        <SearchHitButton
          document={documents.get(hit.file_hash)}
          hit={hit}
          key={`${hit.rank}:${hit.segment_id}:${hit.hit_source}`}
          query={query}
          ranked
          onSelectHit={onSelectHit}
        />
      ))}
    </div>
  );
}
