import { Star } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './ModelDataGrid.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';
import { modelDownloadedBytes, modelPercent, modelRequiredBytes } from './modelLibrary';

export function ModelCell({ model }: { model: ModelAssetRecord }) {
  return (
    <div className={styles.gridModelCell}>
      <a href={`/models/${encodeURIComponent(model.model_id)}`}>{model.display_name}</a>
      <span>{model.quantization ?? model.provider_name ?? 'GGUF'}</span>
      <small>
        {model.selected ? 'Selected' : model.recommended ? 'Recommended' : model.quality}
      </small>
      {model.recommended ? <Star size={12} /> : null}
    </div>
  );
}

export function ProgressCell({ model }: { model: ModelAssetRecord }) {
  const percent = modelPercent(model);
  return (
    <div className={styles.gridProgressCell}>
      <span>
        {formatBytes(modelDownloadedBytes(model))} / {formatBytes(modelRequiredBytes(model))}
      </span>
      <div className={styles.progressMini}>
        <span style={{ width: `${percent}%` }} />
      </div>
      <small>
        {formatPercent(percent)} · {formatRate(model.bytes_per_second)} ·{' '}
        {formatEta(model.eta_seconds)}
      </small>
    </div>
  );
}

export function FilesCell({ model }: { model: ModelAssetRecord }) {
  return (
    <div className={styles.gridFilesCell}>
      <span>
        {model.downloaded_file_count ?? 0}/{model.total_file_count ?? 2}
      </span>
      {(model.files ?? []).map((file) => (
        <small key={file.file_id}>
          {file.file_name}: {formatPercent(file.percent)}
        </small>
      ))}
    </div>
  );
}
