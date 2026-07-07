import type {
  IngestEnginePresetRecord,
  IngestEngineSelection,
  ModelAssetRecord,
} from '../../api/types';
import { isModelReady } from './ingestWizardModels';

export interface EnginePlanItem {
  engineId: string;
  engineKind: string;
  modelId?: string | null;
  parametersJson: string;
  planKey: string;
  presetId: string;
  profileId?: string | null;
  runtimeId?: string | null;
}

export function defaultEnginePlan(
  presets: IngestEnginePresetRecord[],
  selectedProfile: string,
  selectedRuntimeId?: string,
) {
  return presets
    .filter((preset) => preset.default_enabled)
    .map((preset) => enginePlanItemFromPreset(preset, selectedProfile, selectedRuntimeId));
}

export function enginePlanItemFromPreset(
  preset: IngestEnginePresetRecord,
  selectedProfile: string,
  selectedRuntimeId?: string,
): EnginePlanItem {
  return {
    engineId: preset.engine_id,
    engineKind: preset.engine_kind,
    modelId: preset.model_id,
    parametersJson: stableJson(preset.default_parameters),
    planKey: nextEnginePlanKey(preset.preset_id),
    presetId: preset.preset_id,
    profileId: preset.profile_id ?? selectedProfile,
    runtimeId: preset.runtime_id ?? selectedRuntimeId,
  };
}

export function enginePlanSelections(plan: EnginePlanItem[]): IngestEngineSelection[] {
  return plan.map((item, index) => ({
    engine_id: item.engineId,
    engine_kind: item.engineKind,
    model_id: item.modelId,
    ordinal: index,
    parameters: parseParameters(item.parametersJson) ?? {},
    preset_id: item.presetId,
    profile_id: item.profileId,
    runtime_id: item.runtimeId,
  }));
}

export function duplicateEnginePlanItem(item: EnginePlanItem): EnginePlanItem {
  return {
    ...item,
    planKey: nextEnginePlanKey(item.presetId),
  };
}

export function enginePlanReady(
  plan: EnginePlanItem[],
  presets: IngestEnginePresetRecord[],
  models: ModelAssetRecord[],
) {
  if (presets.length === 0) {
    return true;
  }
  if (plan.length === 0) {
    return false;
  }
  return plan.every((item) => {
    const preset = presetById(presets, item.presetId);
    return Boolean(
      preset?.available &&
        parseParameters(item.parametersJson) &&
        modelRequirementsReady(preset, models),
    );
  });
}

export function enginePlanIssue(
  plan: EnginePlanItem[],
  presets: IngestEnginePresetRecord[],
  models: ModelAssetRecord[],
) {
  if (presets.length === 0) {
    return undefined;
  }
  if (plan.length === 0) {
    return 'Add at least one engine.';
  }
  const invalidJson = plan.find((item) => parseParameters(item.parametersJson) === undefined);
  if (invalidJson) {
    return 'Fix the parameter JSON before starting.';
  }
  const unavailable = plan
    .map((item) => presetById(presets, item.presetId))
    .find((preset) => preset && (!preset.available || !modelRequirementsReady(preset, models)));
  if (unavailable) {
    return unavailable.availability_detail ?? `${unavailable.label} is not ready.`;
  }
  return undefined;
}

export function presetById(presets: IngestEnginePresetRecord[], presetId: string) {
  return presets.find((preset) => preset.preset_id === presetId);
}

function modelRequirementsReady(preset: IngestEnginePresetRecord, models: ModelAssetRecord[]) {
  if (!preset.requires_model) {
    return true;
  }
  return preset.download_model_ids.every((modelId) =>
    isModelReady(models.find((model) => model.model_id === modelId)),
  );
}

function parseParameters(value: string) {
  try {
    const parsed = JSON.parse(value) as unknown;
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : undefined;
  } catch {
    return undefined;
  }
}

function stableJson(value: Record<string, unknown>) {
  return JSON.stringify(sortJsonValue(value), null, 2);
}

let enginePlanKeyCounter = 0;

function nextEnginePlanKey(presetId: string) {
  enginePlanKeyCounter += 1;
  return `${presetId}:${enginePlanKeyCounter}`;
}

function sortJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortJsonValue);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, nested]) => [key, sortJsonValue(nested)]),
    );
  }
  return value;
}
