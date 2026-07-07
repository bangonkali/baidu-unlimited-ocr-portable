import type { IngestRunRecord, ModelAssetRecord, StatusPayload } from '../../api/types';
import { IngestExistingRunActions } from './IngestExistingRunActions';
import { RunStatus } from './IngestStartPanelParts';
import styles from './IngestWizard.module.css';
import { IngestWizardSummary } from './IngestWizardSummary';

interface IngestWizardRailProps {
  active: boolean;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  busy?: boolean;
  canRunPostStep: boolean;
  canStart: boolean;
  embeddingAfterIngest: boolean;
  embeddingModel?: ModelAssetRecord;
  embeddingModelOptions: ModelAssetRecord[];
  enginePlanCount: number;
  enginePlanIssue?: string;
  ocrModel?: ModelAssetRecord;
  rootPath: string;
  runs: IngestRunRecord[];
  selectedEmbeddingModelId: string;
  selectedProfile: string;
  selectedRunId: string;
  status?: StatusPayload;
  textIndexAfterIngest: boolean;
  onEmbeddingModelChange: (modelId: string) => void;
  onGenerateEmbedding: () => void;
  onRunChange: (runId: string) => void;
  onStart: () => void;
  onStartTextIndex: () => void;
  onStop: () => void;
}

export function IngestWizardRail(props: IngestWizardRailProps) {
  return (
    <aside className={styles.rail}>
      <IngestWizardSummary
        active={props.active}
        canStart={props.canStart}
        embeddingAfterIngest={props.embeddingAfterIngest}
        embeddingModel={props.embeddingModel}
        enginePlanCount={props.enginePlanCount}
        enginePlanIssue={props.enginePlanIssue}
        ocrModel={props.ocrModel}
        rootPath={props.rootPath}
        selectedProfile={props.selectedProfile}
        textIndexAfterIngest={props.textIndexAfterIngest}
        onStart={props.onStart}
        onStop={props.onStop}
      />
      <IngestExistingRunActions
        busy={props.busy}
        canRunPostStep={props.canRunPostStep}
        embeddingModels={props.embeddingModelOptions}
        selectedEmbeddingModelId={props.selectedEmbeddingModelId}
        selectedRunId={props.selectedRunId}
        runs={props.runs}
        onEmbeddingModelChange={props.onEmbeddingModelChange}
        onGenerateEmbedding={props.onGenerateEmbedding}
        onRunChange={props.onRunChange}
        onStartTextIndex={props.onStartTextIndex}
      />
      <RunStatus
        activeRun={props.activeRun}
        activeRunId={props.activeRunId}
        status={props.status}
      />
    </aside>
  );
}
