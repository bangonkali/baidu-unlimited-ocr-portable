import { useMemo, useState } from 'react';

import type {
  IngestEnginePresetRecord,
  IngestEngineSelection,
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
import { useEnginePlanState } from './useEnginePlanState';
import { useIngestWizardStateSync } from './useIngestWizardStateSync';

interface IngestStartOptions {
  embeddingAfterIngest?: boolean;
  embeddingDimension?: number;
  embeddingModelId?: string;
  engines?: IngestEngineSelection[];
  reprocess?: boolean;
  textIndexAfterIngest?: boolean;
}

interface IngestStartPanelProps {
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  busy?: boolean;
  enginePresets: IngestEnginePresetRecord[];
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
  const wizardState = useIngestWizardLocalState(props);
  const latestRunId = latestCompletedRunId(props.runs);
  const [selectedRunId, setSelectedRunId] = useState(latestRunId ?? '');
  const restartRun = props.runs.find((run) => run.run_id === props.ingestSearch?.restart);
  const enginePlanState = useEnginePlanState({
    enginePresets: props.enginePresets,
    presetIds: props.ingestSearch?.engines,
    restartRun,
    restartRunId: props.ingestSearch?.restart,
    runsReady: props.runs.length > 0,
    models: wizardState.allModels,
    selectedProfile: props.selectedProfile,
    selectedRuntimeId: props.ingestSearch?.runtime,
  });
  const active = isIngestBusy(props.status, props.activeRun);
  const modelReady = isModelReady(wizardState.selectedOcrModel);
  const embeddingReady = isModelReady(wizardState.selectedEmbeddingModel);
  const hasEngineCatalog = props.enginePresets.length > 0;
  const canStart =
    Boolean(props.rootPath.trim()) &&
    (hasEngineCatalog ? enginePlanState.planReady : modelReady) &&
    (!wizardState.embeddingAfterIngest || embeddingReady) &&
    !props.busy &&
    !active &&
    props.profiles.length > 0;
  const canRunPostStep = Boolean(selectedRunId) && !props.busy && !active;

  useIngestWizardStateSync({
    ingestSearch: props.ingestSearch,
    latestRunId,
    recommendedEmbedding: wizardState.recommendedEmbedding,
    selectedEmbeddingModelId: wizardState.selectedEmbeddingModelId,
    selectedRunId,
    setEmbeddingAfterIngest: wizardState.setEmbeddingAfterIngest,
    setReprocess: wizardState.setReprocess,
    setSelectedEmbeddingModelId: wizardState.setSelectedEmbeddingModelId,
    setSelectedRunId,
    setTextIndexAfterIngest: wizardState.setTextIndexAfterIngest,
  });

  const startIngest = () =>
    props.onStart(
      buildIngestWizardStartOptions({
        embeddingAfterIngest: wizardState.embeddingAfterIngest,
        enginePlan: enginePlanState.enginePlan,
        enginePresets: props.enginePresets,
        reprocess: wizardState.reprocess,
        selectedEmbeddingModel: wizardState.selectedEmbeddingModel,
        selectedEmbeddingModelId: wizardState.selectedEmbeddingModelId,
        textIndexAfterIngest: wizardState.textIndexAfterIngest,
      }),
    );
  const generateEmbedding = () =>
    props.onGenerateEmbedding({
      dimension: wizardState.selectedEmbeddingModel?.embedding_dimension ?? undefined,
      modelId: wizardState.selectedEmbeddingModelId,
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
      embeddingAfterIngest={wizardState.embeddingAfterIngest}
      embeddingModelOptions={wizardState.embeddingModelOptions}
      embeddingReady={embeddingReady}
      folderDialogError={props.folderDialogError}
      ocrModelOptions={wizardState.ocrModelOptions}
      enginePlan={enginePlanState.enginePlan}
      enginePlanIssue={enginePlanState.planIssue}
      enginePresets={props.enginePresets}
      profiles={props.profiles}
      recommendedEmbedding={wizardState.recommendedEmbedding}
      recommendedOcr={wizardState.recommendedOcr}
      reprocess={wizardState.reprocess}
      rootPath={props.rootPath}
      runs={props.runs}
      selectedEmbeddingModel={wizardState.selectedEmbeddingModel}
      selectedOcrModel={wizardState.selectedOcrModel}
      selectedProfile={props.selectedProfile}
      selectedRuntimeId={props.ingestSearch?.runtime}
      selectedRunId={selectedRunId}
      status={props.status}
      steps={ingestWizardSteps({
        canStart,
        embeddingAfterIngest: wizardState.embeddingAfterIngest,
        embeddingReady,
        modelReady,
        planIssue: enginePlanState.planIssue,
        planReady: enginePlanState.planReady,
        rootReady: Boolean(props.rootPath.trim()),
        textIndexAfterIngest: wizardState.textIndexAfterIngest,
      })}
      textIndexAfterIngest={wizardState.textIndexAfterIngest}
      onCancelModel={props.onCancelModel}
      onDownloadModel={props.onDownloadModel}
      onEmbeddingAfterIngestChange={wizardState.setEmbeddingAfterIngest}
      onEmbeddingModelChange={wizardState.setSelectedEmbeddingModelId}
      onGenerateEmbedding={generateEmbedding}
      onModelChange={props.onModelChange}
      onPlanChange={enginePlanState.setEnginePlan}
      onPickFolder={props.onPickFolder}
      onProfileChange={props.onProfileChange}
      onReprocessChange={wizardState.setReprocess}
      onRootPathChange={props.onRootPathChange}
      onRunChange={setSelectedRunId}
      onStart={startIngest}
      onStartTextIndex={() => props.onStartTextIndex(selectedRunId)}
      onStop={props.onStop}
      onTextIndexAfterIngestChange={wizardState.setTextIndexAfterIngest}
    />
  );
}

function useIngestWizardLocalState(props: IngestStartPanelProps) {
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
  const [selectedEmbeddingModelId, setSelectedEmbeddingModelId] = useState(
    recommendedEmbedding?.model_id ?? '',
  );
  const selectedEmbeddingModel =
    embeddingModelOptions.find((model) => model.model_id === selectedEmbeddingModelId) ??
    recommendedEmbedding;

  return {
    allModels,
    embeddingAfterIngest,
    embeddingModelOptions,
    ocrModelOptions,
    recommendedEmbedding,
    recommendedOcr,
    reprocess,
    selectedEmbeddingModel,
    selectedEmbeddingModelId,
    selectedOcrModel,
    setEmbeddingAfterIngest,
    setReprocess,
    setSelectedEmbeddingModelId,
    setTextIndexAfterIngest,
    textIndexAfterIngest,
  };
}

export { isIngestBusy };
