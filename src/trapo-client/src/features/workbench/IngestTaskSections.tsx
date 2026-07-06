import { BrainCircuit, FileText, FolderOpen, Play, Square } from 'lucide-react';

import type { IngestRunRecord, ModelAssetRecord, OcrProfileRecord } from '../../api/types';
import styles from './IngestStartPanel.module.css';
import { EmbeddingModelSelect, shortRunLabel } from './IngestStartPanelParts';

export function FolderSection({
  busy,
  folderDialogError,
  rootPath,
  onPickFolder,
  onRootPathChange,
}: {
  busy?: boolean;
  folderDialogError?: string;
  rootPath: string;
  onPickFolder: () => void;
  onRootPathChange: (value: string) => void;
}) {
  return (
    <section className={styles.group} data-tour="folder">
      <h2>Folder</h2>
      <p>Select the local folder to scan recursively for PDFs and images.</p>
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
    </section>
  );
}

export function OcrConfigurationSection({
  busy,
  embeddingAfterIngest,
  embeddingModels,
  modelValue,
  ocrModels,
  profiles,
  reprocess,
  selectedEmbeddingModelId,
  selectedProfile,
  textIndexAfterIngest,
  onEmbeddingAfterIngestChange,
  onEmbeddingModelChange,
  onModelChange,
  onProfileChange,
  onReprocessChange,
  onTextIndexAfterIngestChange,
}: {
  busy?: boolean;
  embeddingAfterIngest: boolean;
  embeddingModels: ModelAssetRecord[];
  modelValue: string;
  ocrModels: ModelAssetRecord[];
  profiles: OcrProfileRecord[];
  reprocess: boolean;
  selectedEmbeddingModelId: string;
  selectedProfile: string;
  textIndexAfterIngest: boolean;
  onEmbeddingAfterIngestChange: (value: boolean) => void;
  onEmbeddingModelChange: (modelId: string) => void;
  onModelChange: (modelId: string) => void;
  onProfileChange: (profileId: string) => void;
  onReprocessChange: (value: boolean) => void;
  onTextIndexAfterIngestChange: (value: boolean) => void;
}) {
  return (
    <section className={styles.group}>
      <h2>OCR Configuration</h2>
      <label className={styles.field}>
        <span>Model</span>
        <select
          disabled={busy}
          onChange={(event) => onModelChange(event.target.value)}
          value={modelValue}
        >
          {ocrModels.map((model) => (
            <option key={model.model_id} value={model.model_id}>
              {model.display_name}
            </option>
          ))}
        </select>
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
      <ToggleRow
        checked={reprocess}
        label="Reprocess completed compatible outputs"
        onChange={onReprocessChange}
      />
      <ToggleRow
        checked={textIndexAfterIngest}
        label="Run Text Index after ingest"
        onChange={onTextIndexAfterIngestChange}
      />
      <ToggleRow
        checked={embeddingAfterIngest}
        label="Generate Embedding after ingest"
        onChange={onEmbeddingAfterIngestChange}
      />
      {embeddingAfterIngest ? (
        <EmbeddingModelSelect
          busy={busy}
          models={embeddingModels}
          selectedModelId={selectedEmbeddingModelId}
          onChange={onEmbeddingModelChange}
        />
      ) : null}
    </section>
  );
}

export function RunTaskSection({
  busy,
  canGenerateEmbedding,
  canRunPostStep,
  embeddingModels,
  runs,
  selectedEmbeddingModelId,
  selectedRunId,
  onEmbeddingModelChange,
  onGenerateEmbedding,
  onRunChange,
  onStartTextIndex,
}: {
  busy?: boolean;
  canGenerateEmbedding: boolean;
  canRunPostStep: boolean;
  embeddingModels: ModelAssetRecord[];
  runs: IngestRunRecord[];
  selectedEmbeddingModelId: string;
  selectedRunId: string;
  onEmbeddingModelChange: (modelId: string) => void;
  onGenerateEmbedding: () => void;
  onRunChange: (runId: string) => void;
  onStartTextIndex: () => void;
}) {
  return (
    <section className={styles.group}>
      <h2>Run Task</h2>
      <label className={styles.field}>
        <span>Source run</span>
        <select
          disabled={busy || runs.length === 0}
          onChange={(event) => onRunChange(event.target.value)}
          value={selectedRunId}
        >
          <option value="">Select a completed ingest run</option>
          {runs.map((run) => (
            <option key={run.run_id} value={run.run_id}>
              {shortRunLabel(run)}
            </option>
          ))}
        </select>
      </label>
      <EmbeddingModelSelect
        busy={busy}
        models={embeddingModels}
        selectedModelId={selectedEmbeddingModelId}
        onChange={onEmbeddingModelChange}
      />
      <div className={styles.actions}>
        <button
          className={styles.button}
          disabled={!canRunPostStep}
          onClick={onStartTextIndex}
          type="button"
        >
          <FileText size={15} />
          Text Index
        </button>
        <button
          className={styles.button}
          disabled={!canGenerateEmbedding}
          onClick={onGenerateEmbedding}
          type="button"
        >
          <BrainCircuit size={15} />
          Generate Embedding
        </button>
      </div>
    </section>
  );
}

export function StartStopActions({
  active,
  canStart,
  onStart,
  onStop,
}: {
  active: boolean;
  canStart: boolean;
  onStart: () => void;
  onStop: () => void;
}) {
  return (
    <div className={styles.actions}>
      <button
        className={styles.primaryButton}
        data-tour="start"
        disabled={!canStart}
        onClick={onStart}
        type="button"
      >
        <Play size={15} />
        Start Ingest
      </button>
      <button className={styles.button} disabled={!active} onClick={onStop} type="button">
        <Square size={15} />
        Stop Active Run
      </button>
    </div>
  );
}

function ToggleRow({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className={styles.checkbox}>
      <input
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
      {label}
    </label>
  );
}
