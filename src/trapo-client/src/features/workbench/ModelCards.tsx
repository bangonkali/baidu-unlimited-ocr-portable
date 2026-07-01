import { ShieldCheck, ShieldOff, Star } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import type { ModelActionHandlers } from './ModelActions';
import { ModelActions } from './ModelActions';
import styles from './ModelCards.module.css';
import { ModelFileTable } from './ModelFileTable';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';
import { modelDownloadedBytes, modelPercent, modelRequiredBytes } from './modelLibrary';
import { statusIcon, statusText } from './modelStatus';

interface ModelCardsProps extends ModelActionHandlers {
  busy?: boolean;
  models: ModelAssetRecord[];
}

export function ModelCards({
  busy,
  models,
  onCancelModel,
  onDownloadModel,
  onSelectModel,
}: ModelCardsProps) {
  return (
    <div className={styles.cardBody}>
      {models.map((model) => (
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
  );
}

function ModelCard(props: ModelActionHandlers & { busy?: boolean; model: ModelAssetRecord }) {
  const { model } = props;
  const percent = modelPercent(model);
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
          <h3>
            <a href={`/models/${encodeURIComponent(model.model_id)}`}>{model.display_name}</a>
          </h3>
          <p>{model.quality ?? model.status_message ?? statusText(model.status)}</p>
        </div>
        <ModelActions {...props} />
      </div>
      <div className={styles.specGrid}>
        <span>{model.quantization ?? 'GGUF'}</span>
        <span>{model.bits ? `${model.bits}-bit` : 'mixed'}</span>
        <span>{model.hardware_tier ?? 'Runtime default'}</span>
        <span>{formatBytes(modelRequiredBytes(model))}</span>
      </div>
      <div className={styles.authRow}>
        {model.auth_available ? <ShieldCheck size={15} /> : <ShieldOff size={15} />}
        <span>
          {model.auth_available
            ? `Authenticated with ${model.auth_source}`
            : 'Using public Hugging Face download'}
        </span>
      </div>
      <ProgressBar label={`${model.display_name} download progress`} percent={percent} />
      <dl className={styles.meta}>
        <dt>Progress</dt>
        <dd>
          {formatBytes(modelDownloadedBytes(model))} / {formatBytes(modelRequiredBytes(model))} (
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
      <ModelFileTable files={model.files} model={model} />
      {model.error ? <div className={styles.error}>{model.error}</div> : null}
    </article>
  );
}

export function ProgressBar({ label, percent }: { label: string; percent: number }) {
  return (
    <div
      aria-label={label}
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={Math.round(percent)}
      className={styles.progress}
      role="progressbar"
    >
      <div style={{ width: `${percent}%` }} />
    </div>
  );
}
