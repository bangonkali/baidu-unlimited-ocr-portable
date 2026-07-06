import type { QueryClient } from '@tanstack/react-query';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { getJson, postJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  GenerateEmbeddingRequest,
  GenerateEmbeddingResponse,
  HybridSearchRequest,
  HybridSearchResponse,
  TextIndexRequest,
  TextIndexResponse,
  UsedEmbeddingModelsPayload,
} from './types';

export function useStartTextIndex() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: TextIndexRequest) =>
      postJson<TextIndexResponse, TextIndexRequest>('/api/rag/text-index', body),
    onSuccess: () => {
      invalidateRagState(queryClient);
    },
  });
}

export function useGenerateEmbedding() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: GenerateEmbeddingRequest) =>
      postJson<GenerateEmbeddingResponse, GenerateEmbeddingRequest>('/api/rag/embeddings', body),
    onSuccess: () => {
      invalidateRagState(queryClient);
    },
  });
}

export function useUsedEmbeddingModels() {
  return useQuery({
    placeholderData: { models: [] },
    queryFn: ({ signal }) =>
      getJson<UsedEmbeddingModelsPayload>('/api/rag/embedding-models/used', signal),
    queryKey: queryKeys.ragEmbeddingModelsUsed,
  });
}

export function useHybridSearch(request: HybridSearchRequest, enabled: boolean) {
  return useQuery({
    enabled,
    placeholderData: { files: [], query: request.query },
    queryFn: ({ signal }) =>
      postJson<HybridSearchResponse, HybridSearchRequest>('/api/rag/search', request, signal),
    queryKey: queryKeys.ragSearch({ ...request }),
  });
}

function invalidateRagState(queryClient: QueryClient) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.status });
  void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
  void queryClient.invalidateQueries({ queryKey: queryKeys.diagnosticRuns });
  void queryClient.invalidateQueries({ queryKey: queryKeys.ragEmbeddingModelsUsed });
  void queryClient.invalidateQueries({ queryKey: ['diagnostics'] });
  void queryClient.invalidateQueries({ queryKey: ['rag'] });
  void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
}
