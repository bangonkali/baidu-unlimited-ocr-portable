import {
  Download,
  HardDriveDownload,
  RotateCcw,
  ShieldCheck,
  ShieldOff,
  XCircle,
} from 'lucide-react';

import type {
  ModelAssetRecord,
  ModelDownloadFileRecord,
  ModelsPayload,
  StatusPayload,
} from '../../api/types';
import styles from './ModelManager.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';

interface ModelManagerProps {
  busy?: boolean;
  models?: ModelsPayload;
  status?: StatusPayload;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
}

export function ModelManager({
  busy,
  models,
  status,
  onCancelModel,
  onDownloadModel,
}: ModelManagerProps) {
  return (
    <section className={styles.manager} aria-label="Models" data-tour="models">
      <header className={styles.header}>
        <HardDriveDownload size={16} />
        <span>Models</span>
      </header>
      <div className={styles.body}>
        {(models?.models ?? []).map((model) => (
          <ModelCard
            busy={busy}
            key={model.model_id}
            model={model}
            onCancelModel={onCancelModel}
            onDownloadModel={onDownloadModel}
            status={status}
          />
        ))}
      </div>
    </section>
  );
}

function ModelCard({
  busy,
  model,
  status,
  onCancelModel,
  onDownloadModel,
}: {
  busy?: boolean;
  model: ModelAssetRecord;
  status?: StatusPayload;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
}) {
  const isDownloading = model.status === 'downloading';
  const isReady = model.status === 'downloaded';
  const isRetry = ['error', 'cancelled'].includes(model.status);
  const percent = isReady ? 100 : (model.overall_percent ?? modelPercentage(model));
  return (
    <article className={styles.model}>
      <div className={styles.titleRow}>
        <div>
          <h2>{model.display_name}</h2>
          <p>{model.status_message ?? statusText(model.status)}</p>
        </div>
        <div className={styles.actions}>
          {isDownloading ? (
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
          {!isReady && !isDownloading ? (
            <button
              className={styles.downloadButton}
              disabled={busy}
              onClick={() => onDownloadModel(model.model_id)}
              type="button"
            >
              {isRetry ? (
                <RotateCcw size={15} strokeWidth={1.9} />
              ) : (
                <Download size={15} strokeWidth={1.9} />
              )}
              <span>{isRetry ? 'Retry' : 'Download missing'}</span>
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
      </div>

      <div className={styles.authRow}>
        {model.auth_available ? <ShieldCheck size={15} /> : <ShieldOff size={15} />}
        <span>
          {model.auth_available
            ? `Authenticated with ${model.auth_source}`
            : 'Using public Hugging Face download'}
        </span>
      </div>

      <div
        aria-label="Overall download progress"
        aria-valuemax={100}
        aria-valuemin={0}
        aria-valuenow={percent}
        className={styles.progress}
        role="progressbar"
      >
        <div style={{ width: `${percent}%` }} />
      </div>

      <dl className={styles.meta}>
        <dt>Status</dt>
        <dd>{model.status}</dd>
        <dt>Progress</dt>
        <dd>
          {formatBytes(model.overall_downloaded_bytes ?? model.downloaded_bytes)} /{' '}
          {formatBytes(model.overall_total_bytes ?? model.total_bytes)} ({formatPercent(percent)})
        </dd>
        <dt>Speed</dt>
        <dd>{formatRate(model.bytes_per_second)}</dd>
        <dt>ETA</dt>
        <dd>{formatEta(model.eta_seconds)}</dd>
        <dt>Local Path</dt>
        <dd title={model.local_path ?? 'models'}>{model.local_path ?? 'models'}</dd>
        <dt>Runtime</dt>
        <dd>{status?.runtime_platform ?? 'windows-x86_64-cuda13'} / CUDA</dd>
      </dl>

      <FileTable files={model.files ?? fallbackFiles(model)} />
      {model.error ? <div className={styles.error}>{model.error}</div> : null}
    </article>
  );
}

function FileTable({ files }: { files: ModelDownloadFileRecord[] }) {
  return (
    <table className={styles.files} aria-label="Required model files">
      <thead>
        <tr className={styles.fileHeader}>
          <th scope="col">File</th>
          <th scope="col">Status</th>
          <th scope="col">Progress</th>
          <th scope="col">Rate</th>
          <th scope="col">ETA</th>
        </tr>
      </thead>
      <tbody>
        {files.map((file) => (
          <tr className={styles.fileRow} key={file.file_id}>
            <td title={file.file_name}>{file.file_name}</td>
            <td>{file.status}</td>
            <td>
              {formatBytes(file.downloaded_bytes)} / {formatBytes(file.total_bytes)} (
              {formatPercent(file.percent)})
            </td>
            <td>{formatRate(file.bytes_per_second)}</td>
            <td>{formatEta(file.eta_seconds)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function statusText(status: string) {
  if (status === 'downloaded') {
    return 'Model files are present. Scans can start.';
  }
  if (status === 'downloading') {
    return 'Downloading model assets from Hugging Face.';
  }
  if (status === 'error') {
    return 'Download failed. Check Diagnostics for the detailed error and retry.';
  }
  if (status === 'cancelled') {
    return 'Download was cancelled. Retry will resume partial files when possible.';
  }
  return 'Download the model files before starting OCR.';
}

function modelPercentage(model: ModelAssetRecord) {
  const total = model.overall_total_bytes ?? model.total_bytes ?? 0;
  const downloaded = model.overall_downloaded_bytes ?? model.downloaded_bytes ?? 0;
  return total > 0 ? Math.min(100, (downloaded / total) * 100) : 0;
}

function fallbackFiles(model: ModelAssetRecord): ModelDownloadFileRecord[] {
  return [model.model_file, model.mmproj_file].filter(Boolean).map((fileName, index) => ({
    downloaded_bytes: 0,
    file_id: index === 0 ? 'model' : 'mmproj',
    file_name: fileName ?? '',
    percent: 0,
    status: model.status,
  }));
}
