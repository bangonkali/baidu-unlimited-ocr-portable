import { Workflow } from 'lucide-react';
import { useEffect, useState } from 'react';

import type {
  IngestRunRecord,
  ModelAssetRecord,
  ModelsPayload,
  OcrProfileRecord,
  StatusPayload,
} from '../../api/types';
import type { IngestRouteSearch } from '../../routeSearch';
import styles from './IngestStartPanel.module.css';
import { isIngestBusy, latestCompletedRunId, RunStatus } from './IngestStartPanelParts';
import {
  FolderSection,
  OcrConfigurationSection,
  RunTaskSection,
  StartStopActions,
} from './IngestTaskSections';

interface IngestStartOptions {
  embeddingAfterIngest?: boolean;
  embeddingDimension?: number;
  embeddingModelId?: string;
  reprocess?: boolean;
  textIndexAfterIngest?: boolean;
}

interface IngestStartPanelProps {
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  busy?: boolean;
  folderDialogError?: string;
  ingestSearch?: IngestRouteSearch;
  model?: ModelAssetRecord;
  models?: ModelsPayload;
  profiles: OcrProfileRecord[];
  rootPath: string;
  selectedProfile: string;
  status?: StatusPayload;
  onModelChange: (modelId: string) => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onRootPathChange: (value: string) => void;
  onStartTextIndex: (sourceRunId: string) => void;
  onGenerateEmbedding: (input: {
    dimension?: number;
    modelId: string;
    sourceRunId: string;
  }) => void;
  onStart: (options?: IngestStartOptions) => void;
  onStop: () => void;
  runs: IngestRunRecord[];
}

export function IngestStartPanel(props: IngestStartPanelProps) {
  const [reprocess, setReprocess] = useState(props.ingestSearch?.reprocess ?? false);
  const [textIndexAfterIngest, setTextIndexAfterIngest] = useState(
    props.ingestSearch?.index ?? false,
  );
  const [embeddingAfterIngest, setEmbeddingAfterIngest] = useState(
    props.ingestSearch?.embed ?? false,
  );
  const latestRunId = latestCompletedRunId(props.runs);
  const [selectedRunId, setSelectedRunId] = useState(latestRunId ?? '');
  const active = isIngestBusy(props.status, props.activeRun);
  const models = props.models?.models ?? [];
  const ocrModels = models.filter((model) => model.model_kind !== 'embedding');
  const embeddingModels = models.filter((model) => model.model_kind === 'embedding');
  const downloadedEmbeddingModels = embeddingModels.filter(
    (model) => model.status === 'downloaded',
  );
  const defaultEmbeddingModel =
    downloadedEmbeddingModels.find(
      (model) => model.model_id === props.ingestSearch?.embedding_model,
    ) ?? downloadedEmbeddingModels[0];
  const [selectedEmbeddingModelId, setSelectedEmbeddingModelId] = useState(
    defaultEmbeddingModel?.model_id ?? '',
  );
  const selectedEmbeddingModel =
    embeddingModels.find((model) => model.model_id === selectedEmbeddingModelId) ??
    defaultEmbeddingModel;
  const modelReady = props.model?.status === 'downloaded' && props.model.model_kind !== 'embedding';
  const embeddingReady = Boolean(
    selectedEmbeddingModelId && selectedEmbeddingModel?.status === 'downloaded',
  );
  const canStart =
    Boolean(props.rootPath.trim()) &&
    modelReady &&
    (!embeddingAfterIngest || embeddingReady) &&
    !props.busy &&
    !active &&
    props.profiles.length > 0;
  const canRunPostStep = Boolean(selectedRunId) && !props.busy && !active;
  const canGenerateEmbedding = canRunPostStep && embeddingReady;

  useEffect(() => {
    setReprocess(props.ingestSearch?.reprocess ?? false);
  }, [props.ingestSearch?.reprocess]);
  useEffect(() => {
    setTextIndexAfterIngest(props.ingestSearch?.index ?? false);
  }, [props.ingestSearch?.index]);
  useEffect(() => {
    setEmbeddingAfterIngest(props.ingestSearch?.embed ?? false);
  }, [props.ingestSearch?.embed]);
  useEffect(() => {
    if (latestRunId && !selectedRunId) {
      setSelectedRunId(latestRunId);
    }
  }, [latestRunId, selectedRunId]);
  useEffect(() => {
    if (defaultEmbeddingModel && !selectedEmbeddingModelId) {
      setSelectedEmbeddingModelId(defaultEmbeddingModel.model_id);
    }
  }, [defaultEmbeddingModel, selectedEmbeddingModelId]);

  const embeddingDimension = selectedEmbeddingModel?.embedding_dimension ?? undefined;
  const startIngest = () =>
    props.onStart({
      embeddingAfterIngest,
      embeddingDimension,
      embeddingModelId: selectedEmbeddingModelId || undefined,
      reprocess,
      textIndexAfterIngest,
    });
  const generateEmbedding = () =>
    props.onGenerateEmbedding({
      dimension: embeddingDimension,
      modelId: selectedEmbeddingModelId,
      sourceRunId: selectedRunId,
    });

  return (
    <section className={styles.panel} aria-label="Start ingest">
      <header className={styles.header}>
        <Workflow size={16} />
        <span>Start Ingest</span>
        <span className={styles.workflowStep}>Text Index</span>
        <span className={styles.workflowStep}>Generate Embedding</span>
      </header>
      <div className={styles.body}>
        <div className={styles.form}>
          <FolderSection
            busy={props.busy}
            folderDialogError={props.folderDialogError}
            rootPath={props.rootPath}
            onPickFolder={props.onPickFolder}
            onRootPathChange={props.onRootPathChange}
          />
          <OcrConfigurationSection
            busy={props.busy}
            embeddingAfterIngest={embeddingAfterIngest}
            embeddingModels={downloadedEmbeddingModels}
            modelValue={props.model?.model_id ?? props.models?.selected_model_id ?? ''}
            ocrModels={ocrModels}
            profiles={props.profiles}
            reprocess={reprocess}
            selectedEmbeddingModelId={selectedEmbeddingModelId}
            selectedProfile={props.selectedProfile}
            textIndexAfterIngest={textIndexAfterIngest}
            onEmbeddingAfterIngestChange={setEmbeddingAfterIngest}
            onEmbeddingModelChange={setSelectedEmbeddingModelId}
            onModelChange={props.onModelChange}
            onProfileChange={props.onProfileChange}
            onReprocessChange={setReprocess}
            onTextIndexAfterIngestChange={setTextIndexAfterIngest}
          />
          <RunTaskSection
            busy={props.busy}
            canGenerateEmbedding={canGenerateEmbedding}
            canRunPostStep={canRunPostStep}
            embeddingModels={downloadedEmbeddingModels}
            runs={props.runs}
            selectedEmbeddingModelId={selectedEmbeddingModelId}
            selectedRunId={selectedRunId}
            onEmbeddingModelChange={setSelectedEmbeddingModelId}
            onGenerateEmbedding={generateEmbedding}
            onRunChange={setSelectedRunId}
            onStartTextIndex={() => props.onStartTextIndex(selectedRunId)}
          />
          <StartStopActions
            active={active}
            canStart={canStart}
            onStart={startIngest}
            onStop={props.onStop}
          />
        </div>
        <RunStatus
          activeRun={props.activeRun}
          activeRunId={props.activeRunId}
          status={props.status}
        />
      </div>
    </section>
  );
}

export { isIngestBusy };
