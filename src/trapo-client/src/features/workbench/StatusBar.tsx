import { DownloadCloud, Power } from 'lucide-react';
import { useState } from 'react';

import { NotificationBell } from './NotificationBell';
import type { PipelineTaskActivity } from './pipelineTaskActivity';
import styles from './StatusBar.module.css';

interface StatusBarProps {
  downloadsActiveCount: number;
  downloadsOpen: boolean;
  documentCount: number;
  host: string;
  logPath?: string;
  realtimeState: string;
  runState: string;
  runtime: string;
  selectedRoot: string;
  onDownloadsToggle: () => void;
  onShutdown: () => void;
  pipelineTask?: PipelineTaskActivity;
  shutdownPending?: boolean;
}

export function StatusBar({
  downloadsActiveCount,
  downloadsOpen,
  documentCount,
  host,
  logPath,
  onDownloadsToggle,
  onShutdown,
  pipelineTask,
  realtimeState,
  runState,
  runtime,
  selectedRoot,
  shutdownPending,
}: StatusBarProps) {
  const [confirmingShutdown, setConfirmingShutdown] = useState(false);
  const confirmShutdown = () => {
    setConfirmingShutdown(false);
    onShutdown();
  };
  return (
    <footer className={styles.statusBar}>
      <span>{runState}</span>
      <span className={styles.realtime}>{realtimeState}</span>
      {pipelineTask ? (
        <span
          className={styles.pipelineTask}
          data-status={pipelineTask.status}
          title={pipelineTask.title}
        >
          <span className={styles.pipelineDot} />
          {pipelineTask.label} {pipelineTask.status}
        </span>
      ) : null}
      <button
        aria-label="Shut down Trapo"
        className={styles.shutdownButton}
        disabled={shutdownPending}
        onClick={() => setConfirmingShutdown(true)}
        title="Shut down Trapo"
        type="button"
      >
        <Power size={13} strokeWidth={1.9} />
      </button>
      <button
        aria-label={downloadsOpen ? 'Hide downloads' : 'Show downloads'}
        aria-pressed={downloadsOpen}
        className={styles.downloadsButton}
        data-open={downloadsOpen}
        onClick={onDownloadsToggle}
        title={downloadsOpen ? 'Hide downloads' : 'Show downloads'}
        type="button"
      >
        <DownloadCloud size={13} strokeWidth={1.9} />
        <span>Downloads</span>
        {downloadsActiveCount > 0 ? (
          <strong className={styles.downloadCount}>{downloadsActiveCount}</strong>
        ) : null}
      </button>
      <span>{documentCount} documents</span>
      <span>{host}</span>
      <span>{runtime}</span>
      <span className={styles.path}>{selectedRoot || logPath || 'No folder'}</span>
      <NotificationBell />
      {confirmingShutdown ? (
        <section className={styles.shutdownConfirm} aria-label="Confirm shutdown" role="dialog">
          <strong>Shut down Trapo?</strong>
          <span>OCR, downloads, and local database writes will be cancelled and flushed.</span>
          <div className={styles.shutdownActions}>
            <button onClick={() => setConfirmingShutdown(false)} type="button">
              Cancel
            </button>
            <button data-danger="true" onClick={confirmShutdown} type="button">
              Shut down
            </button>
          </div>
        </section>
      ) : null}
    </footer>
  );
}
