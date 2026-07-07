import type { QueryClient } from '@tanstack/react-query';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { markShutdownRequested } from '../stores/serviceShutdownStore';
import { buildApiUrl, getJson, postJson, postJsonWithHeaders, putJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  DocumentRegionsPayload,
  DocumentsPayload,
  DocumentTextPayload,
  FolderDialogResponse,
  IngestEnginesPayload,
  IngestPreviewResultsPayload,
  IngestRunRecord,
  IngestRunsPayload,
  IngestStartRequest,
  IngestStartResponse,
  LogsPayload,
  PreviewImagesPayload,
  SettingsPayload,
  SettingsUpdateRequest,
  ShutdownPayload,
  ShutdownRequest,
  StatusPayload,
} from './types';

export {
  useDiagnosticAnalytics,
  useDiagnosticModels,
  useDiagnosticProgress,
  useDiagnosticRuns,
  useDiagnosticTrace,
  useDiagnosticWaterfall,
  useDiagnosticWorkUnitDetail,
  useOcrReplay,
} from './diagnosticsHooks';
export {
  useCancelModelDownload,
  useDownloadModel,
  useModels,
  useSelectModel,
} from './modelHooks';
export {
  useGenerateEmbedding,
  useHybridSearch,
  useStartTextIndex,
  useUsedEmbeddingModels,
} from './ragHooks';
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

export function useDocumentRegions(fileHash?: string, runId?: string, runEngineId?: string) {
  return useQuery({
    enabled: Boolean(fileHash),
    placeholderData: {
      boxes: [],
      file_hash: fileHash ?? '',
      run_engine_id: runEngineId,
      run_id: runId,
    },
    queryFn: ({ signal }) =>
      getJson<DocumentRegionsPayload>(
        buildApiUrl(`/api/documents/${encodeURIComponent(fileHash ?? '')}/regions`, {
          run_engine_id: runEngineId,
          run_id: runId,
        }),
        signal,
      ),
    queryKey: queryKeys.documentRegions(fileHash, runId, runEngineId),
  });
}

export function useDocumentText(fileHash?: string, runId?: string, runEngineId?: string) {
  return useQuery({
    enabled: Boolean(fileHash),
    placeholderData: {
      file_hash: fileHash ?? '',
      pages: [],
      run_engine_id: runEngineId,
      run_id: runId,
    },
    queryFn: ({ signal }) =>
      getJson<DocumentTextPayload>(
        buildApiUrl(`/api/documents/${encodeURIComponent(fileHash ?? '')}/text`, {
          run_engine_id: runEngineId,
          run_id: runId,
        }),
        signal,
      ),
    queryKey: queryKeys.documentText(fileHash, runId, runEngineId),
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

export function useIngestEngines() {
  return useQuery({
    placeholderData: { engines: [] },
    queryFn: ({ signal }) => getJson<IngestEnginesPayload>('/api/ingest/engines', signal),
    queryKey: queryKeys.ingestEngines,
  });
}

export function useIngestPreviewResults(runId?: string, fileHash?: string) {
  return useQuery({
    enabled: Boolean(runId && fileHash),
    placeholderData: { file_hash: fileHash ?? '', results: [], run_id: runId ?? '' },
    queryFn: ({ signal }) =>
      getJson<IngestPreviewResultsPayload>(
        buildApiUrl(`/api/ingest/runs/${encodeURIComponent(runId ?? '')}/preview-results`, {
          file_hash: fileHash,
        }),
        signal,
      ),
    queryKey: queryKeys.previewResults(runId, fileHash),
  });
}

export function useOpenFolderDialog() {
  return useMutation({
    mutationFn: () =>
      postJson<FolderDialogResponse, Record<string, never>>('/api/system/folder-dialog', {}),
  });
}

export function useShutdownServer() {
  return useMutation({
    mutationFn: () =>
      postJsonWithHeaders<ShutdownPayload, ShutdownRequest>(
        '/api/system/shutdown',
        { confirm: 'shutdown' },
        { 'x-trapo-intent': 'shutdown' },
      ),
    onSuccess: markShutdownRequested,
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

export function useResumeRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (runId: string) =>
      postJson<IngestStartResponse, Record<string, never>>(
        `/api/ingest/runs/${encodeURIComponent(runId)}/resume`,
        {},
      ),
    onSuccess: (response) => {
      seedIngestStartResponse(queryClient, response);
      invalidateIngestState(queryClient);
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
  for (const document of response.documents) {
    queryClient.setQueryData<DocumentRegionsPayload>(
      queryKeys.documentRegions(document.file_hash, response.run.run_id),
      { boxes: [], file_hash: document.file_hash, run_id: response.run.run_id },
    );
    queryClient.setQueryData<DocumentTextPayload>(
      queryKeys.documentText(document.file_hash, response.run.run_id),
      { file_hash: document.file_hash, pages: [], run_id: response.run.run_id },
    );
  }
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

function invalidateIngestState(queryClient: QueryClient) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
  void queryClient.invalidateQueries({ queryKey: queryKeys.status });
  void queryClient.invalidateQueries({ queryKey: ['documents'] });
  void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
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
      invalidateIngestState(queryClient);
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
