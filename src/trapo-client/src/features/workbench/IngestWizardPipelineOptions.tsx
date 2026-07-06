import { BrainCircuit, FileText } from 'lucide-react';

import styles from './IngestWizard.module.css';

interface IngestWizardPipelineOptionsProps {
  embeddingAfterIngest: boolean;
  embeddingReady: boolean;
  textIndexAfterIngest: boolean;
  onEmbeddingAfterIngestChange: (value: boolean) => void;
  onTextIndexAfterIngestChange: (value: boolean) => void;
}

export function IngestWizardPipelineOptions({
  embeddingAfterIngest,
  embeddingReady,
  onEmbeddingAfterIngestChange,
  onTextIndexAfterIngestChange,
  textIndexAfterIngest,
}: IngestWizardPipelineOptionsProps) {
  return (
    <section className={styles.card}>
      <h2>Pipeline</h2>
      <p>Choose what should happen after OCR finishes.</p>
      <label className={styles.toggleRow}>
        <input
          checked={textIndexAfterIngest}
          onChange={(event) => onTextIndexAfterIngestChange(event.target.checked)}
          type="checkbox"
        />
        <span className={styles.toggleText}>
          <strong>
            <FileText size={14} /> Text Index
          </strong>
          <span className={styles.hint}>Build the DuckDB full-text index for keyword search.</span>
        </span>
      </label>
      <label className={styles.toggleRow}>
        <input
          checked={embeddingAfterIngest}
          onChange={(event) => onEmbeddingAfterIngestChange(event.target.checked)}
          type="checkbox"
        />
        <span className={styles.toggleText}>
          <strong>
            <BrainCircuit size={14} /> Generate Embedding
          </strong>
          <span className={styles.hint}>
            Add semantic search vectors
            {embeddingReady ? ' with the selected model.' : ' after a model is ready.'}
          </span>
        </span>
      </label>
    </section>
  );
}
