import { Database, FolderOpen, Play } from 'lucide-react';
import type { ReactNode } from 'react';

import type { ModelAssetRecord } from '../../api/types';
import styles from './StartHere.module.css';

interface StartHereProps {
  model?: ModelAssetRecord;
  rootPath: string;
  onOpenModels: () => void;
  onPickFolder: () => void;
  onStart: () => void;
}

export function StartHere(props: StartHereProps) {
  const modelReady = props.model?.status === 'downloaded';
  const folderReady = props.rootPath.trim().length > 0;

  return (
    <section className={styles.panel} aria-label="Start here">
      <Step
        active={!modelReady}
        icon={<Database size={15} />}
        label="1. Download model"
        onClick={props.onOpenModels}
        value={props.model?.status ?? 'missing'}
      />
      <Step
        active={modelReady && !folderReady}
        icon={<FolderOpen size={15} />}
        label="2. Choose folder"
        onClick={props.onPickFolder}
        value={folderReady ? props.rootPath : 'No folder selected'}
      />
      <Step
        active={modelReady && folderReady}
        disabled={!modelReady || !folderReady}
        icon={<Play size={15} />}
        label="3. Start scan"
        onClick={props.onStart}
        value="PDF and images"
      />
    </section>
  );
}

function Step(props: {
  active: boolean;
  disabled?: boolean;
  icon: ReactNode;
  label: string;
  value: string;
  onClick: () => void;
}) {
  return (
    <button
      className={styles.step}
      data-active={props.active}
      disabled={props.disabled}
      onClick={props.onClick}
      type="button"
    >
      {props.icon}
      <span>{props.label}</span>
      <small>{props.value}</small>
    </button>
  );
}
