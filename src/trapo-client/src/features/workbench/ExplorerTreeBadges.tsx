import { CircleAlert, CircleCheck, Clock3, LoaderCircle } from 'lucide-react';

import type { DocumentSummary } from '../../api/types';
import styles from './ExplorerTree.module.css';
import type { PipelineTaskActivity } from './pipelineTaskActivity';

export function documentBadge(status: string, pipelineActivity: PipelineTaskActivity | undefined) {
  return pipelineActivity ? (
    <PipelineActivityIcon activity={pipelineActivity} size={13} />
  ) : (
    <StatusIcon status={status} />
  );
}

export function pageBadge(
  document: DocumentSummary,
  pageNo: number,
  pipelineActivity: PipelineTaskActivity | undefined,
  runningEngineLabel?: string,
) {
  if (pipelineActivity) {
    return <PipelineActivityIcon activity={pipelineActivity} size={12} />;
  }
  if (runningEngineLabel) {
    return (
      <span className={styles.pageRunningEngine}>
        <span>{runningEngineLabel}</span>
        <LoaderCircle className={styles.spin} size={12} />
      </span>
    );
  }
  if (document.current_page === pageNo && document.status === 'running') {
    return <LoaderCircle className={styles.spin} size={12} />;
  }
  if ((document.processed_pages ?? 0) >= pageNo) {
    return <CircleCheck className={styles.ok} size={12} />;
  }
  return <Clock3 className={styles.queued} size={12} />;
}

function StatusIcon({ status }: { status: string }) {
  if (status === 'completed') {
    return <CircleCheck className={styles.ok} size={13} />;
  }
  if (status === 'failed' || status === 'completed_with_errors') {
    return <CircleAlert className={styles.bad} size={13} />;
  }
  if (status === 'running' || status === 'rendering') {
    return <LoaderCircle className={styles.spin} size={13} />;
  }
  return <Clock3 className={styles.queued} size={13} />;
}

function PipelineActivityIcon({
  activity,
  size,
}: {
  activity: PipelineTaskActivity;
  size: number;
}) {
  return (
    <LoaderCircle
      className={styles.activeTask}
      data-task-kind={activity.kind}
      data-task-status={activity.status}
      size={size}
    >
      <title>{activity.title}</title>
    </LoaderCircle>
  );
}
