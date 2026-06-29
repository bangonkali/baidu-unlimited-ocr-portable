import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { CircleHelp, Search } from 'lucide-react';
import { useState } from 'react';

import {
  queryKeys,
  useDocumentPreviewImages,
  useDocumentRegions,
  useDocuments,
  useDocumentText,
  useDownloadModel,
  useIngestRuns,
  useLogs,
  useModels,
  useOpenFolderDialog,
  useRunCommand,
  useRunEvents,
  useStartIngest,
  useStatus,
} from '../../api/hooks';
import { IconButton } from '../../components/IconButton';
import {
  setActiveView,
  setSelectedProfile,
  setSelectedRoot,
  setTourRun,
  useWorkbenchState,
} from '../../stores/workbenchStore';
import { ActivityBar } from './ActivityBar';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { GuidedTour } from './GuidedTour';
import { IngestToolbar } from './IngestToolbar';
import { ModelManager } from './ModelManager';
import { StatusBar } from './StatusBar';
import styles from './WorkbenchPage.module.css';
import { WorkbenchPanels } from './WorkbenchPanels';

export function WorkbenchPage() {
  const queryClient = useQueryClient();
  const workbench = useWorkbenchState();
  const [searchText, setSearchText] = useState('');
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const status = useStatus();
  const documents = useDocuments(debouncedSearch);
  const models = useModels();
  const runs = useIngestRuns();
  const logs = useLogs(220);
  const regions = useDocumentRegions(workbench.selection.fileHash);
  const text = useDocumentText(workbench.selection.fileHash);
  const previewImages = useDocumentPreviewImages(workbench.selection.fileHash);
  const folderDialog = useOpenFolderDialog();
  const downloadModel = useDownloadModel();
  const startIngest = useStartIngest();
  const stopRun = useRunCommand('stop');
  const activeRunId = status.data?.active_run_id ?? null;
  useRunEvents(activeRunId);

  const model = models.data?.models[0];
  const modelReady = model?.status === 'downloaded';
  const selectedDocument = documents.data?.documents.find(
    (document) => document.file_hash === workbench.selection.fileHash,
  );
  const profiles = models.data?.profiles.length
    ? models.data.profiles
    : [
        {
          key: workbench.selectedProfile,
          label: 'Practical zero-empty Q4',
          engine_name: 'Unlimited-OCR FFI',
          description: '',
          default_max_tokens: 8192,
        },
      ];

  const pickFolder = () => {
    void folderDialog.mutateAsync().then((result) => {
      if (!result.cancelled) {
        setSelectedRoot(result.selected_path);
      }
    });
  };
  const startScan = () =>
    startIngest.mutate({
      profile_id: workbench.selectedProfile,
      root_path: workbench.selectedRoot,
    });
  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.status });
    void queryClient.invalidateQueries({ queryKey: queryKeys.models });
    void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
    void queryClient.invalidateQueries({ queryKey: ['documents'] });
    void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
  };

  return (
    <div className={styles.shell}>
      <GuidedTour run={workbench.tourRun} />
      <ActivityBar activeView={workbench.activeView} />
      <main className={styles.main}>
        <div className={styles.commandCenter}>
          <Search size={15} />
          <input
            aria-label="Search documents"
            onChange={(event) => setSearchText(event.target.value)}
            placeholder="Search documents"
            value={searchText}
          />
          <IconButton icon={CircleHelp} label="Start guide" onClick={() => setTourRun(true)} />
        </div>
        <IngestToolbar
          activeRunId={activeRunId}
          busy={startIngest.isPending || folderDialog.isPending}
          modelReady={modelReady}
          onPickFolder={pickFolder}
          onProfileChange={setSelectedProfile}
          onRefresh={refresh}
          onRootPathChange={setSelectedRoot}
          onStart={startScan}
          onStop={() => activeRunId && stopRun.mutate(activeRunId)}
          profiles={profiles}
          rootPath={workbench.selectedRoot}
          runState={status.data?.state}
          selectedProfile={workbench.selectedProfile}
          supportedInputs={status.data?.supported_inputs}
        />
        <div className={styles.body}>
          {workbench.activeView === 'models' ? (
            <ModelManager
              busy={downloadModel.isPending}
              models={models.data}
              onDownloadModel={(modelId) => downloadModel.mutate(modelId)}
              status={status.data}
            />
          ) : null}
          {workbench.activeView === 'diagnostics' ? (
            <DiagnosticsPanel logs={logs.data?.logs ?? []} runs={runs.data?.runs ?? []} />
          ) : null}
          {workbench.activeView === 'workbench' ? (
            <WorkbenchPanels
              documents={documents.data?.documents ?? []}
              logs={logs.data?.logs ?? []}
              model={model}
              onOpenModels={() => setActiveView('models')}
              onPickFolder={pickFolder}
              onStart={startScan}
              previewPages={previewImages.data?.pages ?? []}
              regions={regions.data?.boxes ?? []}
              rootPath={workbench.selectedRoot}
              runs={runs.data?.runs ?? []}
              selectedDocument={selectedDocument}
              textPages={text.data?.pages ?? []}
              workbench={workbench}
            />
          ) : null}
        </div>
        <StatusBar
          documentCount={documents.data?.documents.length ?? 0}
          host={window.location.host}
          logPath={status.data?.log_path}
          runState={status.data?.state ?? 'offline'}
          runtime={status.data?.runtime_platform ?? 'windows-x86_64-cuda13'}
          selectedRoot={workbench.selectedRoot}
        />
      </main>
    </div>
  );
}
