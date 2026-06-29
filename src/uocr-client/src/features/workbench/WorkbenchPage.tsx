import { useDebouncedValue } from '@tanstack/react-pacer';
import { Bot, Database, FileSearch, Search, Settings } from 'lucide-react';
import { useState } from 'react';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';

import {
  useDocumentRegions,
  useDocuments,
  useDocumentText,
  useIngestRuns,
  useModels,
  useOpenFolderDialog,
  useRunCommand,
  useRunEvents,
  useSettings,
  useStartIngest,
  useStatus,
} from '../../api/hooks';
import { IconButton } from '../../components/IconButton';
import {
  setSelectedProfile,
  setSelectedRoot,
  useWorkbenchState,
} from '../../stores/workbenchStore';
import { DetailsPane } from './DetailsPane';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { ExplorerTree } from './ExplorerTree';
import { IngestToolbar } from './IngestToolbar';
import { PreviewPane } from './PreviewPane';
import { StatusBar } from './StatusBar';
import { TextPane } from './TextPane';
import styles from './WorkbenchPage.module.css';

export function WorkbenchPage() {
  const workbench = useWorkbenchState();
  const [searchText, setSearchText] = useState('');
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const status = useStatus();
  const documents = useDocuments(debouncedSearch);
  const models = useModels();
  const settings = useSettings();
  const runs = useIngestRuns();
  const regions = useDocumentRegions(workbench.selection.fileHash);
  const text = useDocumentText(workbench.selection.fileHash);
  const folderDialog = useOpenFolderDialog();
  const startIngest = useStartIngest();
  const pauseRun = useRunCommand('pause');
  const stopRun = useRunCommand('stop');
  const activeRunId = status.data?.active_run_id ?? runs.data?.runs[0]?.run_id ?? null;
  useRunEvents(activeRunId);

  const profiles = models.data?.profiles.length
    ? models.data.profiles
    : [
        {
          key: workbench.selectedProfile,
          label: 'Practical zero-empty Q4',
          engine_name: '',
          description: '',
          default_max_tokens: 8192,
        },
      ];

  return (
    <div className={styles.shell}>
      <ActivityBar />
      <main className={styles.main}>
        <div className={styles.commandCenter}>
          <Search size={15} />
          <input
            aria-label="Search"
            onChange={(event) => setSearchText(event.target.value)}
            placeholder="Search"
            value={searchText}
          />
        </div>
        <IngestToolbar
          activeRunId={activeRunId}
          busy={startIngest.isPending || folderDialog.isPending}
          onPause={() => activeRunId && pauseRun.mutate(activeRunId)}
          onPickFolder={() => {
            void folderDialog.mutateAsync().then((result) => {
              if (!result.cancelled) {
                setSelectedRoot(result.selected_path);
              }
            });
          }}
          onProfileChange={setSelectedProfile}
          onRootPathChange={setSelectedRoot}
          onStart={() =>
            startIngest.mutate({
              profile_id: workbench.selectedProfile,
              root_path: workbench.selectedRoot,
            })
          }
          onStop={() => activeRunId && stopRun.mutate(activeRunId)}
          profiles={profiles}
          rootPath={workbench.selectedRoot}
          selectedProfile={workbench.selectedProfile}
        />
        <PanelGroup className={styles.body} direction="horizontal">
          <Panel defaultSize={19} minSize={14}>
            <ExplorerTree
              documents={documents.data?.documents ?? []}
              selectedFileHash={workbench.selection.fileHash}
            />
          </Panel>
          <ResizeHandle />
          <Panel defaultSize={58} minSize={34}>
            <PanelGroup direction="vertical">
              <Panel defaultSize={68} minSize={40}>
                <PanelGroup direction="horizontal">
                  <Panel defaultSize={58} minSize={30}>
                    <PreviewPane
                      boxes={regions.data?.boxes ?? []}
                      labelsVisible={workbench.labelsVisible}
                      overlayVisible={workbench.overlayVisible}
                      selectedRegionId={workbench.selection.regionId}
                    />
                  </Panel>
                  <ResizeHandle />
                  <Panel defaultSize={42} minSize={24}>
                    <TextPane
                      pages={text.data?.pages ?? []}
                      selectedRegionId={workbench.selection.regionId}
                    />
                  </Panel>
                </PanelGroup>
              </Panel>
              <ResizeHandle horizontal />
              <Panel defaultSize={32} minSize={16}>
                <DiagnosticsPanel runs={runs.data?.runs ?? []} />
              </Panel>
            </PanelGroup>
          </Panel>
          <ResizeHandle />
          <Panel defaultSize={23} minSize={17}>
            <DetailsPane
              labelsVisible={workbench.labelsVisible}
              models={models.data}
              overlayVisible={workbench.overlayVisible}
              selectedFileHash={workbench.selection.fileHash}
              selectedRegionId={workbench.selection.regionId}
              settings={settings.data}
            />
          </Panel>
        </PanelGroup>
        <StatusBar
          documentCount={documents.data?.documents.length ?? 0}
          runState={status.data?.state ?? 'offline'}
          selectedRoot={workbench.selectedRoot}
        />
      </main>
    </div>
  );
}

function ActivityBar() {
  return (
    <aside className={styles.activityBar} aria-label="Primary">
      <div className={styles.brand}>U</div>
      <nav className={styles.activityNav}>
        <IconButton icon={FileSearch} label="Workbench" pressed />
        <IconButton icon={Database} label="Models" />
        <IconButton icon={Bot} label="Diagnostics" />
      </nav>
      <IconButton icon={Settings} label="Settings" />
    </aside>
  );
}

function ResizeHandle({ horizontal = false }: { horizontal?: boolean }) {
  return (
    <PanelResizeHandle
      className={horizontal ? styles.resizeHandleHorizontal : styles.resizeHandle}
    />
  );
}
