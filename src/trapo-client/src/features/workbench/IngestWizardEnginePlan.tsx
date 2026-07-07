import { ArrowDown, ArrowUp, Copy, Plus, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';

import type { IngestEnginePresetRecord, ModelAssetRecord } from '../../api/types';
import wizardStyles from './IngestWizard.module.css';
import styles from './IngestWizardEnginePlan.module.css';
import { EnginePlanModelActions } from './IngestWizardEnginePlanModelActions';
import type { EnginePlanItem } from './ingestEnginePlan';
import { duplicateEnginePlanItem, enginePlanItemFromPreset, presetById } from './ingestEnginePlan';

interface IngestWizardEnginePlanProps {
  busy?: boolean;
  models: ModelAssetRecord[];
  plan: EnginePlanItem[];
  presets: IngestEnginePresetRecord[];
  selectedProfile: string;
  selectedRuntimeId?: string;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onPlanChange: (plan: EnginePlanItem[]) => void;
}

export function IngestWizardEnginePlan(props: IngestWizardEnginePlanProps) {
  const firstPresetId = props.presets[0]?.preset_id ?? '';
  const [addPresetId, setAddPresetId] = useState(firstPresetId);
  const normalizedAddPresetId = props.presets.some((preset) => preset.preset_id === addPresetId)
    ? addPresetId
    : firstPresetId;
  const presetOptions = useMemo(
    () =>
      props.presets.map((preset) => (
        <option key={preset.preset_id} value={preset.preset_id}>
          {preset.label} - {availabilityLabel(preset)}
        </option>
      )),
    [props.presets],
  );
  if (props.presets.length === 0) {
    return null;
  }
  const addPreset = () => {
    const preset = presetById(props.presets, normalizedAddPresetId);
    if (!preset) {
      return;
    }
    props.onPlanChange([
      ...props.plan,
      enginePlanItemFromPreset(preset, props.selectedProfile, props.selectedRuntimeId),
    ]);
  };
  return (
    <section className={wizardStyles.card}>
      <h2>Engine Plan</h2>
      <div className={styles.addRow}>
        <select
          aria-label="Add engine"
          disabled={props.busy}
          onChange={(event) => setAddPresetId(event.target.value)}
          value={normalizedAddPresetId}
        >
          {presetOptions}
        </select>
        <button
          className={wizardStyles.button}
          disabled={props.busy}
          onClick={addPreset}
          type="button"
        >
          <Plus size={15} />
          Add
        </button>
      </div>
      <div className={styles.planRows}>
        {props.plan.map((item, index) => (
          <EnginePlanRow
            busy={props.busy}
            index={index}
            item={item}
            key={item.planKey}
            models={props.models}
            presets={props.presets}
            selectedProfile={props.selectedProfile}
            selectedRuntimeId={props.selectedRuntimeId}
            total={props.plan.length}
            onCancelModel={props.onCancelModel}
            onDownloadModel={props.onDownloadModel}
            onPlanChange={props.onPlanChange}
            plan={props.plan}
          />
        ))}
      </div>
    </section>
  );
}

function EnginePlanRow({
  busy,
  index,
  item,
  models,
  onCancelModel,
  onDownloadModel,
  onPlanChange,
  plan,
  presets,
  selectedProfile,
  selectedRuntimeId,
  total,
}: {
  busy?: boolean;
  index: number;
  item: EnginePlanItem;
  models: ModelAssetRecord[];
  plan: EnginePlanItem[];
  presets: IngestEnginePresetRecord[];
  selectedProfile: string;
  selectedRuntimeId?: string;
  total: number;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onPlanChange: (plan: EnginePlanItem[]) => void;
}) {
  const preset = presetById(presets, item.presetId) ?? presets[0];
  const updateItem = (next: EnginePlanItem) =>
    onPlanChange(
      plan.map((candidate, candidateIndex) => (candidateIndex === index ? next : candidate)),
    );
  const move = (offset: number) => {
    const nextIndex = index + offset;
    if (nextIndex < 0 || nextIndex >= total) {
      return;
    }
    const next = [...plan];
    const currentItem = next[index];
    const targetItem = next[nextIndex];
    if (!currentItem || !targetItem) {
      return;
    }
    next[index] = targetItem;
    next[nextIndex] = currentItem;
    onPlanChange(next);
  };
  return (
    <article className={styles.planRow}>
      <div className={styles.rowHeader}>
        <select
          aria-label={`Engine ${index + 1}`}
          disabled={busy}
          onChange={(event) => {
            const nextPreset = presetById(presets, event.target.value);
            if (nextPreset) {
              updateItem(enginePlanItemFromPreset(nextPreset, selectedProfile, selectedRuntimeId));
            }
          }}
          value={item.presetId}
        >
          {presets.map((option) => (
            <option key={option.preset_id} value={option.preset_id}>
              {option.label}
            </option>
          ))}
        </select>
        <div className={styles.rowActions}>
          <button
            className={styles.iconButton}
            disabled={busy || index === 0}
            onClick={() => move(-1)}
            title="Move up"
            type="button"
          >
            <ArrowUp size={14} />
          </button>
          <button
            className={styles.iconButton}
            disabled={busy || index === total - 1}
            onClick={() => move(1)}
            title="Move down"
            type="button"
          >
            <ArrowDown size={14} />
          </button>
          <button
            className={styles.iconButton}
            disabled={busy}
            onClick={() =>
              onPlanChange([
                ...plan.slice(0, index + 1),
                duplicateEnginePlanItem(item),
                ...plan.slice(index + 1),
              ])
            }
            title="Duplicate"
            type="button"
          >
            <Copy size={14} />
          </button>
          <button
            className={styles.iconButton}
            disabled={busy}
            onClick={() =>
              onPlanChange(plan.filter((_, candidateIndex) => candidateIndex !== index))
            }
            title="Remove"
            type="button"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>
      <div className={styles.statusLine}>
        <span className={wizardStyles.statusBadge} data-ready={preset?.available}>
          {preset ? availabilityLabel(preset) : 'missing preset'}
        </span>
        <span className={wizardStyles.meta}>{preset?.description}</span>
      </div>
      {preset ? (
        <EnginePlanModelActions
          busy={busy}
          models={models}
          onCancelModel={onCancelModel}
          onDownloadModel={onDownloadModel}
          preset={preset}
        />
      ) : null}
      <label className={styles.paramsField}>
        <span>Parameters</span>
        <textarea
          disabled={busy}
          onChange={(event) => updateItem({ ...item, parametersJson: event.target.value })}
          rows={4}
          spellCheck={false}
          value={item.parametersJson}
        />
      </label>
    </article>
  );
}

function availabilityLabel(preset: IngestEnginePresetRecord) {
  if (preset.available) {
    return preset.availability === 'fallback_adapter' ? 'fallback adapter' : 'ready';
  }
  return preset.availability.replaceAll('_', ' ');
}
