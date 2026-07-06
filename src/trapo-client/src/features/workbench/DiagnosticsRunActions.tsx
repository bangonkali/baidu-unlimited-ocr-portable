import { Play, RotateCcw, Square } from 'lucide-react';
import type { MouseEvent, ReactNode } from 'react';

import type { IngestRunRecord } from '../../api/types';
import styles from './DiagnosticsPanel.module.css';

export function DiagnosticsRunActions(props: {
  activeRunId?: string | null;
  hasActiveRun: boolean;
  run: IngestRunRecord;
  onResumeRun?: (runId: string) => void;
  onRestartRun?: (run: IngestRunRecord) => void;
  onStopRun?: (runId?: string) => void;
}) {
  const active = props.activeRunId === props.run.run_id || isActiveRunStatus(props.run.status);
  const canResume = Boolean(props.run.can_resume) && !props.hasActiveRun;
  const canRestart = Boolean(props.run.can_restart);
  return (
    <span className={styles.runActions}>
      {active ? (
        <RunActionButton
          icon={<Square size={13} />}
          label="Stop run"
          onClick={() => props.onStopRun?.(props.run.run_id)}
        />
      ) : null}
      {canResume ? (
        <RunActionButton
          icon={<Play size={13} />}
          label="Resume run"
          onClick={() => props.onResumeRun?.(props.run.run_id)}
        />
      ) : null}
      {canRestart ? (
        <RunActionButton
          icon={<RotateCcw size={13} />}
          label="Restart run"
          onClick={() => props.onRestartRun?.(props.run)}
        />
      ) : null}
    </span>
  );
}

export function isActiveRunStatus(status: string) {
  return status === 'queued' || status === 'running';
}

function RunActionButton(props: { icon: ReactNode; label: string; onClick: () => void }) {
  return (
    <button
      aria-label={props.label}
      className={styles.runAction}
      onClick={(event: MouseEvent<HTMLButtonElement>) => {
        event.stopPropagation();
        props.onClick();
      }}
      title={props.label}
      type="button"
    >
      {props.icon}
    </button>
  );
}
