import type { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { DocumentsPayload, IngestRunsPayload, LogsPayload, ModelsPayload } from '../api/types';
import type { RealtimeEvent } from './realtimeTypes';

function upsertById<T>(items: T[], item: T, getId: (value: T) => string) {
  const id = getId(item);
  const found = items.some((value) => getId(value) === id);
  return found ? items.map((value) => (getId(value) === id ? item : value)) : [item, ...items];
}

function applyModelEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'model.changed' }>,
) {
  queryClient.setQueryData<ModelsPayload>(queryKeys.models, (current) => {
    const existing = current ?? { models: [], profiles: [] };
    return {
      ...existing,
      models: upsertById(existing.models, event.payload, (model) => model.model_id),
    };
  });
}

function applyRunEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'run.changed' }>,
) {
  queryClient.setQueryData<IngestRunsPayload>(queryKeys.runs, (current) => ({
    runs: upsertById(current?.runs ?? [], event.payload, (run) => run.run_id),
  }));
  void queryClient.invalidateQueries({ queryKey: queryKeys.status });
}

function applyDocumentEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'document.changed' }>,
) {
  queryClient.setQueryData<DocumentsPayload>(queryKeys.documents(''), (current) => ({
    documents: upsertById(
      current?.documents ?? [],
      event.payload,
      (document) => document.file_hash,
    ),
  }));
  void queryClient.invalidateQueries({ queryKey: ['documents'], refetchType: 'active' });
}

function applyPageEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'document.page.changed' }>,
) {
  const fileHash = event.payload.file_hash;
  void queryClient.invalidateQueries({ queryKey: queryKeys.documentPreviewImages(fileHash) });
  void queryClient.invalidateQueries({ queryKey: ['documents'], refetchType: 'active' });
}

function applyLogEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'log.appended' }>,
) {
  queryClient.setQueriesData<LogsPayload>({ queryKey: queryKeys.logs }, (current) => {
    const logs = [...(current?.logs ?? []), event.payload].slice(-1000);
    return {
      log_path: current?.log_path,
      logs,
    };
  });
}

export function applyRealtimeEventToQueryClient(queryClient: QueryClient, event: RealtimeEvent) {
  switch (event.type) {
    case 'connection.ready':
      return;
    case 'status.changed':
      queryClient.setQueryData(queryKeys.status, event.payload);
      return;
    case 'model.changed':
      applyModelEvent(queryClient, event);
      return;
    case 'run.changed':
      applyRunEvent(queryClient, event);
      return;
    case 'document.changed':
      applyDocumentEvent(queryClient, event);
      return;
    case 'document.page.changed':
      applyPageEvent(queryClient, event);
      return;
    case 'document.regions.changed':
      queryClient.setQueryData(queryKeys.documentRegions(event.payload.file_hash), event.payload);
      return;
    case 'document.text.changed':
      queryClient.setQueryData(queryKeys.documentText(event.payload.file_hash), event.payload);
      return;
    case 'log.appended':
      applyLogEvent(queryClient, event);
      return;
  }
}
