import styles from './StatusBar.module.css';

interface StatusBarProps {
  documentCount: number;
  runState: string;
  selectedRoot: string;
}

export function StatusBar({ documentCount, runState, selectedRoot }: StatusBarProps) {
  return (
    <footer className={styles.statusBar}>
      <span>{runState}</span>
      <span>{documentCount} documents</span>
      <span className={styles.path}>{selectedRoot || 'No folder'}</span>
    </footer>
  );
}
