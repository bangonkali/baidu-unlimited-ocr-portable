import type { ModelAssetRecord } from '../../api/types';

export interface IngestWizardStartInput {
  embeddingAfterIngest: boolean;
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
    reprocess: input.reprocess,
    textIndexAfterIngest: input.textIndexAfterIngest,
  };
}
