import { CircleAlert, CircleCheck, FileText, LoaderCircle } from 'lucide-react';

import type { IngestRunRecord, LogRecord } from '../../api/types';
import styles from './DiagnosticsPanel.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

export function RunList({ runs }: { runs: IngestRunRecord[] }) {
  return (
    <>
      {runs.length === 0 ? <div className={styles.empty}>No runs</div> : null}
      {runs.map((run) => (
        <div className={styles.runRow} key={run.run_id}>
          {iconForStatus(run.status)}
          <span>{run.run_id}</span>
          <strong>{run.status}</strong>
          <small>
            {runPageLabel(run)} · {percentLabel(run.progress_percent)}
          </small>
          <span
            aria-label={`Run ${run.run_id} progress ${percentLabel(run.progress_percent)}`}
            aria-valuemax={100}
            aria-valuemin={0}
            aria-valuenow={Math.round(clampProgress(run.progress_percent))}
            className={styles.progressTrack}
            role="progressbar"
          >
            <span style={{ width: `${clampProgress(run.progress_percent)}%` }} />
          </span>
        </div>
      ))}
    </>
  );
}

export function LogList({ logs }: { logs: LogRecord[] }) {
  return (
    <>
      {logs.length === 0 ? <div className={styles.empty}>No logs</div> : null}
      {logs.map((log) => (
        <div
          className={styles.logRow}
          key={`${log.timestamp}-${log.level}-${log.component}-${log.message}`}
        >
          <FileText size={14} />
          <span>{log.timestamp}</span>
          <strong data-level={log.level}>{log.level}</strong>
          <em>{log.component}</em>
          <p>{log.message}</p>
        </div>
      ))}
    </>
  );
}

function iconForStatus(status: string) {
  if (status === 'completed') {
    return <CircleCheck size={14} className={styles.ok} />;
  }
  if (status === 'failed' || status === 'cancelled') {
    return <CircleAlert size={14} className={styles.bad} />;
  }
  return <LoaderCircle size={14} className={styles.pending} />;
}
