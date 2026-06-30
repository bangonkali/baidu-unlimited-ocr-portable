import { FolderOpen, Play, RefreshCw, Square } from 'lucide-react';
import type { ComponentType } from 'react';

import type { IngestRunRecord, OcrProfileRecord } from '../../api/types';
import styles from './IngestToolbar.module.css';
import { clampProgress, percentLabel, runPageLabel } from './progressFormat';

interface IngestToolbarProps {
  rootPath: string;
  profiles: OcrProfileRecord[];
  selectedProfile: string;
  activeRunId?: string | null;
  activeRun?: IngestRunRecord;
  busy?: boolean;
  modelReady?: boolean;
  runState?: string;
  supportedInputs?: string[];
  onPickFolder: () => void;
  onRootPathChange: (value: string) => void;
  onProfileChange: (value: string) => void;
  onStart: () => void;
  onStop: () => void;
  onRefresh: () => void;
}

export function IngestToolbar(props: IngestToolbarProps) {
  const canStop = Boolean(props.activeRunId) && props.runState === 'running';
  const canStart = Boolean(props.rootPath.trim()) && props.modelReady && !props.busy;

  return (
    <header className={styles.toolbar}>
      <div className={styles.rootPicker} data-tour="folder">
        <CommandButton icon={FolderOpen} label="Choose Folder" onClick={props.onPickFolder} />
        <input
          aria-label="Selected root"
          className={styles.pathInput}
          onChange={(event) => props.onRootPathChange(event.target.value)}
          placeholder="Or paste a folder path"
          value={props.rootPath}
        />
      </div>
      <select
        aria-label="OCR profile"
        className={styles.select}
        onChange={(event) => props.onProfileChange(event.target.value)}
        value={props.selectedProfile}
      >
        {props.profiles.map((profile) => (
          <option key={profile.key} value={profile.key}>
            {profile.label}
          </option>
        ))}
      </select>
      <div className={styles.actions}>
        <CommandButton
          disabled={!canStart}
          icon={Play}
          label="Start Scan"
          onClick={props.onStart}
          tour="start"
        />
        <CommandButton disabled={!canStop} icon={Square} label="Stop" onClick={props.onStop} />
        <CommandButton
          disabled={props.busy}
          icon={RefreshCw}
          label="Refresh"
          onClick={props.onRefresh}
        />
      </div>
      <WorkflowProgress run={props.activeRun} />
      <div className={styles.supported}>{(props.supportedInputs ?? []).join('  ')}</div>
    </header>
  );
}

function WorkflowProgress({ run }: { run?: IngestRunRecord }) {
  const progress = clampProgress(run?.progress_percent);
  return (
    <div className={styles.workflowProgress}>
      <span>{run ? `Workflow ${percentLabel(run.progress_percent)}` : 'Workflow idle'}</span>
      <span>{runPageLabel(run)}</span>
      <span
        aria-label={`Workflow progress ${percentLabel(run?.progress_percent)}`}
        aria-valuemax={100}
        aria-valuemin={0}
        aria-valuenow={Math.round(progress)}
        className={styles.progressTrack}
        role="progressbar"
      >
        <span style={{ width: `${progress}%` }} />
      </span>
    </div>
  );
}

function CommandButton(props: {
  disabled?: boolean;
  icon: ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  onClick?: () => void;
  tour?: string;
}) {
  const Icon = props.icon;
  return (
    <button
      className={styles.commandButton}
      data-tour={props.tour}
      disabled={props.disabled}
      onClick={props.onClick}
      type="button"
    >
      <Icon size={15} strokeWidth={1.9} />
      <span>{props.label}</span>
    </button>
  );
}
