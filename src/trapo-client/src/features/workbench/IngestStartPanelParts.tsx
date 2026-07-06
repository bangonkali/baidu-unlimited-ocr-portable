import type { IngestRunRecord, ModelAssetRecord, StatusPayload } from '../../api/types';
import styles from './IngestStartPanel.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

export function EmbeddingModelSelect({
  busy,
  models,
  selectedModelId,
  onChange,
}: {
  busy?: boolean;
  models: ModelAssetRecord[];
  selectedModelId: string;
  onChange: (modelId: string) => void;
}) {
  return (
    <label className={styles.field}>
      <span>Embedding model</span>
      <select
        disabled={busy || models.length === 0}
        onChange={(event) => onChange(event.target.value)}
        value={selectedModelId}
      >
        <option value="">Select a downloaded embedding model</option>
        {models.map((model) => (
          <option key={model.model_id} value={model.model_id}>
            {model.display_name}
          </option>
        ))}
      </select>
    </label>
  );
}

export function RunStatus({
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

export function isIngestBusy(status?: StatusPayload, activeRun?: IngestRunRecord) {
  const state = String(activeRun?.status ?? status?.state ?? '');
  return (
    Boolean(status?.active_run_id ?? activeRun?.run_id) && ['queued', 'running'].includes(state)
  );
}

export function latestCompletedRunId(runs: IngestRunRecord[]) {
  return runs.find((run) => run.status === 'completed')?.run_id ?? runs[0]?.run_id;
}

export function shortRunLabel(run: IngestRunRecord) {
  const shortId = run.run_id.length > 8 ? run.run_id.slice(0, 8) : run.run_id;
  return `${shortId} - ${run.status} - ${run.root_path}`;
}
