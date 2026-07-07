import { useMemo } from 'react';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';

import { useHybridSearch, useUsedEmbeddingModels } from '../../api/hooks';
import type { HybridSearchHit } from '../../api/types';
import type { SearchRouteSearch } from '../../routeSearch';
import { setAutoFollowRegions, setOverlayVisible, setSelection } from '../../stores/workbenchStore';
import { SearchPane } from './SearchPane';
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
  const files = search.data?.files ?? [];
  const hits = search.data?.hits ?? files.flatMap((file) => file.hits);
  const selectHit = (hit: HybridSearchHit) => {
    setAutoFollowRegions(false);
    if (hit.annotation_id) {
      setOverlayVisible(true);
    }
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
            files={files}
            hits={hits}
            loading={search.isFetching}
            models={usedModels.data?.models ?? []}
            query={query}
            runId={runId}
            runs={props.runs}
            selectedModelId={props.search?.model ?? ''}
            view={props.search?.view ?? 'tree'}
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
