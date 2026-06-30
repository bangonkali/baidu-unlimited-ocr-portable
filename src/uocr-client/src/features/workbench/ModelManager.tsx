import {
  CheckCircle2,
  CircleDot,
  Cpu,
  Download,
  HardDriveDownload,
  Library,
  RotateCcw,
  ShieldCheck,
  ShieldOff,
  Star,
  XCircle,
} from 'lucide-react';

import type {
  ModelAssetRecord,
  ModelDownloadFileRecord,
  ModelsPayload,
  StatusPayload,
} from '../../api/types';
import fileStyles from './ModelFileTable.module.css';
import styles from './ModelManager.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';

interface ModelManagerProps {
  busy?: boolean;
  models?: ModelsPayload;
  status?: StatusPayload;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onSelectModel: (modelId: string) => void;
}

export function ModelManager({
  busy,
  models,
  status,
  onCancelModel,
  onDownloadModel,
  onSelectModel,
}: ModelManagerProps) {
  const library = models?.models ?? [];
  const selected =
    library.find((model) => model.selected) ??
    library.find((model) => model.model_id === models?.selected_model_id) ??
    library[0];

  return (
    <section className={styles.manager} aria-label="Models" data-tour="models">
      <header className={styles.header}>
        <div className={styles.headerTitle}>
          <Library size={16} />
          <span>Model Library</span>
        </div>
        <span className={styles.provider}>{models?.provider_repo ?? selected?.repo_id}</span>
      </header>
      <div className={styles.summary}>
        <div>
          <span className={styles.eyebrow}>Selected model</span>
          <h2>{selected?.display_name ?? 'No model selected'}</h2>
          <p>{selected?.notes ?? 'Choose a model variant and download its required files.'}</p>
        </div>
        <div className={styles.summaryStats}>
          <span>
            <Cpu size={14} />
            {status?.runtime_platform ?? 'windows-x86_64-cuda13'} / {status?.accelerator ?? 'cuda'}
          </span>
          <span>
            <HardDriveDownload size={14} />
            {selected
              ? formatBytes(selected.total_required_bytes ?? selected.overall_total_bytes)
              : '0 B'}
          </span>
        </div>
      </div>
      <div className={styles.body}>
        {library.map((model) => (
          <ModelCard
            busy={busy}
            key={model.model_id}
            model={model}
            onCancelModel={onCancelModel}
            onDownloadModel={onDownloadModel}
            onSelectModel={onSelectModel}
          />
        ))}
      </div>
    </section>
  );
}

function ModelCard({
  busy,
  model,
  onCancelModel,
  onDownloadModel,
  onSelectModel,
}: {
  busy?: boolean;
  model: ModelAssetRecord;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onSelectModel: (modelId: string) => void;
}) {
  const isDownloading = model.status === 'downloading';
  const isReady = model.status === 'downloaded';
  const isRetry = ['error', 'cancelled'].includes(model.status);
  const percent = isReady ? 100 : (model.overall_percent ?? modelPercentage(model));
  return (
    <article className={styles.model} data-selected={model.selected === true}>
      <div className={styles.titleRow}>
        <div className={styles.titleBlock}>
          <div className={styles.badges}>
            <span className={styles.statusBadge} data-status={model.status}>
              {statusIcon(model.status)}
              {model.status}
            </span>
            {model.selected ? <span className={styles.badge}>Selected</span> : null}
            {model.recommended ? (
              <span className={styles.badge}>
                <Star size={12} />
                Recommended
              </span>
            ) : null}
          </div>
          <h3>{model.display_name}</h3>
          <p>{model.quality ?? model.status_message ?? statusText(model.status)}</p>
        </div>
        <div className={styles.actions}>
          <button
            className={model.selected ? styles.selectedButton : styles.secondaryButton}
            disabled={busy || model.selected}
            onClick={() => onSelectModel(model.model_id)}
            type="button"
          >
            <CircleDot size={15} strokeWidth={1.9} />
            <span>{model.selected ? 'In Use' : 'Use'}</span>
          </button>
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
      </div>

      <div className={styles.specGrid}>
        <span>{model.quantization ?? 'GGUF'}</span>
        <span>{model.bits ? `${model.bits}-bit` : 'mixed'}</span>
        <span>{model.hardware_tier ?? 'CUDA runtime'}</span>
        <span>{formatBytes(model.total_required_bytes ?? model.overall_total_bytes)}</span>
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
        aria-label={`${model.display_name} download progress`}
        aria-valuemax={100}
        aria-valuemin={0}
        aria-valuenow={percent}
        className={styles.progress}
        role="progressbar"
      >
        <div style={{ width: `${percent}%` }} />
      </div>

      <dl className={styles.meta}>
        <dt>Progress</dt>
        <dd>
          {formatBytes(model.overall_downloaded_bytes ?? model.downloaded_bytes)} /{' '}
          {formatBytes(model.overall_total_bytes ?? model.total_required_bytes)} (
          {formatPercent(percent)})
        </dd>
        <dt>Speed</dt>
        <dd>{formatRate(model.bytes_per_second)}</dd>
        <dt>ETA</dt>
        <dd>{formatEta(model.eta_seconds)}</dd>
        <dt>Files</dt>
        <dd>
          {model.downloaded_file_count ?? 0}/{model.total_file_count ?? 2} ready
        </dd>
      </dl>

      <FileTable files={model.files ?? fallbackFiles(model)} />
      {model.error ? <div className={styles.error}>{model.error}</div> : null}
    </article>
  );
}

function FileTable({ files }: { files: ModelDownloadFileRecord[] }) {
  return (
    <table className={fileStyles.files} aria-label="Required model files">
      <thead>
        <tr className={fileStyles.fileHeader}>
          <th scope="col">File</th>
          <th scope="col">Status</th>
          <th scope="col">Progress</th>
          <th scope="col">Rate</th>
          <th scope="col">ETA</th>
        </tr>
      </thead>
      <tbody>
        {files.map((file) => (
          <tr className={fileStyles.fileRow} key={file.file_id}>
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

function statusIcon(status: string) {
  return status === 'downloaded' ? <CheckCircle2 size={12} /> : <CircleDot size={12} />;
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
