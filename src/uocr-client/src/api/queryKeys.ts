export const queryKeys = {
  documents: (q: string) => ['documents', q] as const,
  documentPreviewImages: (fileHash?: string) => ['document-preview-images', fileHash] as const,
  documentRegions: (fileHash?: string) => ['document-regions', fileHash] as const,
  documentText: (fileHash?: string) => ['document-text', fileHash] as const,
  logs: ['logs'] as const,
  models: ['models'] as const,
  ocrMetrics: ['ocr-metrics'] as const,
  runs: ['runs'] as const,
  settings: ['settings'] as const,
  status: ['status'] as const,
};
