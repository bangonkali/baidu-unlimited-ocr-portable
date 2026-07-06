import { Link } from '@tanstack/react-router';
import { Cpu, Gauge, Moon, Settings2, Sun } from 'lucide-react';

import type { ModelsPayload, OcrProfileRecord, SettingsPayload } from '../../api/types';
import type { SettingsSection } from '../../routeSearch';
import type { ThemeMode } from '../../stores/workbenchStore';
import { ModelDownloadSettings } from './ModelDownloadSettings';
import styles from './SettingsPanel.module.css';

interface SettingsPanelProps {
  activeSection?: SettingsSection;
  busy?: boolean;
  models?: ModelsPayload;
  profiles: OcrProfileRecord[];
  selectedProfile: string;
  settings?: SettingsPayload;
  theme: ThemeMode;
  onDownloadConcurrencyChange: (value: number) => void;
  onModelChange: (modelId: string) => void;
  onProfileChange: (profileId: string) => void;
  onRuntimeChange: (runtimeId: string) => void;
  onThemeChange: (theme: ThemeMode) => void;
}

export function SettingsPanel(props: SettingsPanelProps) {
  const selectedModel = props.models?.selected_model_id ?? props.settings?.selected_model_id ?? '';
  const selectedRuntime = props.settings?.selected_runtime_id ?? '';
  const activeSection = props.activeSection ?? 'appearance';

  return (
    <section className={styles.panel} aria-label="Settings">
      <header className={styles.header}>
        <Settings2 size={16} />
        <span>Settings</span>
      </header>
      <div className={styles.settingsLayout}>
        <nav className={styles.nav} aria-label="Settings sections">
          {settingsSections.map((section) => (
            <Link
              className={styles.navLink}
              data-active={activeSection === section.id}
              key={section.id}
              search={{ section: section.id }}
              to="/settings"
            >
              {section.label}
            </Link>
          ))}
        </nav>
        <div className={styles.content}>
          {activeSection === 'appearance' ? <AppearanceSettings {...props} /> : null}
          {activeSection === 'runtime' ? (
            <RuntimeSettings
              busy={props.busy}
              onRuntimeChange={props.onRuntimeChange}
              selectedRuntime={selectedRuntime}
              settings={props.settings}
            />
          ) : null}
          {activeSection === 'ocr' ? (
            <OcrSettings
              busy={props.busy}
              models={props.models}
              onModelChange={props.onModelChange}
              onProfileChange={props.onProfileChange}
              profiles={props.profiles}
              selectedModel={selectedModel}
              selectedProfile={props.selectedProfile}
              settings={props.settings}
            />
          ) : null}
          {activeSection === 'storage' ? <StorageSettings settings={props.settings} /> : null}
          {activeSection === 'models' ? (
            <ModelDownloadSettings
              busy={props.busy}
              models={props.models}
              onDownloadConcurrencyChange={props.onDownloadConcurrencyChange}
              settings={props.settings}
            />
          ) : null}
        </div>
      </div>
    </section>
  );
}

const settingsSections: Array<{ id: SettingsSection; label: string }> = [
  { id: 'appearance', label: 'Appearance' },
  { id: 'runtime', label: 'Runtime' },
  { id: 'ocr', label: 'OCR Defaults' },
  { id: 'models', label: 'Models' },
  { id: 'storage', label: 'Storage' },
];

function AppearanceSettings({
  onThemeChange,
  theme,
}: Pick<SettingsPanelProps, 'onThemeChange' | 'theme'>) {
  return (
    <section className={styles.group}>
      <h2>Appearance</h2>
      <p>Choose a VS Code-style color theme for the local workbench.</p>
      <div className={styles.themeGrid}>
        <ThemeButton
          active={theme === 'dark'}
          icon="dark"
          label="Dark"
          onClick={() => onThemeChange('dark')}
        />
        <ThemeButton
          active={theme === 'light'}
          icon="light"
          label="Light"
          onClick={() => onThemeChange('light')}
        />
      </div>
    </section>
  );
}

function RuntimeSettings({
  busy,
  onRuntimeChange,
  selectedRuntime,
  settings,
}: {
  busy?: boolean;
  selectedRuntime: string;
  settings?: SettingsPayload;
  onRuntimeChange: (runtimeId: string) => void;
}) {
  return (
    <section className={styles.group}>
      <h2>Runtime</h2>
      <label>
        <span>Runtime</span>
        <select
          disabled={busy}
          onChange={(event) => onRuntimeChange(event.target.value)}
          value={selectedRuntime}
        >
          {(settings?.runtime_variants ?? []).map((runtime) => (
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
      <RuntimeList settings={settings} />
    </section>
  );
}

function OcrSettings({
  busy,
  models,
  onModelChange,
  onProfileChange,
  profiles,
  selectedModel,
  selectedProfile,
  settings,
}: {
  busy?: boolean;
  models?: ModelsPayload;
  profiles: OcrProfileRecord[];
  selectedModel: string;
  selectedProfile: string;
  settings?: SettingsPayload;
  onModelChange: (modelId: string) => void;
  onProfileChange: (profileId: string) => void;
}) {
  return (
    <section className={styles.group}>
      <h2>OCR Defaults</h2>
      <label>
        <span>Model</span>
        <select
          disabled={busy}
          onChange={(event) => onModelChange(event.target.value)}
          value={selectedModel}
        >
          {(models?.models ?? []).map((model) => (
            <option key={model.model_id} value={model.model_id}>
              {model.display_name}
            </option>
          ))}
        </select>
      </label>
      <label>
        <span>Profile</span>
        <select
          disabled={busy}
          onChange={(event) => onProfileChange(event.target.value)}
          value={selectedProfile}
        >
          {profiles.map((profile) => (
            <option key={profile.key} value={profile.key}>
              {profile.label}
            </option>
          ))}
        </select>
      </label>
      <div className={styles.meta}>
        <span>PDF DPI {settings?.pdf_dpi ?? 200}</span>
        <span>Concurrency {settings?.ocr_concurrency ?? 1}</span>
      </div>
    </section>
  );
}

function StorageSettings({ settings }: { settings?: SettingsPayload }) {
  return (
    <section className={styles.group}>
      <h2>Storage</h2>
      <p>{settings?.database_path ?? 'Database path unavailable'}</p>
      <p>{settings?.cache_path ?? 'Cache path unavailable'}</p>
    </section>
  );
}

function ThemeButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: 'dark' | 'light';
  label: string;
  onClick: () => void;
}) {
  const Icon = icon === 'dark' ? Moon : Sun;
  return (
    <button aria-pressed={active} className={styles.themeButton} onClick={onClick} type="button">
      <Icon size={16} />
      <span>{label}</span>
    </button>
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
