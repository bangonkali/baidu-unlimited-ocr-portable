import { FolderOpen } from 'lucide-react';

import type { OcrProfileRecord } from '../../api/types';
import styles from './IngestWizard.module.css';

interface IngestWizardSourceProps {
  busy?: boolean;
  folderDialogError?: string;
  profiles: OcrProfileRecord[];
  reprocess: boolean;
  rootPath: string;
  selectedProfile: string;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onReprocessChange: (value: boolean) => void;
  onRootPathChange: (value: string) => void;
}

export function IngestWizardSource({
  busy,
  folderDialogError,
  onPickFolder,
  onProfileChange,
  onReprocessChange,
  onRootPathChange,
  profiles,
  reprocess,
  rootPath,
  selectedProfile,
}: IngestWizardSourceProps) {
  return (
    <section className={styles.card} data-tour="folder">
      <h2>Source</h2>
      <p>Choose the folder to scan and the OCR profile for this run.</p>
      <div className={styles.actions}>
        <button className={styles.button} disabled={busy} onClick={onPickFolder} type="button">
          <FolderOpen size={15} />
          Choose Folder
        </button>
      </div>
      {folderDialogError ? (
        <p className={styles.error} role="alert">
          {folderDialogError}
        </p>
      ) : null}
      <label className={styles.field}>
        <span>Folder path</span>
        <input
          onChange={(event) => onRootPathChange(event.target.value)}
          placeholder="Paste a folder path"
          value={rootPath}
        />
      </label>
      <label className={styles.field}>
        <span>Profile</span>
        <select
          disabled={busy}
          onChange={(event) => onProfileChange(event.target.value)}
          value={selectedProfile}
        >
          {profiles.map((profile) => (
            <option key={profile.key} value={profile.key}>
              {profile.label}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.toggleRow}>
        <input
          checked={reprocess}
          onChange={(event) => onReprocessChange(event.target.checked)}
          type="checkbox"
        />
        <span className={styles.toggleText}>
          <strong>Reprocess completed compatible outputs</strong>
          <span className={styles.hint}>Use this when source files or OCR settings changed.</span>
        </span>
      </label>
    </section>
  );
}
