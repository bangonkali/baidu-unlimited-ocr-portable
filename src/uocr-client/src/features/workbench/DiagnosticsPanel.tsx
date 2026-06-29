import { CircleAlert, CircleCheck, LoaderCircle } from 'lucide-react';

import type { IngestRunRecord } from '../../api/types';
import styles from './DiagnosticsPanel.module.css';

interface DiagnosticsPanelProps {
  runs: IngestRunRecord[];
}

export function DiagnosticsPanel({ runs }: DiagnosticsPanelProps) {
  return (
    <section className={styles.panel} aria-label="Diagnostics">
      <div className={styles.header}>Diagnostics</div>
      <div className={styles.tabs}>
        <button className={styles.tab} type="button">
          Progress
        </button>
        <button className={styles.tab} type="button">
          Events
        </button>
        <button className={styles.tab} type="button">
          Models
        </button>
      </div>
      <div className={styles.body}>
        {runs.length === 0 ? <div className={styles.empty}>No runs</div> : null}
        {runs.map((run) => (
          <div className={styles.runRow} key={run.run_id}>
            {iconForStatus(run.status)}
            <span>{run.run_id}</span>
            <strong>{run.status}</strong>
            <small>{run.queued_files ?? 0} files</small>
          </div>
        ))}
      </div>
    </section>
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
