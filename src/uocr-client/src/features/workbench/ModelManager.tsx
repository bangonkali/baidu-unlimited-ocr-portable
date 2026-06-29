import { Download, HardDriveDownload } from 'lucide-react';

import type { ModelsPayload, StatusPayload } from '../../api/types';
import styles from './ModelManager.module.css';

interface ModelManagerProps {
  busy?: boolean;
  models?: ModelsPayload;
  status?: StatusPayload;
  onDownloadModel: (modelId: string) => void;
}

export function ModelManager({ busy, models, status, onDownloadModel }: ModelManagerProps) {
  return (
    <section className={styles.manager} aria-label="Models" data-tour="models">
      <header className={styles.header}>
        <HardDriveDownload size={16} />
        <span>Models</span>
      </header>
      <div className={styles.body}>
        {(models?.models ?? []).map((model) => (
          <article className={styles.model} key={model.model_id}>
            <div className={styles.titleRow}>
              <div>
                <h2>{model.display_name}</h2>
                <p>{model.status_message ?? statusText(model.status)}</p>
              </div>
              <button
                className={styles.downloadButton}
                disabled={busy || model.status === 'downloaded' || model.status === 'downloading'}
                onClick={() => onDownloadModel(model.model_id)}
                type="button"
              >
                <Download size={15} strokeWidth={1.9} />
                <span>Download model</span>
              </button>
            </div>
            <ProgressBar model={model} />
            <dl className={styles.meta}>
              <dt>Status</dt>
              <dd>{model.status}</dd>
              <dt>Files</dt>
              <dd>{[model.model_file, model.mmproj_file].filter(Boolean).join(', ')}</dd>
              <dt>Local Path</dt>
              <dd>{model.local_path ?? 'models'}</dd>
              <dt>Runtime</dt>
              <dd>{status?.runtime_platform ?? 'windows-x86_64-cuda13'} / CUDA</dd>
            </dl>
            {model.error ? <div className={styles.error}>{model.error}</div> : null}
          </article>
        ))}
      </div>
    </section>
  );
}

function ProgressBar({ model }: { model: ModelsPayload['models'][number] }) {
  const total = model.total_bytes ?? 0;
  const downloaded = model.downloaded_bytes ?? 0;
  const percent = total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
  const value = model.status === 'downloaded' ? 100 : percent;
  return (
    <div
      aria-label="Download progress"
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={value}
      className={styles.progress}
      role="progressbar"
    >
      <div style={{ width: `${value}%` }} />
    </div>
  );
}

function statusText(status: string) {
  if (status === 'downloaded') {
    return 'Model files are present. Scans can start.';
  }
  if (status === 'downloading') {
    return 'Downloading model assets from Hugging Face.';
  }
  return 'Download the model files before starting OCR.';
}
