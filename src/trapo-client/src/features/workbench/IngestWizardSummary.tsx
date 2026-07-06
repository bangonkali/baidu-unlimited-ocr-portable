import { Play, Square } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './IngestWizard.module.css';
import { isModelReady, modelStatusLabel } from './ingestWizardModels';

interface IngestWizardSummaryProps {
  active: boolean;
  canStart: boolean;
  embeddingAfterIngest: boolean;
  embeddingModel?: ModelAssetRecord;
  rootPath: string;
  selectedProfile: string;
  textIndexAfterIngest: boolean;
  ocrModel?: ModelAssetRecord;
  onStart: () => void;
  onStop: () => void;
}

export function IngestWizardSummary({
  active,
  canStart,
  embeddingAfterIngest,
  embeddingModel,
  ocrModel,
  onStart,
  onStop,
  rootPath,
  selectedProfile,
  textIndexAfterIngest,
}: IngestWizardSummaryProps) {
  return (
    <section className={styles.summary} aria-label="Ingest summary">
      <h2>Ready Check</h2>
      <p>
        {summaryMessage({ canStart, embeddingAfterIngest, embeddingModel, ocrModel, rootPath })}
      </p>
      <dl className={styles.summaryList}>
        <div>
          <dt>Folder</dt>
          <dd>{rootPath || 'Choose a folder'}</dd>
        </div>
        <div>
          <dt>OCR model</dt>
          <dd>
            {ocrModel?.display_name ?? 'Select a model'} · {modelStatusLabel(ocrModel)}
          </dd>
        </div>
        <div>
          <dt>Profile</dt>
          <dd>{selectedProfile}</dd>
        </div>
        <div>
          <dt>Text index</dt>
          <dd>{textIndexAfterIngest ? 'After ingest' : 'Skip'}</dd>
        </div>
        <div>
          <dt>Embedding</dt>
          <dd>
            {embeddingAfterIngest
              ? `${embeddingModel?.display_name ?? 'Select a model'} · ${modelStatusLabel(embeddingModel)}`
              : 'Optional'}
          </dd>
        </div>
      </dl>
      <div className={styles.actions}>
        <button
          className={styles.primaryButton}
          data-tour="start"
          disabled={!canStart}
          onClick={onStart}
          type="button"
        >
          <Play size={15} />
          Start Ingest
        </button>
        <button className={styles.button} disabled={!active} onClick={onStop} type="button">
          <Square size={15} />
          Stop Active Run
        </button>
      </div>
    </section>
  );
}

function summaryMessage(args: {
  canStart: boolean;
  embeddingAfterIngest: boolean;
  embeddingModel?: ModelAssetRecord;
  ocrModel?: ModelAssetRecord;
  rootPath: string;
}) {
  if (args.canStart) {
    return 'Everything needed for this workflow is ready.';
  }
  if (!args.rootPath.trim()) {
    return 'Choose a folder before starting.';
  }
  if (!isModelReady(args.ocrModel)) {
    return 'Download or select a ready OCR model.';
  }
  if (args.embeddingAfterIngest && !isModelReady(args.embeddingModel)) {
    return 'Download an embedding model or turn off embedding generation.';
  }
  return 'A task is already running or the wizard is waiting for data.';
}
