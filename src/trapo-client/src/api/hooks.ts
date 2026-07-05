import type { QueryClient } from '@tanstack/react-query';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { buildApiUrl, getJson, postJson, putJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  DocumentRegionsPayload,
  DocumentsPayload,
  DocumentTextPayload,
  FolderDialogResponse,
  IngestRunRecord,
  IngestRunsPayload,
  IngestStartRequest,
  IngestStartResponse,
  LogsPayload,
  ModelDownloadRecord,
  ModelDownloadRequest,
  ModelSelectRecord,
  ModelsPayload,
  PreviewImagesPayload,
  SettingsPayload,
  SettingsUpdateRequest,
  StatusPayload,
} from './types';

export {
  useDiagnosticAnalytics,
  useDiagnosticModels,
  useDiagnosticProgress,
  useDiagnosticRuns,
  useDiagnosticTrace,
  useOcrReplay,
} from './diagnosticsHooks';
export { queryKeys };

export function useStatus() {
  return useQuery({
    queryFn: ({ signal }) => getJson<StatusPayload>('/api/status', signal),
    queryKey: queryKeys.status,
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

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: SettingsUpdateRequest) =>
      putJson<SettingsPayload, SettingsUpdateRequest>('/api/settings', body),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.settings });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
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
      postJson<IngestStartResponse, IngestStartRequest>('/api/ingest/start', body),
    onSuccess: (response) => {
      seedIngestStartResponse(queryClient, response);
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: ['documents'] });
      void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
    },
  });
}

function seedIngestStartResponse(queryClient: QueryClient, response: IngestStartResponse) {
  queryClient.setQueryData<IngestRunsPayload>(queryKeys.runs, (current) => ({
    runs: upsertById(current?.runs ?? [], response.run, (run) => run.run_id),
  }));
  queryClient.setQueryData<DocumentsPayload>(queryKeys.documents(''), (current) => ({
    documents: upsertManyById(
      current?.documents ?? [],
      response.documents,
      (document) => document.file_hash,
    ),
  }));
  queryClient.setQueryData<StatusPayload>(queryKeys.status, (current) =>
    current
      ? {
          ...current,
          active_run_id: response.run.run_id,
          state: response.run.status,
        }
      : current,
  );
}

function upsertManyById<T>(current: T[], incoming: T[], getId: (value: T) => string) {
  return incoming.reduce((items, item) => upsertById(items, item, getId), current);
}

function upsertById<T>(items: T[], item: T, getId: (value: T) => string) {
  const id = getId(item);
  const found = items.some((value) => getId(value) === id);
  return found ? items.map((value) => (getId(value) === id ? item : value)) : [item, ...items];
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
