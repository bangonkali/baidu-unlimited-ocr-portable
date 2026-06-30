import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

import {
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
  useSettings,
  useStartIngest,
  useStatus,
  useUpdateSettings,
} from '../../api/hooks';
import { useRealtimeState } from '../../realtime/realtimeStore';
import type {
  DiagnosticsRouteSearch,
  ModelRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
} from '../../routeSearch';
import type { ActiveView } from '../../stores/workbenchStore';
import { setSelectedRoot, setTourRun, useWorkbenchState } from '../../stores/workbenchStore';
import { ActivityBar } from './ActivityBar';
import { GuidedTour } from './GuidedTour';
import { IngestToolbar } from './IngestToolbar';
import { useModelRouteActions } from './useModelRouteActions';
import { useWorkbenchIngestActions } from './useWorkbenchIngestActions';
import { useRouteSearchSync, useRouteSearchText } from './useWorkbenchRouteSync';
import styles from './WorkbenchPage.module.css';
import {
  CommandCenter,
  profileOptions,
  selectedModel,
  useAutoFollowLatestRegion,
  usePersistentProfile,
  useWorkbenchRefresh,
  WorkbenchFooter,
} from './WorkbenchPageSupport';
import { WorkbenchViewContent } from './WorkbenchViewContent';
import { viewPath } from './workbenchRouteState';

interface WorkbenchPageProps {
  activeView?: ActiveView;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  modelScope?: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  settingsSearch?: SettingsRouteSearch;
  workbenchSearch?: WorkbenchRouteSearch;
}

export function WorkbenchPage({
  activeView = 'workbench',
  diagnosticsSearch,
  modelScope = 'library',
  modelSearch,
  settingsSearch,
  workbenchSearch,
}: WorkbenchPageProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const workbench = useWorkbenchState();
  const realtime = useRealtimeState();
  const [searchText, setSearchText] = useState(workbenchSearch?.q ?? diagnosticsSearch?.q ?? '');
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const status = useStatus();
  const documents = useDocuments(debouncedSearch);
  const models = useModels();
  const runs = useIngestRuns();
  const logs = useLogs(220);
  const settings = useSettings();
  const regions = useDocumentRegions(workbench.selection.fileHash);
  const text = useDocumentText(workbench.selection.fileHash);
  const previewImages = useDocumentPreviewImages(workbench.selection.fileHash);
  const folderDialog = useOpenFolderDialog();
  const downloadModel = useDownloadModel();
  const cancelModelDownload = useCancelModelDownload();
  const selectModel = useSelectModel();
  const updateSettings = useUpdateSettings();
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
  useRouteSearchText(activeView, diagnosticsSearch, workbenchSearch, searchText, setSearchText);
  useAutoFollowLatestRegion(workbench, regions.data);
  useRouteSearchSync({ activeView, navigate, searchText, workbench, workbenchSearch });
  const changeProfile = usePersistentProfile(
    settings.data?.default_profile,
    workbench.selectedProfile,
    (profileId) => updateSettings.mutate({ default_profile: profileId }),
  );
  const refresh = useWorkbenchRefresh(queryClient);

  const { pickFolder, startScan } = useWorkbenchIngestActions({
    folderDialog,
    model,
    rootPath: workbench.selectedRoot,
    selectedProfile: workbench.selectedProfile,
    startIngest,
  });
  const navigateView = (view: ActiveView) => {
    void navigate({ to: viewPath(view) });
  };
  const { changeModelScope, updateModelRouteSearch } = useModelRouteActions(
    navigate,
    modelScope,
    modelSearch,
  );
  return (
    <div className={styles.shell}>
      <GuidedTour onViewChange={navigateView} run={workbench.tourRun} />
      <ActivityBar activeView={activeView} />
      <main className={styles.main}>
        <CommandCenter
          onSearchTextChange={setSearchText}
          onStartGuide={() => setTourRun(true)}
          searchText={searchText}
        />
        <IngestToolbar
          activeRun={activeRun}
          activeRunId={activeRunId}
          busy={startIngest.isPending || folderDialog.isPending}
          modelReady={modelReady}
          onPickFolder={pickFolder}
          onProfileChange={changeProfile}
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
          <WorkbenchViewContent
            activeView={activeView}
            documents={documents.data?.documents ?? []}
            logs={logs.data?.logs ?? []}
            model={model}
            modelBusy={
              downloadModel.isPending || cancelModelDownload.isPending || selectModel.isPending
            }
            modelScope={modelScope}
            modelSearch={modelSearch}
            models={models.data}
            onCancelModel={(modelId) => cancelModelDownload.mutate(modelId)}
            onDownloadModel={(modelId, force) => downloadModel.mutate({ force, modelId })}
            onModelChange={(modelId) => selectModel.mutate(modelId)}
            onModelRouteSearchChange={updateModelRouteSearch}
            onModelScopeChange={changeModelScope}
            onOpenModels={() => navigateView('models')}
            onPickFolder={pickFolder}
            onProfileChange={changeProfile}
            onRuntimeChange={(runtimeId) =>
              updateSettings.mutate({ selected_runtime_id: runtimeId })
            }
            onSelectModel={(modelId) => selectModel.mutate(modelId)}
            onStart={startScan}
            previewPages={previewImages.data?.pages ?? []}
            profiles={profiles}
            regions={regions.data?.boxes ?? []}
            rootPath={workbench.selectedRoot}
            runs={runs.data?.runs ?? []}
            selectedDocument={selectedDocument}
            selectedProfile={workbench.selectedProfile}
            settings={settings.data}
            settingsBusy={selectModel.isPending || updateSettings.isPending}
            settingsSearch={settingsSearch}
            status={status.data}
            textPages={text.data?.pages ?? []}
            workbench={workbench}
          />
        </div>
        <WorkbenchFooter
          accelerator={status.data?.accelerator}
          documentCount={documents.data?.documents.length ?? 0}
          logPath={status.data?.log_path}
          realtimeState={realtime.connectionState}
          runState={status.data?.state ?? 'offline'}
          runtimePlatform={status.data?.runtime_platform}
          selectedRoot={workbench.selectedRoot}
        />
      </main>
    </div>
  );
}
