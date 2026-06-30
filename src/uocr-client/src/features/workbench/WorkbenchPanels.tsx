import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';

import type {
  DocumentSummary,
  IngestRunRecord,
  LogRecord,
  ModelAssetRecord,
  OverlayBox,
  PageTextRecord,
} from '../../api/types';
import type { useWorkbenchState } from '../../stores/workbenchStore';
import { setAutoFollowRegions } from '../../stores/workbenchStore';
import { DetailsPane } from './DetailsPane';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { ExplorerTree } from './ExplorerTree';
import { PreviewPane } from './PreviewPane';
import { StartHere } from './StartHere';
import { TextPane } from './TextPane';
import styles from './WorkbenchPage.module.css';

interface WorkbenchPanelsProps {
  documents: DocumentSummary[];
  logs: LogRecord[];
  model?: ModelAssetRecord;
  onOpenModels: () => void;
  onPickFolder: () => void;
  onStart: () => void;
  previewPages: number[];
  regions: OverlayBox[];
  rootPath: string;
  runs: IngestRunRecord[];
  selectedDocument?: DocumentSummary;
  textPages: PageTextRecord[];
  workbench: ReturnType<typeof useWorkbenchState>;
}

export function WorkbenchPanels(props: WorkbenchPanelsProps) {
  const selectedRegion = props.regions.find(
    (region) => region.region_id === props.workbench.selection.regionId,
  );
  return (
    <div className={styles.workbenchStack}>
      <StartHere
        model={props.model}
        onOpenModels={props.onOpenModels}
        onPickFolder={props.onPickFolder}
        onStart={props.onStart}
        rootPath={props.rootPath}
      />
      <PanelGroup direction="horizontal">
        <Panel defaultSize={19} minSize={14}>
          <ExplorerTree
            documents={props.documents}
            selectedFileHash={props.workbench.selection.fileHash}
          />
        </Panel>
        <ResizeHandle />
        <Panel defaultSize={58} minSize={34}>
          <DocumentWorkspace {...props} />
        </Panel>
        <ResizeHandle />
        <Panel defaultSize={23} minSize={17}>
          <DetailsPane
            document={props.selectedDocument}
            labelsVisible={props.workbench.labelsVisible}
            overlayVisible={props.workbench.overlayVisible}
            selectedRegion={selectedRegion}
            selectedRegionId={props.workbench.selection.regionId}
          />
        </Panel>
      </PanelGroup>
    </div>
  );
}

function DocumentWorkspace(props: WorkbenchPanelsProps) {
  return (
    <PanelGroup direction="vertical">
      <Panel defaultSize={68} minSize={40}>
        <PanelGroup direction="horizontal">
          <Panel defaultSize={58} minSize={30}>
            <PreviewPane
              autoFollowRegions={props.workbench.autoFollowRegions}
              boxes={props.regions}
              fileHash={props.workbench.selection.fileHash}
              labelsVisible={props.workbench.labelsVisible}
              overlayVisible={props.workbench.overlayVisible}
              pages={props.previewPages}
              selectedPageNo={props.workbench.selection.pageNo}
              selectedRegionId={props.workbench.selection.regionId}
              onAutoFollowChange={setAutoFollowRegions}
            />
          </Panel>
          <ResizeHandle />
          <Panel defaultSize={42} minSize={24}>
            <TextPane
              autoFollowRegions={props.workbench.autoFollowRegions}
              document={props.selectedDocument}
              pages={props.textPages}
              selectedRegionId={props.workbench.selection.regionId}
            />
          </Panel>
        </PanelGroup>
      </Panel>
      <ResizeHandle horizontal />
      <Panel defaultSize={32} minSize={16}>
        <DiagnosticsPanel logs={props.logs} runs={props.runs} />
      </Panel>
    </PanelGroup>
  );
}

function ResizeHandle({ horizontal = false }: { horizontal?: boolean }) {
  return (
    <PanelResizeHandle
      className={horizontal ? styles.resizeHandleHorizontal : styles.resizeHandle}
    />
  );
}
