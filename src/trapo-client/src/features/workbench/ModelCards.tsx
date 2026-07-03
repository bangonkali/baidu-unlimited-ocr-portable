import { ShieldCheck, ShieldOff, Star } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import type { ModelActionHandlers } from './ModelActions';
import { ModelActions } from './ModelActions';
import styles from './ModelCards.module.css';
import { formatBytes } from './modelDownloadFormat';
import { modelRequiredBytes } from './modelLibrary';
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
      {model.error ? <div className={styles.error}>{model.error}</div> : null}
    </article>
  );
}
