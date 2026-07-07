import type { IngestEnginePresetRecord, ModelAssetRecord } from '../../api/types';
import type { EnginePlanItem } from './ingestEnginePlan';
import { enginePlanSelections } from './ingestEnginePlan';

export interface IngestWizardStartInput {
  embeddingAfterIngest: boolean;
  enginePlan?: EnginePlanItem[];
  enginePresets?: IngestEnginePresetRecord[];
  reprocess: boolean;
  selectedEmbeddingModel?: ModelAssetRecord;
  selectedEmbeddingModelId: string;
  textIndexAfterIngest: boolean;
}

export function buildIngestWizardStartOptions(input: IngestWizardStartInput) {
  return {
    embeddingAfterIngest: input.embeddingAfterIngest,
    embeddingDimension: input.selectedEmbeddingModel?.embedding_dimension ?? undefined,
    embeddingModelId: input.selectedEmbeddingModelId || undefined,
    engines:
      input.enginePresets && input.enginePresets.length > 0 && input.enginePlan
        ? enginePlanSelections(input.enginePlan)
        : undefined,
    reprocess: input.reprocess,
    textIndexAfterIngest: input.textIndexAfterIngest,
  };
}
