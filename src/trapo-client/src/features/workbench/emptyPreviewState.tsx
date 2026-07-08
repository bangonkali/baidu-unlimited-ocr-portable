import styles from './EmptyPreviewState.module.css';

interface EmptyPreviewStateProps {
  detail?: string;
  title: string;
}

export function EmptyPreviewState({ detail, title }: EmptyPreviewStateProps) {
  return (
    <div className={styles.emptyState}>
      <div className={styles.title}>{title}</div>
      {detail ? <div className={styles.detail}>{detail}</div> : null}
    </div>
  );
}
