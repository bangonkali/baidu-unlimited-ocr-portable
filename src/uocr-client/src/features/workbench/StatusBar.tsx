import styles from './StatusBar.module.css';

interface StatusBarProps {
  documentCount: number;
  host: string;
  logPath?: string;
  runState: string;
  runtime: string;
  selectedRoot: string;
}

export function StatusBar({
  documentCount,
  host,
  logPath,
  runState,
  runtime,
  selectedRoot,
}: StatusBarProps) {
  return (
    <footer className={styles.statusBar}>
      <span>{runState}</span>
      <span>{documentCount} documents</span>
      <span>{host}</span>
      <span>{runtime}</span>
      <span className={styles.path}>{selectedRoot || logPath || 'No folder'}</span>
    </footer>
  );
}
