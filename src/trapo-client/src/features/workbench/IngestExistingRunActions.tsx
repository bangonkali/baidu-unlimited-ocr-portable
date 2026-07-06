import { BrainCircuit, FileText } from 'lucide-react';

import type { IngestRunRecord, ModelAssetRecord } from '../../api/types';
import { shortRunLabel } from './IngestStartPanelParts';
import styles from './IngestWizard.module.css';
import { isModelReady } from './ingestWizardModels';

interface IngestExistingRunActionsProps {
  busy?: boolean;
  canRunPostStep: boolean;
  embeddingModels: ModelAssetRecord[];
  selectedEmbeddingModelId: string;
  selectedRunId: string;
  runs: IngestRunRecord[];
  onEmbeddingModelChange: (modelId: string) => void;
  onGenerateEmbedding: () => void;
  onRunChange: (runId: string) => void;
  onStartTextIndex: () => void;
}

export function IngestExistingRunActions({
  busy,
  canRunPostStep,
  embeddingModels,
  onEmbeddingModelChange,
  onGenerateEmbedding,
  onRunChange,
  onStartTextIndex,
  runs,
  selectedEmbeddingModelId,
  selectedRunId,
}: IngestExistingRunActionsProps) {
  const selectedEmbeddingModel = embeddingModels.find(
    (model) => model.model_id === selectedEmbeddingModelId,
  );
  const canGenerateEmbedding = canRunPostStep && isModelReady(selectedEmbeddingModel);
  return (
    <section className={styles.summary}>
      <h2>Existing Run</h2>
      <p>Run indexing or embeddings for OCR output that already completed.</p>
      <label className={styles.field}>
        <span>Source run</span>
        <select
          disabled={busy || runs.length === 0}
          onChange={(event) => onRunChange(event.target.value)}
          value={selectedRunId}
        >
          <option value="">Select an ingest run</option>
          {runs.map((run) => (
            <option key={run.run_id} value={run.run_id}>
              {shortRunLabel(run)}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.field}>
        <span>Embedding model</span>
        <select
          disabled={busy || embeddingModels.length === 0}
          onChange={(event) => onEmbeddingModelChange(event.target.value)}
          value={selectedEmbeddingModelId}
        >
          <option value="">Select an embedding model</option>
          {embeddingModels.map((model) => (
            <option key={model.model_id} value={model.model_id}>
              {model.display_name} - {model.status}
            </option>
          ))}
        </select>
      </label>
      <div className={styles.actions}>
        <button
          className={styles.button}
          disabled={!canRunPostStep}
          onClick={onStartTextIndex}
          type="button"
        >
          <FileText size={15} />
          Text Index
        </button>
        <button
          className={styles.button}
          disabled={!canGenerateEmbedding}
          onClick={onGenerateEmbedding}
          type="button"
        >
          <BrainCircuit size={15} />
          Generate Embedding
        </button>
      </div>
    </section>
  );
}
