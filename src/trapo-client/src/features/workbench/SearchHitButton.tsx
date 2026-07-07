import type { ReactNode } from 'react';

import type { DocumentSummary, HybridSearchHit } from '../../api/types';
import styles from './SearchView.module.css';

interface SearchHitButtonProps {
  hit: HybridSearchHit;
  query: string;
  document?: DocumentSummary;
  ranked?: boolean;
  treeItem?: boolean;
  onSelectHit: (hit: HybridSearchHit) => void;
}

export function SearchHitButton({
  document,
  hit,
  query,
  ranked = false,
  treeItem = false,
  onSelectHit,
}: SearchHitButtonProps) {
  return (
    <button
      className={classNames(styles.hit, ranked ? styles.rankedHit : undefined)}
      onClick={() => onSelectHit(hit)}
      role={treeItem ? 'treeitem' : undefined}
      type="button"
    >
      <span className={styles.hitMeta}>
        {ranked ? (
          <>
            <span className={styles.hitRank}>#{hit.rank}</span> ·{' '}
            {document?.display_name ?? hit.file_hash} ·{' '}
          </>
        ) : null}
        page {hit.page_no} · {hit.category} ·{' '}
        <span className={styles.hitSource}>{hit.hit_source}</span>
      </span>
      <span className={styles.snippet}>{highlightSnippet(hit.text, query, hit.hit_source)}</span>
    </button>
  );
}

function highlightSnippet(text: string, query: string, source: string) {
  const snippet = text.length > 220 ? `${text.slice(0, 220)}...` : text;
  const needle = query.trim();
  if (!source.startsWith('fts') || !needle) {
    return snippet;
  }
  const matcher = new RegExp(needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
  const nodes: ReactNode[] = [];
  let lastIndex = 0;
  for (const match of snippet.matchAll(matcher)) {
    const start = match.index ?? 0;
    const end = start + match[0].length;
    if (start > lastIndex) {
      nodes.push(<span key={`text:${lastIndex}:${start}`}>{snippet.slice(lastIndex, start)}</span>);
    }
    nodes.push(<mark key={`match:${start}:${end}`}>{snippet.slice(start, end)}</mark>);
    lastIndex = end;
  }
  if (lastIndex < snippet.length) {
    nodes.push(<span key={`text:${lastIndex}:${snippet.length}`}>{snippet.slice(lastIndex)}</span>);
  }
  return nodes;
}

function classNames(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(' ');
}
