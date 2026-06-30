import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { buildApiUrl, getJson, postJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  DocumentRegionsPayload,
  DocumentsPayload,
  DocumentTextPayload,
  FolderDialogResponse,
  IngestRunRecord,
  IngestRunsPayload,
  IngestStartRequest,
  LogsPayload,
  ModelDownloadRecord,
  ModelDownloadRequest,
  ModelSelectRecord,
  ModelsPayload,
  PreviewImagesPayload,
  SettingsPayload,
  StatusPayload,
} from './types';

export { queryKeys };

export function useStatus() {
  return useQuery({
    queryFn: ({ signal }) => getJson<StatusPayload>('/api/status', signal),
    queryKey: queryKeys.status,
    refetchInterval: 15000,
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
  });
}
