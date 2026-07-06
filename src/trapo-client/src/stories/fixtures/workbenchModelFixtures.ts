import type { ModelAssetRecord } from '../../api/types';

export const fixtureNomicEmbeddingModel: ModelAssetRecord = {
  bits: 4,
  context_tokens: 8192,
  display_name: 'Nomic Embed Text v1.5 Q4_K_M',
  downloaded_bytes: 0,
  downloaded_file_count: 0,
  embedding_dimension: 768,
  files: [
    {
      downloaded_bytes: 0,
      file_id: 'model',
      file_name: 'nomic-embed-text-v1.5.Q4_K_M.gguf',
      percent: 0,
      status: 'missing',
      total_bytes: 84_106_624,
    },
  ],
  hardware_tier: 'CPU / 4GB VRAM',
  model_id: 'nomic-embed-text-v1-5-q4-k-m',
  model_kind: 'embedding',
  notes: 'Small first embedding model for local RAG.',
  overall_downloaded_bytes: 0,
  overall_percent: 0,
  overall_total_bytes: 84_106_624,
  provider_name: 'Nomic AI',
  quality: 'Ultra-light MRL',
  quantization: 'Q4_K_M',
  recommended: true,
  repo_id: 'nomic-ai/nomic-embed-text-v1.5-GGUF',
  revision: 'main',
  routing_origin: 'embedding',
  selected: false,
  status: 'missing',
  total_file_count: 1,
  total_required_bytes: 84_106_624,
};

export const fixtureDownloadedEmbeddingModel: ModelAssetRecord = {
  ...fixtureNomicEmbeddingModel,
  downloaded_bytes: 84_106_624,
  downloaded_file_count: 1,
  files: [
    {
      downloaded_bytes: 84_106_624,
      file_id: 'model',
      file_name: 'nomic-embed-text-v1.5.Q4_K_M.gguf',
      percent: 100,
      status: 'downloaded',
      total_bytes: 84_106_624,
    },
  ],
  overall_downloaded_bytes: 84_106_624,
  overall_percent: 100,
  status: 'downloaded',
};
