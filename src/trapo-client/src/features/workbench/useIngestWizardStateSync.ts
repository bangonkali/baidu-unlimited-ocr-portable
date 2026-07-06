import { useEffect } from 'react';

import type { ModelAssetRecord } from '../../api/types';
import type { IngestRouteSearch } from '../../routeSearch';

interface IngestWizardStateSyncArgs {
  ingestSearch?: IngestRouteSearch;
  latestRunId?: string;
  recommendedEmbedding?: ModelAssetRecord;
  selectedEmbeddingModelId: string;
  selectedRunId: string;
  setEmbeddingAfterIngest: (value: boolean) => void;
  setReprocess: (value: boolean) => void;
  setSelectedEmbeddingModelId: (value: string) => void;
  setSelectedRunId: (value: string) => void;
  setTextIndexAfterIngest: (value: boolean) => void;
}

export function useIngestWizardStateSync(args: IngestWizardStateSyncArgs) {
  useEffect(() => {
    args.setReprocess(args.ingestSearch?.reprocess ?? false);
  }, [args.ingestSearch?.reprocess, args.setReprocess]);
  useEffect(() => {
    args.setTextIndexAfterIngest(args.ingestSearch?.index ?? true);
  }, [args.ingestSearch?.index, args.setTextIndexAfterIngest]);
  useEffect(() => {
    if (args.ingestSearch?.embed !== undefined) {
      args.setEmbeddingAfterIngest(args.ingestSearch.embed);
    }
  }, [args.ingestSearch?.embed, args.setEmbeddingAfterIngest]);
  useEffect(() => {
    if (args.latestRunId && !args.selectedRunId) {
      args.setSelectedRunId(args.latestRunId);
    }
  }, [args.latestRunId, args.selectedRunId, args.setSelectedRunId]);
  useEffect(() => {
    const nextModelId = args.ingestSearch?.embedding_model ?? args.recommendedEmbedding?.model_id;
    if (nextModelId && !args.selectedEmbeddingModelId) {
      args.setSelectedEmbeddingModelId(nextModelId);
    }
  }, [
    args.ingestSearch?.embedding_model,
    args.recommendedEmbedding,
    args.selectedEmbeddingModelId,
    args.setSelectedEmbeddingModelId,
  ]);
}
