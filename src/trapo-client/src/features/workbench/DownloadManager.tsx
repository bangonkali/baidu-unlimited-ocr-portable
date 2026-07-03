import { DownloadCloud, LoaderCircle, X, XCircle } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './DownloadManager.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';
import { modelDownloadedBytes, modelPercent, modelRequiredBytes } from './modelLibrary';

interface DownloadManagerProps {
  models: ModelAssetRecord[];
  busy?: boolean;
  onCancelModel: (modelId: string) => void;
  onClose: () => void;
}

export function DownloadManager({ busy, models, onCancelModel, onClose }: DownloadManagerProps) {
  const tracked = models.filter((model) => model.status !== 'downloaded');
  const totalBytes = tracked.reduce((total, model) => total + modelRequiredBytes(model), 0);
  const downloadedBytes = tracked.reduce((total, model) => total + modelDownloadedBytes(model), 0);
  const totalPercent = totalBytes > 0 ? (downloadedBytes / totalBytes) * 100 : 100;

  return (
    <aside className={styles.overlay} aria-label="Download manager">
      <header className={styles.header}>
        <span>Download Manager</span>
        <button className={styles.close} onClick={onClose} title="Close" type="button">
          <X size={14} />
        </button>
      </header>
      <div className={styles.summary}>
        <DownloadCloud size={17} />
        <div className={styles.summaryText}>
          <strong>{tracked.length} queued or pending</strong>
          <span>
            {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
          </span>
          <div className={styles.progress}>
            <span style={{ width: `${Math.min(totalPercent, 100)}%` }} />
          </div>
        </div>
      </div>
      <div className={styles.list}>
        {tracked.length === 0 ? <div className={styles.empty}>No active downloads</div> : null}
        {tracked.map((model) => (
          <DownloadItem
            busy={busy}
            key={model.model_id}
            model={model}
            onCancelModel={onCancelModel}
          />
        ))}
      </div>
    </aside>
  );
}

function DownloadItem({
  busy,
  model,
  onCancelModel,
}: {
  busy?: boolean;
  model: ModelAssetRecord;
  onCancelModel: (modelId: string) => void;
}) {
  const percent = modelPercent(model);
  const cancellable = model.status === 'downloading' || model.status === 'queued';
  return (
    <article className={styles.item}>
      <div className={styles.rowHeader}>
        {model.status === 'downloading' ? <LoaderCircle size={14} /> : <DownloadCloud size={14} />}
        <strong>{model.display_name}</strong>
        <span className={styles.status}>{model.status}</span>
        {cancellable ? (
          <button
            className={styles.cancel}
            disabled={busy}
            onClick={() => onCancelModel(model.model_id)}
            title="Cancel"
            type="button"
          >
            <XCircle size={14} />
          </button>
        ) : null}
      </div>
      <div className={styles.progress}>
        <span style={{ width: `${Math.min(percent, 100)}%` }} />
      </div>
      <div className={styles.meta}>
        <span>{formatPercent(percent)}</span>
        <span>{formatRate(model.bytes_per_second)}</span>
        <span>{formatEta(model.eta_seconds)}</span>
      </div>
      <div className={styles.files}>
        {(model.files ?? []).map((file) => (
          <div className={styles.fileRow} key={file.file_id}>
            <span>{file.file_name}</span>
            <span>{file.status}</span>
            <span>{formatPercent(file.percent)}</span>
          </div>
        ))}
      </div>
    </article>
  );
}
