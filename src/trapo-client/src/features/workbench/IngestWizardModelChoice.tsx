import { Download, RotateCcw, XCircle } from 'lucide-react';
import { useState } from 'react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './IngestWizard.module.css';
import {
  formatModelBytes,
  isModelDownloading,
  isModelReady,
  modelRequiredBytes,
  modelStatusLabel,
} from './ingestWizardModels';

interface IngestWizardModelChoiceProps {
  busy?: boolean;
  description: string;
  label: string;
  models: ModelAssetRecord[];
  recommendedModelId?: string;
  selectedModelId: string;
  onCancelModel: (modelId: string) => void;
  onChange: (modelId: string) => void;
  onDownloadModel: (modelId: string) => void;
}

export function IngestWizardModelChoice({
  busy,
  description,
  label,
  models,
  onCancelModel,
  onChange,
  onDownloadModel,
  recommendedModelId,
  selectedModelId,
}: IngestWizardModelChoiceProps) {
  const [confirmingDownloadId, setConfirmingDownloadId] = useState('');
  const selected = models.find((model) => model.model_id === selectedModelId);
  const download = (modelId: string) => {
    if (confirmingDownloadId !== modelId) {
      setConfirmingDownloadId(modelId);
      return;
    }
    setConfirmingDownloadId('');
    onDownloadModel(modelId);
  };

  return (
    <section className={styles.card} data-emphasis={!isModelReady(selected)}>
      <h2>{label}</h2>
      <p>{description}</p>
      <label className={styles.field}>
        <span>{label}</span>
        <select
          disabled={busy || models.length === 0}
          onChange={(event) => {
            setConfirmingDownloadId('');
            onChange(event.target.value);
          }}
          value={selectedModelId}
        >
          <option value="">Select a compatible model</option>
          {models.map((model) => (
            <option key={model.model_id} value={model.model_id}>
              {model.display_name}
              {model.model_id === recommendedModelId ? ' - recommended' : ''}
              {' - '}
              {modelStatusLabel(model)}
            </option>
          ))}
        </select>
      </label>
      <ModelReadiness
        busy={busy}
        confirming={confirmingDownloadId === selected?.model_id}
        model={selected}
        onCancelModel={onCancelModel}
        onDownload={download}
      />
    </section>
  );
}

function ModelReadiness({
  busy,
  confirming,
  model,
  onCancelModel,
  onDownload,
}: {
  busy?: boolean;
  confirming: boolean;
  model?: ModelAssetRecord;
  onCancelModel: (modelId: string) => void;
  onDownload: (modelId: string) => void;
}) {
  if (!model) {
    return <p className={styles.error}>No compatible model is available in the catalog.</p>;
  }
  const active = isModelDownloading(model);
  const ready = isModelReady(model);
  return (
    <div className={styles.modelDetail}>
      <div className={styles.statusLine}>
        <strong className={styles.modelName}>{model.display_name}</strong>
        <span className={styles.statusBadge} data-active={active} data-ready={ready}>
          {modelStatusLabel(model)}
        </span>
      </div>
      <span className={styles.meta}>
        {model.provider_name ?? model.repo_id ?? 'model provider'} · {model.quality ?? 'model'} ·{' '}
        {model.hardware_tier ?? 'local runtime'} · {formatModelBytes(modelRequiredBytes(model))}
      </span>
      <span className={styles.hint}>{model.notes ?? 'Download this model to use this step.'}</span>
      <div className={styles.actions}>
        {active ? (
          <button
            className={styles.button}
            disabled={busy}
            onClick={() => onCancelModel(model.model_id)}
            type="button"
          >
            <XCircle size={15} />
            Cancel download
          </button>
        ) : null}
        {!ready && !active ? (
          <button
            className={confirming ? styles.primaryButton : styles.button}
            disabled={busy}
            onClick={() => onDownload(model.model_id)}
            type="button"
          >
            {model.status === 'failed' || model.status === 'cancelled' ? (
              <RotateCcw size={15} />
            ) : (
              <Download size={15} />
            )}
            {confirming ? 'Confirm download' : 'Download required model'}
          </button>
        ) : null}
      </div>
    </div>
  );
}
