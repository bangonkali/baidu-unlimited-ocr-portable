import { FolderOpen, Pause, Play, RefreshCw, Square } from 'lucide-react';
import type { OcrProfileRecord } from '../../api/types';
import { IconButton } from '../../components/IconButton';
import styles from './IngestToolbar.module.css';

interface IngestToolbarProps {
  rootPath: string;
  profiles: OcrProfileRecord[];
  selectedProfile: string;
  activeRunId?: string | null;
  busy?: boolean;
  onPickFolder: () => void;
  onRootPathChange: (value: string) => void;
  onProfileChange: (value: string) => void;
  onStart: () => void;
  onPause: () => void;
  onStop: () => void;
}

export function IngestToolbar(props: IngestToolbarProps) {
  const canControl = Boolean(props.activeRunId);

  return (
    <header className={styles.toolbar}>
      <div className={styles.rootPicker}>
        <IconButton icon={FolderOpen} label="Pick folder" onClick={props.onPickFolder} />
        <input
          aria-label="Selected root"
          className={styles.pathInput}
          onChange={(event) => props.onRootPathChange(event.target.value)}
          placeholder="Folder path"
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
        <IconButton
          disabled={props.busy || !props.rootPath}
          icon={Play}
          label="Start"
          onClick={props.onStart}
        />
        <IconButton disabled={!canControl} icon={Pause} label="Pause" onClick={props.onPause} />
        <IconButton disabled={!canControl} icon={Square} label="Stop" onClick={props.onStop} />
        <IconButton disabled={props.busy} icon={RefreshCw} label="Refresh" />
      </div>
    </header>
  );
}
