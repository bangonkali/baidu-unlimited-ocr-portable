import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { getJson, postJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  ModelDownloadRecord,
  ModelDownloadRequest,
  ModelSelectRecord,
  ModelsPayload,
} from './types';

export function useModels() {
  return useQuery({
    placeholderData: { models: [], profiles: [] },
    queryFn: ({ signal }) => getJson<ModelsPayload>('/api/models', signal),
    queryKey: queryKeys.models,
  });
}

export interface DownloadModelInput {
  modelId: string;
  force?: boolean;
}

export function useDownloadModel() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: DownloadModelInput) =>
      postJson<ModelDownloadRecord, ModelDownloadRequest>(
        `/api/models/${encodeURIComponent(input.modelId)}/download`,
        { force: input.force ?? false },
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.models });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}

export function useCancelModelDownload() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (modelId: string) =>
      postJson<ModelDownloadRecord, Record<string, never>>(
        `/api/models/${encodeURIComponent(modelId)}/cancel`,
        {},
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.models });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}

export function useSelectModel() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (modelId: string) =>
      postJson<ModelSelectRecord, Record<string, never>>(
        `/api/models/${encodeURIComponent(modelId)}/select`,
        {},
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.models });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}
