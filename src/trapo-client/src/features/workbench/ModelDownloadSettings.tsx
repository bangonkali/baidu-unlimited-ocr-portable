import type { ModelsPayload, SettingsPayload } from '../../api/types';
import styles from './SettingsPanel.module.css';

interface ModelDownloadSettingsProps {
  busy?: boolean;
  models?: ModelsPayload;
  settings?: SettingsPayload;
  onDownloadConcurrencyChange: (value: number) => void;
}

export function ModelDownloadSettings({
  busy,
  models,
  onDownloadConcurrencyChange,
  settings,
}: ModelDownloadSettingsProps) {
  const concurrency = settings?.download_concurrency ?? 4;
  return (
    <section className={styles.group}>
      <h2>Models</h2>
      <p>{models?.provider_label ?? models?.provider_repo ?? 'Unlimited-OCR model catalog'}</p>
      <label>
        <span>Concurrent downloads</span>
        <input
          disabled={busy}
          max={16}
          min={1}
          onChange={(event) => onDownloadConcurrencyChange(Number(event.target.value))}
          step={1}
          type="number"
          value={concurrency}
        />
      </label>
      <div className={styles.meta}>
        <span>{models?.models.length ?? 0} variants</span>
        <span>{concurrency} files at once</span>
        <span>Selected {models?.selected_model_id ?? 'none'}</span>
      </div>
    </section>
  );
}
