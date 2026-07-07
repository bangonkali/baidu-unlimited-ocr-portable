import { useEffect, useRef, useState } from 'react';

import type { IngestEnginePresetRecord, IngestRunRecord, ModelAssetRecord } from '../../api/types';
import type { EnginePlanItem } from './ingestEnginePlan';
import {
  defaultEnginePlan,
  enginePlanFromPresetIds,
  enginePlanFromRunConfigs,
  enginePlanIssue,
  enginePlanReady,
} from './ingestEnginePlan';

export function useEnginePlanState(args: {
  enginePresets: IngestEnginePresetRecord[];
  models: ModelAssetRecord[];
  presetIds?: string[];
  restartRun?: IngestRunRecord;
  restartRunId?: string;
  runsReady: boolean;
  selectedProfile: string;
  selectedRuntimeId?: string;
}) {
  const {
    enginePresets,
    models,
    presetIds,
    restartRun,
    restartRunId,
    runsReady,
    selectedProfile,
    selectedRuntimeId,
  } = args;
  const [enginePlan, setEnginePlan] = useState<EnginePlanItem[]>([]);
  const initializedEnginePlanRef = useRef(false);
  useEffect(() => {
    if (initializedEnginePlanRef.current || enginePresets.length === 0) {
      return;
    }
    if (restartRunId && !restartRun && !runsReady) {
      return;
    }
    initializedEnginePlanRef.current = true;
    setEnginePlan(
      initialEnginePlan({
        enginePresets,
        presetIds,
        restartRun,
        selectedProfile,
        selectedRuntimeId,
      }),
    );
  }, [
    enginePresets,
    presetIds,
    restartRun,
    restartRunId,
    runsReady,
    selectedProfile,
    selectedRuntimeId,
  ]);

  useEffect(() => {
    setEnginePlan((current) =>
      current.map((item) =>
        item.profileId && item.profileId !== selectedProfile
          ? { ...item, profileId: selectedProfile }
          : item,
      ),
    );
  }, [selectedProfile]);

  return {
    enginePlan,
    planIssue: enginePlanIssue(enginePlan, enginePresets, models),
    planReady: enginePlanReady(enginePlan, enginePresets, models),
    setEnginePlan,
  };
}

function initialEnginePlan(args: {
  enginePresets: IngestEnginePresetRecord[];
  presetIds?: string[];
  restartRun?: IngestRunRecord;
  selectedProfile: string;
  selectedRuntimeId?: string;
}) {
  const fromRun = args.restartRun?.engine_configs?.length
    ? enginePlanFromRunConfigs(
        args.restartRun.engine_configs,
        args.enginePresets,
        args.selectedProfile,
        args.selectedRuntimeId,
      )
    : [];
  if (fromRun.length > 0) {
    return fromRun;
  }
  const fromQuery = args.presetIds?.length
    ? enginePlanFromPresetIds(
        args.presetIds,
        args.enginePresets,
        args.selectedProfile,
        args.selectedRuntimeId,
      )
    : [];
  return fromQuery.length > 0
    ? fromQuery
    : defaultEnginePlan(args.enginePresets, args.selectedProfile, args.selectedRuntimeId);
}
