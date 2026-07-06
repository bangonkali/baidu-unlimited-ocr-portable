import type { RefObject } from 'react';
import { useEffect, useRef } from 'react';
import type { ImperativePanelHandle } from 'react-resizable-panels';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';
import { annotationIdOf } from '../../api/annotationIdentity';
import type {
  DocumentSummary,
  IngestRunRecord,
  LogRecord,
  ModelAssetRecord,
  OverlayBox,
  PageTextRecord,
} from '../../api/types';
import type { useWorkbenchState } from '../../stores/workbenchStore';
import { setPaneCollapsed } from '../../stores/workbenchStore';
import { DetailsPane } from './DetailsPane';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { ExplorerTree } from './ExplorerTree';
import { PreviewPane } from './PreviewPane';
import { TextPane } from './TextPane';
import { scopedRegionText } from './traceRegionAnchors';
import styles from './WorkbenchPage.module.css';
import type { WorkbenchExplorerFilter } from './workbenchExplorerFilter';

export interface WorkbenchPanelsProps {
  activeRunId?: string | null;
  documents: DocumentSummary[];
  explorerFilter: WorkbenchExplorerFilter;
  logs: LogRecord[];
  model?: ModelAssetRecord;
  onOpenModels: () => void;
  onPickFolder: () => void;
  onResumeRun: (runId: string) => void;
  onRestartRun: (run: IngestRunRecord) => void;
  onStart: () => void;
  onStopRun: (runId?: string) => void;
  previewPages: number[];
  regions: OverlayBox[];
  rootPath: string;
  runs: IngestRunRecord[];
  selectedDocument?: DocumentSummary;
  textPages: PageTextRecord[];
  workbench: ReturnType<typeof useWorkbenchState>;
  onAutoFollowChange: (enabled: boolean) => void;
  onExplorerFilterChange: (filter: WorkbenchExplorerFilter) => void;
  onSelectDocument: (fileHash: string, pageNo?: number, runId?: string) => void;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}

export function WorkbenchPanels(props: WorkbenchPanelsProps) {
  const explorerRef = useRef<ImperativePanelHandle>(null);
  const detailsRef = useRef<ImperativePanelHandle>(null);
  const selectedRegion = props.regions.find(
    (region) => annotationIdOf(region) === props.workbench.selection.regionId,
  );
  const selectedRegionContent = selectedRegion
    ? scopedRegionText(props.textPages, annotationIdOf(selectedRegion))
    : undefined;
  usePanelCollapseSync(explorerRef, props.workbench.panesCollapsed.explorer);
  usePanelCollapseSync(detailsRef, props.workbench.panesCollapsed.details);
  return (
    <div className={styles.workbenchStack}>
      <PanelGroup direction="horizontal">
        <Panel
          collapsible
          collapsedSize={0}
          defaultSize={19}
          minSize={14}
          onCollapse={() => setPaneCollapsed('explorer', true)}
          onExpand={() => setPaneCollapsed('explorer', false)}
          ref={explorerRef}
        >
          <ExplorerTree
            documents={props.documents}
            filter={props.explorerFilter}
            onFilterChange={props.onExplorerFilterChange}
            onSelectDocument={props.onSelectDocument}
            rootPath={props.rootPath}
            runs={props.runs}
            selectedFileHash={props.workbench.selection.fileHash}
            selectedRunId={props.explorerFilter.runId}
          />
        </Panel>
        <ResizeHandle />
        <Panel defaultSize={58} minSize={34}>
          <DocumentWorkspace {...props} />
        </Panel>
        <ResizeHandle />
        <Panel
          collapsible
          collapsedSize={0}
          defaultSize={23}
          minSize={17}
          onCollapse={() => setPaneCollapsed('details', true)}
          onExpand={() => setPaneCollapsed('details', false)}
          ref={detailsRef}
        >
          <DetailsPane
            document={props.selectedDocument}
            labelsVisible={props.workbench.labelsVisible}
            overlayVisible={props.workbench.overlayVisible}
            selectedRegion={selectedRegion}
            selectedRegionContent={selectedRegionContent}
            selectedRegionId={props.workbench.selection.regionId}
          />
        </Panel>
      </PanelGroup>
    </div>
  );
}

export function DocumentWorkspace(props: WorkbenchPanelsProps) {
  const diagnosticsRef = useRef<ImperativePanelHandle>(null);
  usePanelCollapseSync(diagnosticsRef, props.workbench.panesCollapsed.diagnostics);
  return (
    <PanelGroup direction="vertical">
      <Panel defaultSize={68} minSize={40}>
        <PanelGroup direction="horizontal">
          <Panel defaultSize={58} minSize={30}>
            <PreviewPane
              autoFollowRegions={props.workbench.autoFollowRegions}
              boxes={props.regions}
              fileHash={props.workbench.selection.fileHash}
              focusRevision={props.workbench.focusRevision}
              labelsVisible={props.workbench.labelsVisible}
              overlayVisible={props.workbench.overlayVisible}
              pages={props.previewPages}
              selectedPageNo={props.workbench.selection.pageNo}
              selectedRegionId={props.workbench.selection.regionId}
              onAutoFollowChange={props.onAutoFollowChange}
              onSelectRegion={props.onSelectRegion}
            />
          </Panel>
          <ResizeHandle />
          <Panel defaultSize={42} minSize={24}>
            <TextPane
              autoFollowRegions={props.workbench.autoFollowRegions}
              document={props.selectedDocument}
              focusRevision={props.workbench.focusRevision}
              onSelectRegion={props.onSelectRegion}
              pages={props.textPages}
              regions={props.regions}
              selectedRegionId={props.workbench.selection.regionId}
            />
          </Panel>
        </PanelGroup>
      </Panel>
      <ResizeHandle horizontal />
      <Panel
        collapsible
        collapsedSize={0}
        defaultSize={32}
        minSize={16}
        onCollapse={() => setPaneCollapsed('diagnostics', true)}
        onExpand={() => setPaneCollapsed('diagnostics', false)}
        ref={diagnosticsRef}
      >
        <DiagnosticsPanel
          logs={props.logs}
          activeRunId={props.activeRunId}
          onResumeRun={props.onResumeRun}
          onRestartRun={props.onRestartRun}
          onStopRun={props.onStopRun}
          runs={props.runs}
        />
      </Panel>
    </PanelGroup>
  );
}

function usePanelCollapseSync(
  panelRef: RefObject<ImperativePanelHandle | null>,
  collapsed: boolean,
) {
  useEffect(() => {
    const panel = panelRef.current;
    if (!panel) {
      return;
    }
    if (collapsed && !panel.isCollapsed()) {
      panel.collapse();
    }
    if (!collapsed && panel.isCollapsed()) {
      panel.expand();
    }
  }, [collapsed, panelRef]);
}

function ResizeHandle({ horizontal = false }: { horizontal?: boolean }) {
  return (
    <PanelResizeHandle
      className={horizontal ? styles.resizeHandleHorizontal : styles.resizeHandle}
    />
  );
}
