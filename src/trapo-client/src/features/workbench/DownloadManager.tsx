import { DownloadCloud, LoaderCircle, X, XCircle } from 'lucide-react';

import type { ModelAssetRecord, ModelDownloadFileRecord } from '../../api/types';
import styles from './DownloadManager.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';

export interface ActiveDownloadItem {
  file: ModelDownloadFileRecord;
  id: string;
  modelId: string;
  modelName: string;
}

interface DownloadManagerProps {
  models: ModelAssetRecord[];
  busy?: boolean;
  onCancelModel: (modelId: string) => void;
  onClose: () => void;
}

const activeStatuses = new Set(['queued', 'downloading', 'cancelling']);

export function DownloadManager({ busy, models, onCancelModel, onClose }: DownloadManagerProps) {
  const tracked = activeDownloadItems(models);
  const totalBytes = tracked.reduce((total, item) => total + (item.file.total_bytes ?? 0), 0);
  const downloadedBytes = tracked.reduce((total, item) => total + item.file.downloaded_bytes, 0);
  const totalPercent = totalBytes > 0 ? (downloadedBytes / totalBytes) * 100 : 0;

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
          <strong>{tracked.length} active files</strong>
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
        {tracked.map((item) => (
          <DownloadItem busy={busy} item={item} key={item.id} onCancelModel={onCancelModel} />
        ))}
      </div>
    </aside>
  );
}

export function activeDownloadItems(models: ModelAssetRecord[]): ActiveDownloadItem[] {
  return models.flatMap((model) =>
    (model.files ?? [])
      .filter((file) => activeStatuses.has(file.status))
      .map((file) => ({
        file,
        id: `${model.model_id}:${file.file_id}`,
        modelId: model.model_id,
        modelName: model.display_name,
      })),
  );
}

function DownloadItem({
  busy,
  item,
  onCancelModel,
}: {
  busy?: boolean;
  item: ActiveDownloadItem;
  onCancelModel: (modelId: string) => void;
}) {
  const isDownloading = item.file.status === 'downloading';
  const cancellable = item.file.status === 'downloading' || item.file.status === 'queued';
  return (
    <article className={styles.item}>
      <div className={styles.rowHeader}>
        {isDownloading ? <LoaderCircle size={14} /> : <DownloadCloud size={14} />}
        <div className={styles.title}>
          <strong>{item.file.file_name}</strong>
          <span>{item.modelName}</span>
        </div>
        <span className={styles.status}>{item.file.status}</span>
        {cancellable ? (
          <button
            className={styles.cancel}
            disabled={busy}
            onClick={() => onCancelModel(item.modelId)}
            title="Cancel"
            type="button"
          >
            <XCircle size={14} />
          </button>
        ) : null}
      </div>
      <div className={styles.progress}>
        <span style={{ width: `${Math.min(item.file.percent, 100)}%` }} />
      </div>
      <div className={styles.meta}>
        <span>{formatPercent(item.file.percent)}</span>
        <span>{formatRate(item.file.bytes_per_second)}</span>
        <span>{formatEta(item.file.eta_seconds)}</span>
      </div>
    </article>
  );
}
