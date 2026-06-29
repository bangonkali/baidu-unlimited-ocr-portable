import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { buildApiUrl, getJson, postJson, putJson } from './http';
import type {
  AnnotationSettingsPayload,
  DocumentRegionsPayload,
  DocumentsPayload,
  DocumentTextPayload,
  FolderDialogResponse,
  IngestRunRecord,
  IngestRunsPayload,
  IngestStartRequest,
  ModelsPayload,
  SettingsPayload,
  StatusPayload,
} from './types';

const queryKeys = {
  documents: (q: string) => ['documents', q] as const,
  models: ['models'] as const,
  runs: ['runs'] as const,
  settings: ['settings'] as const,
  status: ['status'] as const,
};

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
    queryKey: ['document-regions', fileHash],
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
    queryKey: ['document-text', fileHash],
  });
}

export function useModels() {
  return useQuery({
    placeholderData: { models: [], profiles: [] },
    queryFn: ({ signal }) => getJson<ModelsPayload>('/api/models', signal),
    queryKey: queryKeys.models,
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
    },
  });
}

export function useRunCommand(command: 'pause' | 'resume' | 'stop') {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (runId: string) =>
      postJson<IngestRunRecord, Record<string, never>>(`/api/ingest/runs/${runId}/${command}`, {}),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
    },
  });
}

export function useUpdateAnnotationSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: AnnotationSettingsPayload) =>
      putJson<AnnotationSettingsPayload, AnnotationSettingsPayload>(
        '/api/annotation-settings',
        body,
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['annotation-settings'] });
    },
  });
}

export function useRunEvents(runId?: string | null) {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (!runId) {
      return undefined;
    }
    const source = new EventSource(`/api/ingest/runs/${runId}/events`);
    source.onmessage = () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      void queryClient.invalidateQueries({ queryKey: ['documents'] });
    };
    return () => source.close();
  }, [queryClient, runId]);
}
