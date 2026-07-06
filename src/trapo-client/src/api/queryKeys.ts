export const queryKeys = {
  documents: (q: string) => ['documents', q] as const,
  documentPreviewImages: (fileHash?: string) => ['document-preview-images', fileHash] as const,
  documentRegions: (fileHash?: string, runId?: string) =>
    ['document-regions', fileHash, runId ?? 'latest'] as const,
  documentText: (fileHash?: string, runId?: string) =>
    ['document-text', fileHash, runId ?? 'latest'] as const,
  logs: ['logs'] as const,
  models: ['models'] as const,
  ocrReplay: (params: Record<string, unknown>) => ['ocr-replay', params] as const,
  diagnosticRuns: ['diagnostics', 'runs'] as const,
  diagnosticTrace: (params: Record<string, unknown>) => ['diagnostics', 'trace', params] as const,
  diagnosticWaterfall: (params: Record<string, unknown>) =>
    ['diagnostics', 'waterfall', params] as const,
  diagnosticProgress: (runId?: string, limit = 5000) =>
    ['diagnostics', 'progress', runId ?? 'latest', limit] as const,
  diagnosticAnalytics: (runId?: string) => ['diagnostics', 'analytics', runId ?? 'latest'] as const,
  diagnosticModels: (runId?: string) => ['diagnostics', 'models', runId ?? 'latest'] as const,
  ragEmbeddingModelsUsed: ['rag', 'embedding-models', 'used'] as const,
  ragSearch: (params: Record<string, unknown>) => ['rag', 'search', params] as const,
  runs: ['runs'] as const,
  settings: ['settings'] as const,
  status: ['status'] as const,
};
