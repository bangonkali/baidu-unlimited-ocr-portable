import { Cpu, Gauge, Settings2 } from 'lucide-react';

import type { ModelsPayload, OcrProfileRecord, SettingsPayload } from '../../api/types';
import styles from './SettingsPanel.module.css';

interface SettingsPanelProps {
  activeSection?: 'runtime' | 'ocr' | 'storage' | 'models';
  busy?: boolean;
  models?: ModelsPayload;
  profiles: OcrProfileRecord[];
  selectedProfile: string;
  settings?: SettingsPayload;
  onModelChange: (modelId: string) => void;
  onProfileChange: (profileId: string) => void;
  onRuntimeChange: (runtimeId: string) => void;
}

export function SettingsPanel(props: SettingsPanelProps) {
  const selectedModel = props.models?.selected_model_id ?? props.settings?.selected_model_id ?? '';
  const selectedRuntime = props.settings?.selected_runtime_id ?? '';

  return (
    <section className={styles.panel} aria-label="Settings">
      <header className={styles.header}>
        <Settings2 size={16} />
        <span>Settings</span>
      </header>
      <div className={styles.grid}>
        <section className={styles.group} data-active={props.activeSection === 'runtime'}>
          <h2>Inference</h2>
          <label>
            <span>Runtime</span>
            <select
              disabled={props.busy}
              onChange={(event) => props.onRuntimeChange(event.target.value)}
              value={selectedRuntime}
            >
              {(props.settings?.runtime_variants ?? []).map((runtime) => (
                <option
                  disabled={!runtime.selectable}
                  key={runtime.runtime_id}
                  value={runtime.runtime_id}
                >
                  {runtime.label}
                </option>
              ))}
            </select>
          </label>
          <RuntimeList settings={props.settings} />
        </section>

        <section
          className={styles.group}
          data-active={props.activeSection === 'ocr' || props.activeSection === 'models'}
        >
          <h2>OCR Defaults</h2>
          <label>
            <span>Model</span>
            <select
              disabled={props.busy}
              onChange={(event) => props.onModelChange(event.target.value)}
              value={selectedModel}
            >
              {(props.models?.models ?? []).map((model) => (
                <option key={model.model_id} value={model.model_id}>
                  {model.display_name}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>Profile</span>
            <select
              disabled={props.busy}
              onChange={(event) => props.onProfileChange(event.target.value)}
              value={props.selectedProfile}
            >
              {props.profiles.map((profile) => (
                <option key={profile.key} value={profile.key}>
                  {profile.label}
                </option>
              ))}
            </select>
          </label>
          <div className={styles.meta}>
            <span>PDF DPI {props.settings?.pdf_dpi ?? 200}</span>
            <span>Concurrency {props.settings?.ocr_concurrency ?? 1}</span>
          </div>
        </section>

        <section className={styles.group} data-active={props.activeSection === 'storage'}>
          <h2>Storage</h2>
          <p>{props.settings?.database_path ?? 'Database path unavailable'}</p>
          <p>{props.settings?.cache_path ?? 'Cache path unavailable'}</p>
        </section>
      </div>
    </section>
  );
}

function RuntimeList({ settings }: { settings?: SettingsPayload }) {
  return (
    <div className={styles.runtimeList}>
      {(settings?.runtime_variants ?? []).map((runtime) => (
        <article
          className={styles.runtimeCard}
          data-selected={runtime.selected}
          key={runtime.runtime_id}
        >
          <div>
            {runtime.accelerator === 'cpu' ? <Cpu size={14} /> : <Gauge size={14} />}
            <strong>{runtime.accelerator.toUpperCase()}</strong>
            <span>{runtime.platform}</span>
          </div>
          <small>
            {runtime.selectable
              ? 'Ready'
              : runtime.installed
                ? runtime.support_detail
                : 'Runtime files are not installed'}
          </small>
        </article>
      ))}
    </div>
  );
}
