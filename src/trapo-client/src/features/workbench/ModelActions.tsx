import { CircleDot, Download, RotateCcw, XCircle } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './ModelManager.module.css';

export interface ModelActionHandlers {
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onSelectModel: (modelId: string) => void;
}

interface ModelActionsProps extends ModelActionHandlers {
  busy?: boolean;
  compact?: boolean;
  model: ModelAssetRecord;
}

export function ModelActions({
  busy,
  compact,
  model,
  onCancelModel,
  onDownloadModel,
  onSelectModel,
}: ModelActionsProps) {
  const isDownloading = model.status === 'downloading';
  const isQueued = model.status === 'queued';
  const isActive = isDownloading || isQueued || model.status === 'cancelling';
  const isReady = model.status === 'downloaded';
  const isRetry = ['error', 'cancelled'].includes(model.status);
  return (
    <div className={compact ? styles.actionsCompact : styles.actions}>
      <button
        className={model.selected ? styles.selectedButton : styles.secondaryButton}
        disabled={busy || model.selected}
        onClick={() => onSelectModel(model.model_id)}
        type="button"
      >
        <CircleDot size={15} strokeWidth={1.9} />
        <span>{model.selected ? 'In Use' : 'Use'}</span>
      </button>
      {isDownloading || isQueued ? (
        <button
          className={styles.secondaryButton}
          disabled={busy}
          onClick={() => onCancelModel(model.model_id)}
          type="button"
        >
          <XCircle size={15} strokeWidth={1.9} />
          <span>Cancel</span>
        </button>
      ) : null}
      {!isReady && !isActive ? (
        <button
          className={styles.downloadButton}
          disabled={busy}
          onClick={() => onDownloadModel(model.model_id)}
          type="button"
        >
          {isRetry ? <RotateCcw size={15} strokeWidth={1.9} /> : <Download size={15} />}
          <span>{isRetry ? 'Retry' : 'Download'}</span>
        </button>
      ) : null}
      {isReady ? (
        <button
          className={styles.secondaryButton}
          disabled={busy}
          onClick={() => onDownloadModel(model.model_id, true)}
          type="button"
        >
          <RotateCcw size={15} strokeWidth={1.9} />
          <span>Re-download</span>
        </button>
      ) : null}
    </div>
  );
}
