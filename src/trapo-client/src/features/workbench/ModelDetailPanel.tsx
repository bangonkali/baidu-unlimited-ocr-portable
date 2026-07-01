import { Boxes, Star } from 'lucide-react';

import type { ModelAssetRecord } from '../../api/types';
import type { ModelActionHandlers } from './ModelActions';
import { ModelActions } from './ModelActions';
import styles from './ModelDetailPanel.module.css';
import { ModelFileTable } from './ModelFileTable';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';
import { modelDownloadedBytes, modelPercent, modelRequiredBytes } from './modelLibrary';
import { statusIcon, statusText } from './modelStatus';

interface ModelDetailPanelProps extends ModelActionHandlers {
  busy?: boolean;
  model?: ModelAssetRecord;
}

export function ModelDetailPanel(props: ModelDetailPanelProps) {
  const { model } = props;
  if (!model) {
    return (
      <section className={styles.panel} aria-label="Model detail">
        <header className={styles.header}>
          <div className={styles.headerTitle}>
            <Boxes size={16} />
            <span>Model Detail</span>
          </div>
        </header>
        <div className={styles.empty}>Model not found.</div>
      </section>
    );
  }
  const percent = modelPercent(model);
  return (
    <section className={styles.panel} aria-label="Model detail">
      <header className={styles.header}>
        <div className={styles.headerTitle}>
          <Boxes size={16} />
          <span>Model Detail</span>
        </div>
      </header>
      <div className={styles.body}>
        <section className={styles.hero}>
          <div className={styles.titleBlock}>
            <div className={styles.badges}>
              <span className={styles.badge}>
                {statusIcon(model.status)}
                {model.status}
              </span>
              {model.selected ? <span className={styles.badge}>In Use</span> : null}
              {model.recommended ? (
                <span className={styles.badge}>
                  <Star size={12} />
                  Recommended
                </span>
              ) : null}
            </div>
            <h1>{model.display_name}</h1>
            <p>{model.notes ?? model.status_message ?? statusText(model.status)}</p>
          </div>
          <ModelActions {...props} model={model} />
        </section>
        <section className={styles.section}>
          <h2>Download Progress</h2>
          <div className={styles.metrics}>
            <Metric label="Progress" value={formatPercent(percent)} />
            <Metric
              label="Downloaded"
              value={`${formatBytes(modelDownloadedBytes(model))} / ${formatBytes(
                modelRequiredBytes(model),
              )}`}
            />
            <Metric label="Speed" value={formatRate(model.bytes_per_second)} />
            <Metric label="ETA" value={formatEta(model.eta_seconds)} />
          </div>
          <ModelFileTable files={model.files} model={model} />
        </section>
        <section className={styles.section}>
          <h2>Model Metadata</h2>
          <dl className={styles.meta}>
            <dt>Model ID</dt>
            <dd>{model.model_id}</dd>
            <dt>Provider</dt>
            <dd>{model.provider_name ?? 'Hugging Face'}</dd>
            <dt>Repository</dt>
            <dd>{model.repo_id ?? 'Unavailable'}</dd>
            <dt>Revision</dt>
            <dd>{model.revision ?? 'main'}</dd>
            <dt>Quantization</dt>
            <dd>{model.quantization ?? 'GGUF'}</dd>
            <dt>Bits</dt>
            <dd>{model.bits ? `${model.bits}-bit` : 'mixed'}</dd>
            <dt>Hardware tier</dt>
            <dd>{model.hardware_tier ?? 'Runtime default'}</dd>
            <dt>Local path</dt>
            <dd>{model.local_path ?? 'Not downloaded'}</dd>
            <dt>Auth</dt>
            <dd>{model.auth_available ? `Using ${model.auth_source}` : 'Public download'}</dd>
            <dt>Status detail</dt>
            <dd>{model.error ?? model.status_message ?? statusText(model.status)}</dd>
          </dl>
        </section>
      </div>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className={styles.metric}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
