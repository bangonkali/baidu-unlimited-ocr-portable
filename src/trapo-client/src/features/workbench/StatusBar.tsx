import { DownloadCloud } from 'lucide-react';

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
}

export function StatusBar({
  downloadsActiveCount,
  downloadsOpen,
  documentCount,
  host,
  logPath,
  onDownloadsToggle,
  realtimeState,
  runState,
  runtime,
  selectedRoot,
}: StatusBarProps) {
  return (
    <footer className={styles.statusBar}>
      <span>{runState}</span>
      <span className={styles.realtime}>{realtimeState}</span>
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
    </footer>
  );
}
