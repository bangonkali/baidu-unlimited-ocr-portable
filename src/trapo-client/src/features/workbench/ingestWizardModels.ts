import type { ModelAssetRecord } from '../../api/types';

export const RECOMMENDED_EMBEDDING_MODEL_ID = 'nomic-embed-text-v1-5-q4-k-m';

export function ocrModels(models: ModelAssetRecord[]) {
  return models.filter((model) => model.model_kind !== 'embedding');
}

export function embeddingModels(models: ModelAssetRecord[]) {
  return models.filter((model) => model.model_kind === 'embedding');
}

export function isModelReady(model?: ModelAssetRecord) {
  return model?.status === 'downloaded';
}

export function isModelDownloading(model?: ModelAssetRecord) {
  return model ? ['queued', 'downloading', 'cancelling'].includes(model.status) : false;
}

export function recommendedOcrModel(
  models: ModelAssetRecord[],
  preferredModelId?: string,
  selectedModelId?: string,
) {
  const candidates = ocrModels(models);
  return (
    candidates.find((model) => model.model_id === preferredModelId && isModelReady(model)) ??
    candidates.find((model) => model.model_id === selectedModelId && isModelReady(model)) ??
    candidates.find((model) => model.selected && isModelReady(model)) ??
    candidates.find((model) => model.recommended) ??
    smallestModel(candidates)
  );
}

export function recommendedEmbeddingModel(models: ModelAssetRecord[], preferredModelId?: string) {
  const candidates = embeddingModels(models);
  return (
    candidates.find((model) => model.model_id === preferredModelId && isModelReady(model)) ??
    candidates.find(
      (model) => model.model_id === RECOMMENDED_EMBEDDING_MODEL_ID && isModelReady(model),
    ) ??
    candidates.find(isModelReady) ??
    candidates.find((model) => model.model_id === RECOMMENDED_EMBEDDING_MODEL_ID) ??
    smallestModel(candidates)
  );
}

export function modelRequiredBytes(model?: ModelAssetRecord) {
  return model?.total_required_bytes ?? model?.overall_total_bytes ?? model?.total_bytes ?? 0;
}

export function modelStatusLabel(model?: ModelAssetRecord) {
  if (!model) {
    return 'Missing';
  }
  if (model.status === 'downloaded') {
    return 'Ready';
  }
  if (model.status === 'queued') {
    return 'Queued';
  }
  if (model.status === 'downloading') {
    return 'Downloading';
  }
  if (model.status === 'cancelling') {
    return 'Cancelling';
  }
  if (model.status === 'failed') {
    return 'Failed';
  }
  if (model.status === 'cancelled') {
    return 'Cancelled';
  }
  return 'Missing';
}

export function formatModelBytes(bytes?: number | null) {
  if (!bytes || bytes <= 0) {
    return 'unknown size';
  }
  if (bytes >= 1024 ** 3) {
    return `${(bytes / 1024 ** 3).toFixed(1)}GB`;
  }
  return `${Math.max(1, Math.round(bytes / 1024 ** 2))}MB`;
}

function smallestModel(models: ModelAssetRecord[]) {
  return [...models].sort((left, right) => modelRequiredBytes(left) - modelRequiredBytes(right))[0];
}
