import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { CircleHelp, Search } from 'lucide-react';
import { useEffect, useState } from 'react';

import {
  queryKeys,
  useCancelModelDownload,
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
  useSelectModel,
  useStartIngest,
  useStatus,
} from '../../api/hooks';
import type { DocumentRegionsPayload, ModelsPayload, OcrProfileRecord } from '../../api/types';
import { IconButton } from '../../components/IconButton';
import { useRealtimeState } from '../../realtime/realtimeStore';
import {
  followLatestRegion,
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

function selectedModel(models?: ModelsPayload) {
  return (
    models?.models.find((item) => item.selected) ??
    models?.models.find((item) => item.model_id === models.selected_model_id) ??
    models?.models[0]
  );
}

function profileOptions(profiles: OcrProfileRecord[] | undefined, selectedProfile: string) {
  return profiles?.length
    ? profiles
    : [
        {
          key: selectedProfile,
          label: 'Experimental exact-prefill Q4',
          engine_name: 'Unlimited-OCR FFI',
          description: '',
          default_max_tokens: 8192,
        },
      ];
}

function useAutoFollowLatestRegion(
  workbench: ReturnType<typeof useWorkbenchState>,
  regions?: DocumentRegionsPayload,
) {
  const latestRegion = regions?.boxes.at(-1);
  useEffect(() => {
    if (!workbench.autoFollowRegions || !regions || !latestRegion) {
      return;
    }
    if (workbench.selection.regionId === latestRegion.region_id) {
      return;
    }
    followLatestRegion(regions.file_hash, regions.boxes);
  }, [latestRegion, regions, workbench.autoFollowRegions, workbench.selection.regionId]);
}

export function WorkbenchPage() {
  const queryClient = useQueryClient();
  const workbench = useWorkbenchState();
  const realtime = useRealtimeState();
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
  const cancelModelDownload = useCancelModelDownload();
  const selectModel = useSelectModel();
  const startIngest = useStartIngest();
  const stopRun = useRunCommand('stop');
  const activeRunId = status.data?.active_run_id ?? null;
  const activeRun = runs.data?.runs.find((run) => run.run_id === activeRunId);

  const model = selectedModel(models.data);
  const modelReady = model?.status === 'downloaded';
  const selectedDocument = documents.data?.documents.find(
    (document) => document.file_hash === workbench.selection.fileHash,
  );
  const profiles = profileOptions(models.data?.profiles, workbench.selectedProfile);
  useAutoFollowLatestRegion(workbench, regions.data);

  const pickFolder = () => {
    void folderDialog.mutateAsync().then((result) => {
      if (!result.cancelled) {
        setSelectedRoot(result.selected_path);
      }
    });
  };
  const startScan = () =>
    startIngest.mutate({
      model_id: model?.model_id,
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
          activeRun={activeRun}
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
              busy={
                downloadModel.isPending || cancelModelDownload.isPending || selectModel.isPending
              }
              models={models.data}
              onCancelModel={(modelId) => cancelModelDownload.mutate(modelId)}
              onDownloadModel={(modelId, force) => downloadModel.mutate({ force, modelId })}
              onSelectModel={(modelId) => selectModel.mutate(modelId)}
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
          realtimeState={realtime.connectionState}
          runState={status.data?.state ?? 'offline'}
          runtime={status.data?.runtime_platform ?? 'windows-x86_64-cuda13'}
          selectedRoot={workbench.selectedRoot}
        />
      </main>
    </div>
  );
}
