import { Search } from 'lucide-react';
import type { ReactNode } from 'react';
import { useMemo } from 'react';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';

import { useHybridSearch, useUsedEmbeddingModels } from '../../api/hooks';
import type { DocumentSummary, HybridSearchFileResult, HybridSearchHit } from '../../api/types';
import type { SearchRouteSearch } from '../../routeSearch';
import { setAutoFollowRegions, setSelection } from '../../stores/workbenchStore';
import styles from './SearchView.module.css';
import type { WorkbenchPanelsProps } from './WorkbenchPanels';
import { DocumentWorkspace } from './WorkbenchPanels';

interface SearchViewProps
  extends Omit<
    WorkbenchPanelsProps,
    'explorerFilter' | 'onExplorerFilterChange' | 'onSelectDocument' | 'onSelectRegion' | 'onStart'
  > {
  search?: SearchRouteSearch;
  onRouteSearchChange: (patch: Partial<SearchRouteSearch>) => void;
}

export function SearchView(props: SearchViewProps) {
  const usedModels = useUsedEmbeddingModels();
  const query = props.search?.q ?? '';
  const runId = props.search?.run;
  const request = useMemo(
    () => ({
      embedding_model_id: props.search?.model,
      limit: 60,
      query,
      source_run_id: runId,
    }),
    [props.search?.model, query, runId],
  );
  const search = useHybridSearch(request, query.trim().length > 0);
  const documentByHash = useMemo(
    () => new Map(props.documents.map((document) => [document.file_hash, document])),
    [props.documents],
  );
  const selectHit = (hit: HybridSearchHit) => {
    setAutoFollowRegions(false);
    setSelection({
      fileHash: hit.file_hash,
      pageNo: hit.page_no,
      regionId: hit.annotation_id ?? undefined,
      runId,
    });
  };
  const workspaceProps: WorkbenchPanelsProps = {
    ...props,
    explorerFilter: { runId, scope: 'run' },
    onExplorerFilterChange: () => undefined,
    onSelectDocument: (fileHash, pageNo = 1, targetRunId) => {
      setAutoFollowRegions(false);
      setSelection({ fileHash, pageNo, regionId: undefined, runId: targetRunId ?? runId });
    },
    onSelectRegion: (pageNo, regionId) => {
      setAutoFollowRegions(false);
      setSelection({ pageNo, regionId });
    },
    onStart: () => undefined,
  };

  return (
    <div className={styles.searchShell}>
      <PanelGroup direction="horizontal">
        <Panel defaultSize={27} minSize={18}>
          <SearchPane
            documents={documentByHash}
            files={search.data?.files ?? []}
            loading={search.isFetching}
            models={usedModels.data?.models ?? []}
            query={query}
            runId={runId}
            runs={props.runs}
            selectedModelId={props.search?.model ?? ''}
            onChange={props.onRouteSearchChange}
            onSelectHit={selectHit}
          />
        </Panel>
        <PanelResizeHandle className={styles.resizeHandle} />
        <Panel defaultSize={73} minSize={42}>
          <DocumentWorkspace {...workspaceProps} />
        </Panel>
      </PanelGroup>
    </div>
  );
}

function SearchPane({
  documents,
  files,
  loading,
  models,
  query,
  runId,
  runs,
  selectedModelId,
  onChange,
  onSelectHit,
}: {
  documents: Map<string, DocumentSummary>;
  files: HybridSearchFileResult[];
  loading: boolean;
  models: Array<{ dimension: number; display_name: string; model_id: string; provider: string }>;
  query: string;
  runId?: string;
  runs: WorkbenchPanelsProps['runs'];
  selectedModelId: string;
  onChange: (patch: Partial<SearchRouteSearch>) => void;
  onSelectHit: (hit: HybridSearchHit) => void;
}) {
  return (
    <aside className={styles.searchPane} aria-label="Search">
      <header className={styles.header}>
        <Search size={15} />
        <span>Search</span>
      </header>
      <div className={styles.controls}>
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
      <div className={styles.resultList}>
        {loading ? <div className={styles.empty}>Searching...</div> : null}
        {!loading && query.trim() && files.length === 0 ? (
          <div className={styles.empty}>No indexed matches</div>
        ) : null}
        {!query.trim() ? <div className={styles.empty}>Enter a phrase to search.</div> : null}
        {files.map((file) => (
          <FileResult
            document={documents.get(file.file_hash)}
            file={file}
            key={file.file_hash}
            query={query}
            onSelectHit={onSelectHit}
          />
        ))}
      </div>
    </aside>
  );
}

function FileResult({
  document,
  file,
  query,
  onSelectHit,
}: {
  document?: DocumentSummary;
  file: HybridSearchFileResult;
  query: string;
  onSelectHit: (hit: HybridSearchHit) => void;
}) {
  return (
    <section className={styles.fileResult}>
      <div className={styles.fileHeader}>
        <span>{document?.display_name ?? file.file_hash}</span>
        <b>{file.hit_count}</b>
      </div>
      {file.hits.map((hit) => (
        <button
          className={styles.hit}
          data-source={hit.hit_source.startsWith('vss') ? 'vss' : 'fts'}
          key={`${hit.segment_id}:${hit.hit_source}:${hit.score}`}
          onClick={() => onSelectHit(hit)}
          type="button"
        >
          <span className={styles.hitMeta}>
            page {hit.page_no} · {hit.category} · {hit.hit_source}
          </span>
          <span className={styles.snippet}>
            {highlightSnippet(hit.text, query, hit.hit_source)}
          </span>
        </button>
      ))}
    </section>
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

function shortRunLabel(runId: string) {
  return runId.length > 8 ? runId.slice(0, 8) : runId;
}
