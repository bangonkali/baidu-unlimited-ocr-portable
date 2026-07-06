import type { ModelAssetRecord, OcrProfileRecord } from '../../api/types';
import styles from './IngestWizard.module.css';
import { IngestWizardModelChoice } from './IngestWizardModelChoice';
import { IngestWizardPipelineOptions } from './IngestWizardPipelineOptions';
import { IngestWizardSource } from './IngestWizardSource';
import type { WizardStepRecord } from './IngestWizardStepper';
import { IngestWizardStepper } from './IngestWizardStepper';

interface IngestWizardFlowProps {
  busy?: boolean;
  embeddingAfterIngest: boolean;
  embeddingModelOptions: ModelAssetRecord[];
  embeddingReady: boolean;
  folderDialogError?: string;
  ocrModelOptions: ModelAssetRecord[];
  profiles: OcrProfileRecord[];
  recommendedEmbedding?: ModelAssetRecord;
  recommendedOcr?: ModelAssetRecord;
  reprocess: boolean;
  rootPath: string;
  selectedEmbeddingModel?: ModelAssetRecord;
  selectedOcrModel?: ModelAssetRecord;
  selectedProfile: string;
  steps: WizardStepRecord[];
  textIndexAfterIngest: boolean;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onEmbeddingAfterIngestChange: (value: boolean) => void;
  onEmbeddingModelChange: (modelId: string) => void;
  onModelChange: (modelId: string) => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onReprocessChange: (value: boolean) => void;
  onRootPathChange: (value: string) => void;
  onTextIndexAfterIngestChange: (value: boolean) => void;
}

export function IngestWizardFlow(props: IngestWizardFlowProps) {
  return (
    <div className={styles.wizard}>
      <IngestWizardStepper steps={props.steps} />
      <IngestWizardSource
        busy={props.busy}
        folderDialogError={props.folderDialogError}
        rootPath={props.rootPath}
        selectedProfile={props.selectedProfile}
        profiles={props.profiles}
        reprocess={props.reprocess}
        onPickFolder={props.onPickFolder}
        onProfileChange={props.onProfileChange}
        onReprocessChange={props.onReprocessChange}
        onRootPathChange={props.onRootPathChange}
      />
      <IngestWizardModelChoice
        busy={props.busy}
        description="Use a downloaded Unlimited OCR model, or confirm a download before starting."
        label="OCR model"
        models={props.ocrModelOptions}
        recommendedModelId={props.recommendedOcr?.model_id}
        selectedModelId={props.selectedOcrModel?.model_id ?? ''}
        onCancelModel={props.onCancelModel}
        onChange={props.onModelChange}
        onDownloadModel={props.onDownloadModel}
      />
      <IngestWizardPipelineOptions
        embeddingAfterIngest={props.embeddingAfterIngest}
        embeddingReady={props.embeddingReady}
        textIndexAfterIngest={props.textIndexAfterIngest}
        onEmbeddingAfterIngestChange={props.onEmbeddingAfterIngestChange}
        onTextIndexAfterIngestChange={props.onTextIndexAfterIngestChange}
      />
      <IngestWizardModelChoice
        busy={props.busy}
        description="Nomic Q4 is the recommended small first model for semantic search."
        label="Embedding model"
        models={props.embeddingModelOptions}
        recommendedModelId={props.recommendedEmbedding?.model_id}
        selectedModelId={props.selectedEmbeddingModel?.model_id ?? ''}
        onCancelModel={props.onCancelModel}
        onChange={props.onEmbeddingModelChange}
        onDownloadModel={props.onDownloadModel}
      />
    </div>
  );
}
