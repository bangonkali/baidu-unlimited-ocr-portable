import { FolderOpen, Play, Square, Workflow } from 'lucide-react';
import { useEffect, useState } from 'react';

import type {
  IngestRunRecord,
  ModelAssetRecord,
  ModelsPayload,
  OcrProfileRecord,
  StatusPayload,
} from '../../api/types';
import type { IngestRouteSearch } from '../../routeSearch';
import styles from './IngestStartPanel.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

interface IngestStartOptions {
  reprocess?: boolean;
}

interface IngestStartPanelProps {
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  busy?: boolean;
  ingestSearch?: IngestRouteSearch;
  model?: ModelAssetRecord;
  models?: ModelsPayload;
  profiles: OcrProfileRecord[];
  rootPath: string;
  selectedProfile: string;
  status?: StatusPayload;
  onModelChange: (modelId: string) => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onRootPathChange: (value: string) => void;
  onStart: (options?: IngestStartOptions) => void;
  onStop: () => void;
}

export function IngestStartPanel(props: IngestStartPanelProps) {
  const [reprocess, setReprocess] = useState(props.ingestSearch?.reprocess ?? false);
  const active = isIngestBusy(props.status, props.activeRun);
  const modelReady = props.model?.status === 'downloaded';
  const canStart =
    Boolean(props.rootPath.trim()) &&
    modelReady &&
    !props.busy &&
    !active &&
    props.profiles.length > 0;

  useEffect(() => {
    setReprocess(props.ingestSearch?.reprocess ?? false);
  }, [props.ingestSearch?.reprocess]);

  return (
    <section className={styles.panel} aria-label="Start ingest">
      <header className={styles.header}>
        <Workflow size={16} />
        <span>Start Ingest</span>
      </header>
      <div className={styles.body}>
        <div className={styles.form}>
          <section className={styles.group}>
            <h2>Folder</h2>
            <p>Select the local folder to scan recursively for PDFs and images.</p>
            <div className={styles.actions}>
              <button
                className={styles.button}
                disabled={props.busy}
                onClick={props.onPickFolder}
                type="button"
              >
                <FolderOpen size={15} />
                Choose Folder
              </button>
            </div>
            <label className={styles.field}>
              <span>Folder path</span>
              <input
                onChange={(event) => props.onRootPathChange(event.target.value)}
                placeholder="Paste a folder path"
                value={props.rootPath}
              />
            </label>
          </section>
          <section className={styles.group}>
            <h2>OCR Configuration</h2>
            <label className={styles.field}>
              <span>Model</span>
              <select
                disabled={props.busy}
                onChange={(event) => props.onModelChange(event.target.value)}
                value={props.models?.selected_model_id ?? props.model?.model_id ?? ''}
              >
                {(props.models?.models ?? []).map((model) => (
                  <option key={model.model_id} value={model.model_id}>
                    {model.display_name}
                  </option>
                ))}
              </select>
            </label>
            <label className={styles.field}>
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
            <label className={styles.checkbox}>
              <input
                checked={reprocess}
                onChange={(event) => setReprocess(event.target.checked)}
                type="checkbox"
              />
              Reprocess completed compatible outputs
            </label>
          </section>
          <div className={styles.actions}>
            <button
              className={styles.primaryButton}
              disabled={!canStart}
              onClick={() => props.onStart({ reprocess })}
              type="button"
            >
              <Play size={15} />
              Start Ingest
            </button>
            <button
              className={styles.button}
              disabled={!active}
              onClick={props.onStop}
              type="button"
            >
              <Square size={15} />
              Stop Active Run
            </button>
          </div>
        </div>
        <RunStatus
          activeRun={props.activeRun}
          activeRunId={props.activeRunId}
          status={props.status}
        />
      </div>
    </section>
  );
}

export function isIngestBusy(status?: StatusPayload, activeRun?: IngestRunRecord) {
  const state = String(activeRun?.status ?? status?.state ?? '');
  return (
    Boolean(status?.active_run_id ?? activeRun?.run_id) && ['queued', 'running'].includes(state)
  );
}

function RunStatus({
  activeRun,
  activeRunId,
  status,
}: {
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  status?: StatusPayload;
}) {
  const progress = clampProgress(activeRun?.progress_percent);
  return (
    <aside className={styles.status}>
      <section className={styles.group}>
        <h2>Run Status</h2>
        <p>
          {isIngestBusy(status, activeRun)
            ? 'An ingest is active. Start is locked until it completes or stops.'
            : 'No ingest is currently blocking a new run.'}
        </p>
        <span
          aria-label={`Workflow progress ${percentLabel(activeRun?.progress_percent)}`}
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={Math.round(progress)}
          className={styles.progressTrack}
          role="progressbar"
        >
          <span style={{ width: `${progress}%` }} />
        </span>
        <dl className={styles.statusGrid}>
          <dt>State</dt>
          <dd>{activeRun?.status ?? status?.state ?? 'idle'}</dd>
          <dt>Run</dt>
          <dd>{activeRunId ?? activeRun?.run_id ?? 'No active run'}</dd>
          <dt>Pages</dt>
          <dd>{runPageLabel(activeRun)}</dd>
          <dt>Progress</dt>
          <dd>{percentLabel(activeRun?.progress_percent)}</dd>
          <dt>Profile</dt>
          <dd>{activeRun?.profile_id ?? status?.default_profile ?? 'default'}</dd>
          <dt>Runtime</dt>
          <dd>
            {status?.runtime_platform ?? 'runtime'} / {status?.accelerator ?? 'accelerator'}
          </dd>
        </dl>
      </section>
    </aside>
  );
}
