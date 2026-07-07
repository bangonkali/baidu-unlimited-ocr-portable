import { ChevronDown, ChevronRight } from 'lucide-react';
import { useEffect, useState } from 'react';

import type { DocumentSummary, HybridSearchFileResult, HybridSearchHit } from '../../api/types';
import { SearchHitButton } from './SearchHitButton';
import styles from './SearchView.module.css';

interface SearchTreeResultsProps {
  documents: Map<string, DocumentSummary>;
  files: HybridSearchFileResult[];
  query: string;
  onSelectHit: (hit: HybridSearchHit) => void;
}

export function SearchTreeResults({
  documents,
  files,
  query,
  onSelectHit,
}: SearchTreeResultsProps) {
  const [expandedIds, setExpandedIds] = useState(() => defaultExpandedFileIds(files));
  useEffect(() => {
    setExpandedIds(defaultExpandedFileIds(files));
  }, [files]);

  return (
    <div className={styles.searchTree} role="tree">
      {files.map((file) => (
        <FileResult
          document={documents.get(file.file_hash)}
          expanded={expandedIds.has(fileNodeId(file.file_hash))}
          file={file}
          key={file.file_hash}
          query={query}
          onSelectHit={onSelectHit}
          onToggle={() => toggleExpanded(file.file_hash, setExpandedIds)}
        />
      ))}
    </div>
  );
}

function FileResult({
  document,
  expanded,
  file,
  query,
  onSelectHit,
  onToggle,
}: {
  document?: DocumentSummary;
  expanded: boolean;
  file: HybridSearchFileResult;
  query: string;
  onSelectHit: (hit: HybridSearchHit) => void;
  onToggle: () => void;
}) {
  const label = document?.display_name ?? file.file_hash;
  const nodeId = fileNodeId(file.file_hash);
  return (
    <section className={styles.fileResult}>
      <button
        aria-controls={`${nodeId}:hits`}
        aria-expanded={expanded}
        aria-label={expanded ? `Collapse ${label}` : `Expand ${label}`}
        className={styles.fileHeader}
        onClick={onToggle}
        role="treeitem"
        type="button"
      >
        <span className={styles.fileChevron} aria-hidden="true">
          {expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
        </span>
        <span className={styles.fileLabel} title={label}>
          {label}
        </span>
        <b>{file.hit_count}</b>
      </button>
      {expanded ? (
        <div className={styles.fileHits} id={`${nodeId}:hits`}>
          {file.hits.map((hit) => (
            <SearchHitButton
              hit={hit}
              key={`${hit.rank}:${hit.segment_id}:${hit.hit_source}`}
              query={query}
              treeItem
              onSelectHit={onSelectHit}
            />
          ))}
        </div>
      ) : null}
    </section>
  );
}

function defaultExpandedFileIds(files: HybridSearchFileResult[]) {
  return new Set(files.map((file) => fileNodeId(file.file_hash)));
}

function toggleExpanded(
  fileHash: string,
  setExpandedIds: (update: (current: Set<string>) => Set<string>) => void,
) {
  const id = fileNodeId(fileHash);
  setExpandedIds((current) => {
    const next = new Set(current);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    return next;
  });
}

function fileNodeId(fileHash: string) {
  return `search-file:${fileHash}`;
}
