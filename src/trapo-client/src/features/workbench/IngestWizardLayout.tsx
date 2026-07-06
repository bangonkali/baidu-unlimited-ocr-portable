import { Workflow } from 'lucide-react';

import type {
  IngestRunRecord,
  ModelAssetRecord,
  OcrProfileRecord,
  StatusPayload,
} from '../../api/types';
import styles from './IngestWizard.module.css';
import { IngestWizardFlow } from './IngestWizardFlow';
import { IngestWizardRail } from './IngestWizardRail';
import type { WizardStepRecord } from './IngestWizardStepper';

interface IngestWizardLayoutProps {
  active: boolean;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  busy?: boolean;
  canRunPostStep: boolean;
  canStart: boolean;
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
  runs: IngestRunRecord[];
  selectedEmbeddingModel?: ModelAssetRecord;
  selectedOcrModel?: ModelAssetRecord;
  selectedProfile: string;
  selectedRunId: string;
  status?: StatusPayload;
  steps: WizardStepRecord[];
  textIndexAfterIngest: boolean;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onEmbeddingAfterIngestChange: (value: boolean) => void;
  onEmbeddingModelChange: (modelId: string) => void;
  onGenerateEmbedding: () => void;
  onModelChange: (modelId: string) => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onReprocessChange: (value: boolean) => void;
  onRootPathChange: (value: string) => void;
  onRunChange: (runId: string) => void;
  onStart: () => void;
  onStartTextIndex: () => void;
  onStop: () => void;
  onTextIndexAfterIngestChange: (value: boolean) => void;
}

export function IngestWizardLayout(props: IngestWizardLayoutProps) {
  return (
    <section className={styles.panel} aria-label="Start ingest wizard">
      <header className={styles.header}>
        <span className={styles.title}>
          <Workflow size={16} />
          Start Ingest
        </span>
        <span className={styles.inlineMeta}>Text Index · Generate Embedding</span>
      </header>
      <div className={styles.body}>
        <IngestWizardFlow
          {...props}
          onTextIndexAfterIngestChange={props.onTextIndexAfterIngestChange}
        />
        <IngestWizardRail
          {...props}
          embeddingModel={props.selectedEmbeddingModel}
          ocrModel={props.selectedOcrModel}
          selectedEmbeddingModelId={props.selectedEmbeddingModel?.model_id ?? ''}
          onStartTextIndex={props.onStartTextIndex}
        />
      </div>
    </section>
  );
}
