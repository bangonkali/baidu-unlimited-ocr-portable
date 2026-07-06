import { useMemo, useState } from 'react';

import type {
  IngestRunRecord,
  ModelAssetRecord,
  ModelsPayload,
  OcrProfileRecord,
  StatusPayload,
} from '../../api/types';
import type { IngestRouteSearch } from '../../routeSearch';
import { isIngestBusy, latestCompletedRunId } from './IngestStartPanelParts';
import { IngestWizardLayout } from './IngestWizardLayout';
import {
  embeddingModels,
  isModelReady,
  ocrModels,
  recommendedEmbeddingModel,
  recommendedOcrModel,
} from './ingestWizardModels';
import { buildIngestWizardStartOptions } from './ingestWizardStart';
import { ingestWizardSteps } from './ingestWizardSteps';
import { useIngestWizardStateSync } from './useIngestWizardStateSync';

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
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
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
  const allModels = useMemo(() => props.models?.models ?? [], [props.models?.models]);
  const ocrModelOptions = ocrModels(allModels);
  const embeddingModelOptions = embeddingModels(allModels);
  const recommendedOcr = recommendedOcrModel(
    allModels,
    props.ingestSearch?.model,
    props.models?.selected_model_id,
  );
  const recommendedEmbedding = recommendedEmbeddingModel(
    allModels,
    props.ingestSearch?.embedding_model,
  );
  const selectedOcrModel =
    ocrModelOptions.find((model) => model.model_id === props.model?.model_id) ?? recommendedOcr;
  const [reprocess, setReprocess] = useState(props.ingestSearch?.reprocess ?? false);
  const [textIndexAfterIngest, setTextIndexAfterIngest] = useState(
    props.ingestSearch?.index ?? true,
  );
  const [embeddingAfterIngest, setEmbeddingAfterIngest] = useState(
    props.ingestSearch?.embed ?? isModelReady(recommendedEmbedding),
  );
  const latestRunId = latestCompletedRunId(props.runs);
  const [selectedRunId, setSelectedRunId] = useState(latestRunId ?? '');
  const [selectedEmbeddingModelId, setSelectedEmbeddingModelId] = useState(
    recommendedEmbedding?.model_id ?? '',
  );
  const selectedEmbeddingModel =
    embeddingModelOptions.find((model) => model.model_id === selectedEmbeddingModelId) ??
    recommendedEmbedding;
  const active = isIngestBusy(props.status, props.activeRun);
  const modelReady = isModelReady(selectedOcrModel);
  const embeddingReady = isModelReady(selectedEmbeddingModel);
  const canStart =
    Boolean(props.rootPath.trim()) &&
    modelReady &&
    (!embeddingAfterIngest || embeddingReady) &&
    !props.busy &&
    !active &&
    props.profiles.length > 0;
  const canRunPostStep = Boolean(selectedRunId) && !props.busy && !active;

  useIngestWizardStateSync({
    ingestSearch: props.ingestSearch,
    latestRunId,
    recommendedEmbedding,
    selectedEmbeddingModelId,
    selectedRunId,
    setEmbeddingAfterIngest,
    setReprocess,
    setSelectedEmbeddingModelId,
    setSelectedRunId,
    setTextIndexAfterIngest,
  });

  const startIngest = () =>
    props.onStart(
      buildIngestWizardStartOptions({
        embeddingAfterIngest,
        reprocess,
        selectedEmbeddingModel,
        selectedEmbeddingModelId,
        textIndexAfterIngest,
      }),
    );
  const generateEmbedding = () =>
    props.onGenerateEmbedding({
      dimension: selectedEmbeddingModel?.embedding_dimension ?? undefined,
      modelId: selectedEmbeddingModelId,
      sourceRunId: selectedRunId,
    });

  return (
    <IngestWizardLayout
      active={active}
      activeRun={props.activeRun}
      activeRunId={props.activeRunId}
      busy={props.busy}
      canRunPostStep={canRunPostStep}
      canStart={canStart}
      embeddingAfterIngest={embeddingAfterIngest}
      embeddingModelOptions={embeddingModelOptions}
      embeddingReady={embeddingReady}
      folderDialogError={props.folderDialogError}
      ocrModelOptions={ocrModelOptions}
      profiles={props.profiles}
      recommendedEmbedding={recommendedEmbedding}
      recommendedOcr={recommendedOcr}
      reprocess={reprocess}
      rootPath={props.rootPath}
      runs={props.runs}
      selectedEmbeddingModel={selectedEmbeddingModel}
      selectedOcrModel={selectedOcrModel}
      selectedProfile={props.selectedProfile}
      selectedRunId={selectedRunId}
      status={props.status}
      steps={ingestWizardSteps({
        canStart,
        embeddingAfterIngest,
        embeddingReady,
        modelReady,
        rootReady: Boolean(props.rootPath.trim()),
        textIndexAfterIngest,
      })}
      textIndexAfterIngest={textIndexAfterIngest}
      onCancelModel={props.onCancelModel}
      onDownloadModel={props.onDownloadModel}
      onEmbeddingAfterIngestChange={setEmbeddingAfterIngest}
      onEmbeddingModelChange={setSelectedEmbeddingModelId}
      onGenerateEmbedding={generateEmbedding}
      onModelChange={props.onModelChange}
      onPickFolder={props.onPickFolder}
      onProfileChange={props.onProfileChange}
      onReprocessChange={setReprocess}
      onRootPathChange={props.onRootPathChange}
      onRunChange={setSelectedRunId}
      onStart={startIngest}
      onStartTextIndex={() => props.onStartTextIndex(selectedRunId)}
      onStop={props.onStop}
      onTextIndexAfterIngestChange={setTextIndexAfterIngest}
    />
  );
}

export { isIngestBusy };
