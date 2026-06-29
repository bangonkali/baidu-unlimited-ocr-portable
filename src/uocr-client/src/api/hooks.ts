import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { buildApiUrl, getJson, postJson } from './http';
import type {
  DocumentRegionsPayload,
  DocumentsPayload,
  DocumentTextPayload,
  FolderDialogResponse,
  IngestRunRecord,
  IngestRunsPayload,
  IngestStartRequest,
  LogsPayload,
  ModelAssetRecord,
  ModelDownloadRecord,
  ModelDownloadRequest,
  ModelsPayload,
  PreviewImagesPayload,
  SettingsPayload,
  StatusPayload,
} from './types';

export const queryKeys = {
  documents: (q: string) => ['documents', q] as const,
  documentPreviewImages: (fileHash?: string) => ['document-preview-images', fileHash] as const,
  documentRegions: (fileHash?: string) => ['document-regions', fileHash] as const,
  documentText: (fileHash?: string) => ['document-text', fileHash] as const,
  logs: ['logs'] as const,
  models: ['models'] as const,
  runs: ['runs'] as const,
  settings: ['settings'] as const,
  status: ['status'] as const,
};

export function useStatus() {
  return useQuery({
    queryFn: ({ signal }) => getJson<StatusPayload>('/api/status', signal),
    queryKey: queryKeys.status,
    refetchInterval: 3000,
  });
}

export function useDocuments(q: string) {
  return useQuery({
    placeholderData: { documents: [] },
    queryFn: ({ signal }) =>
      getJson<DocumentsPayload>(buildApiUrl('/api/documents', { q }), signal),
    queryKey: queryKeys.documents(q),
  });
}

export function useDocumentRegions(fileHash?: string) {
  return useQuery({
    enabled: Boolean(fileHash),
    placeholderData: { boxes: [], file_hash: fileHash ?? '' },
    queryFn: ({ signal }) =>
      getJson<DocumentRegionsPayload>(
        `/api/documents/${encodeURIComponent(fileHash ?? '')}/regions`,
        signal,
      ),
    queryKey: queryKeys.documentRegions(fileHash),
  });
}

export function useDocumentText(fileHash?: string) {
  return useQuery({
    enabled: Boolean(fileHash),
    placeholderData: { file_hash: fileHash ?? '', pages: [] },
    queryFn: ({ signal }) =>
      getJson<DocumentTextPayload>(
        `/api/documents/${encodeURIComponent(fileHash ?? '')}/text`,
        signal,
      ),
    queryKey: queryKeys.documentText(fileHash),
  });
}

export function useDocumentPreviewImages(fileHash?: string) {
  return useQuery({
    enabled: Boolean(fileHash),
    placeholderData: { file_hash: fileHash ?? '', pages: [], variants: [] },
    queryFn: ({ signal }) =>
      getJson<PreviewImagesPayload>(
        `/api/documents/${encodeURIComponent(fileHash ?? '')}/preview-images`,
        signal,
      ),
    queryKey: queryKeys.documentPreviewImages(fileHash),
  });
}

export function useModels() {
  return useQuery({
    placeholderData: { models: [], profiles: [] },
    queryFn: ({ signal }) => getJson<ModelsPayload>('/api/models', signal),
    queryKey: queryKeys.models,
    refetchInterval: 1000,
  });
}

interface DownloadModelInput {
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

export function useModelDownloadEvents(modelId?: string) {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (!modelId) {
      return undefined;
    }
    const source = new EventSource(`/api/models/${encodeURIComponent(modelId)}/events`);
    const applyModelEvent = (event: MessageEvent<string>) => {
      let model: ModelAssetRecord;
      try {
        model = JSON.parse(event.data) as ModelAssetRecord;
      } catch {
        return;
      }
      queryClient.setQueryData<ModelsPayload>(queryKeys.models, (current) => {
        const existing = current ?? { models: [], profiles: [] };
        const found = existing.models.some((item) => item.model_id === model.model_id);
        return {
          ...existing,
          models: found
            ? existing.models.map((item) => (item.model_id === model.model_id ? model : item))
            : [...existing.models, model],
        };
      });
      if (!['downloading', 'metadata', 'retrying'].includes(model.status)) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
      }
    };
    source.addEventListener('model', applyModelEvent as EventListener);
    source.onmessage = applyModelEvent;
    return () => source.close();
  }, [queryClient, modelId]);
}

export function useSettings() {
  return useQuery({
    queryFn: ({ signal }) => getJson<SettingsPayload>('/api/settings', signal),
    queryKey: queryKeys.settings,
  });
}

export function useIngestRuns() {
  return useQuery({
    placeholderData: { runs: [] },
    queryFn: ({ signal }) => getJson<IngestRunsPayload>('/api/ingest/runs', signal),
    queryKey: queryKeys.runs,
    refetchInterval: 3000,
  });
}

export function useOpenFolderDialog() {
  return useMutation({
    mutationFn: () =>
      postJson<FolderDialogResponse, Record<string, never>>('/api/system/folder-dialog', {}),
  });
}

export function useStartIngest() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: IngestStartRequest) =>
      postJson<IngestRunRecord, IngestStartRequest>('/api/ingest/start', body),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: ['documents'] });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}

export function useRunCommand(command: 'stop') {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (runId: string) =>
      postJson<IngestRunRecord, Record<string, never>>(`/api/ingest/runs/${runId}/${command}`, {}),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}

export function useLogs(limit = 200) {
  return useQuery({
    placeholderData: { logs: [] },
    queryFn: ({ signal }) =>
      getJson<LogsPayload>(buildApiUrl('/api/logs/recent', { limit }), signal),
    queryKey: [...queryKeys.logs, limit],
    refetchInterval: 3000,
  });
}

export function useRunEvents(runId?: string | null) {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (!runId) {
      return undefined;
    }
    const source = new EventSource(`/api/ingest/runs/${runId}/events`);
    const invalidateRunQueries = () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: ['documents'] });
      void queryClient.invalidateQueries({ queryKey: ['document-preview-images'] });
      void queryClient.invalidateQueries({ queryKey: ['document-regions'] });
      void queryClient.invalidateQueries({ queryKey: ['document-text'] });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    };
    source.onmessage = invalidateRunQueries;
    source.addEventListener('snapshot', invalidateRunQueries);
    return () => source.close();
  }, [queryClient, runId]);
}
