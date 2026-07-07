import type { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type {
  DocumentsPayload,
  IngestRunsPayload,
  LogsPayload,
  ModelsPayload,
  PreviewImagesPayload,
  StatusPayload,
} from '../api/types';
import { followLatestPage } from '../stores/workbenchStore';
import {
  applyRegionRemove,
  applyRegionUpsert,
  applySpanRemove,
  applySpanUpsert,
  applyTextPatch,
  ensureTextPage,
} from './ocrStreamReducer';
import {
  invalidatePreviewResults,
  setRegionsPayload,
  setTextPayload,
  updateRegionsPayload,
  updateTextPayload,
} from './realtimeQueryCache';
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
    const incoming = event.payload;
    const models = upsertById(existing.models, incoming, (model) => model.model_id);
    return {
      ...existing,
      models: incoming.selected
        ? models.map((model) => ({ ...model, selected: model.model_id === incoming.model_id }))
        : models,
      selected_model_id: incoming.selected ? incoming.model_id : existing.selected_model_id,
    };
  });
  if (event.payload.selected) {
    queryClient.setQueryData<StatusPayload>(queryKeys.status, (current) =>
      current ? { ...current, selected_model_id: event.payload.model_id } : current,
    );
  }
}

function applyRunEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'run.changed' }>,
) {
  queryClient.setQueryData<IngestRunsPayload>(queryKeys.runs, (current) => ({
    runs: upsertById(current?.runs ?? [], event.payload, (run) => run.run_id),
  }));
  queryClient.setQueryData<StatusPayload>(queryKeys.status, (current) => {
    if (!current) {
      return current;
    }
    const active = isActiveRunStatus(event.payload.status);
    return {
      ...current,
      active_run_id: active
        ? event.payload.run_id
        : current.active_run_id === event.payload.run_id
          ? null
          : current.active_run_id,
      state: event.payload.status,
    };
  });
  void queryClient.invalidateQueries({ queryKey: ['ingest-preview-results'] });
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
  for (const [queryKey, payload] of queryClient.getQueriesData<DocumentsPayload>({
    queryKey: ['documents'],
  })) {
    if (!payload || sameQueryKey(queryKey, queryKeys.documents(''))) {
      continue;
    }
    if (!payload.documents.some((document) => document.file_hash === event.payload.file_hash)) {
      continue;
    }
    queryClient.setQueryData<DocumentsPayload>(queryKey, {
      documents: upsertById(payload.documents, event.payload, (document) => document.file_hash),
    });
  }
}

function applyPageEvent(
  queryClient: QueryClient,
  event: Extract<RealtimeEvent, { type: 'document.page.changed' }>,
) {
  const fileHash = event.payload.file_hash;
  if (event.payload.preview_available) {
    queryClient.setQueryData<PreviewImagesPayload>(
      queryKeys.documentPreviewImages(fileHash),
      (current) => ({
        file_hash: fileHash,
        variants: uniqueStrings([...(current?.variants ?? []), 'source']),
        pages: uniqueSortedNumbers([...(current?.pages ?? []), event.payload.page_no]),
      }),
    );
  }
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
      setRegionsPayload(queryClient, event.payload);
      invalidatePreviewResults(queryClient, event.payload.run_id, event.payload.file_hash);
      return;
    case 'document.text.changed':
      setTextPayload(queryClient, event.payload);
      invalidatePreviewResults(queryClient, event.payload.run_id, event.payload.file_hash);
      return;
    case 'ocr.page.stream.started':
      updateTextPayload(queryClient, event.payload, (current) =>
        ensureTextPage(current, event.payload),
      );
      followLatestPage(
        event.payload.file_hash,
        event.payload.page_no,
        event.payload.run_id,
        event.payload.run_engine_id,
      );
      return;
    case 'ocr.page.raw.delta':
      return;
    case 'ocr.page.text.patch':
      updateTextPayload(queryClient, event.payload, (current) =>
        applyTextPatch(current, event.payload),
      );
      followLatestPage(
        event.payload.file_hash,
        event.payload.page_no,
        event.payload.run_id,
        event.payload.run_engine_id,
      );
      return;
    case 'ocr.page.region.upsert':
      updateRegionsPayload(queryClient, event.payload, (current) =>
        applyRegionUpsert(current, event.payload),
      );
      return;
    case 'ocr.page.region.remove':
      updateRegionsPayload(queryClient, event.payload, (current) =>
        applyRegionRemove(current, event.payload),
      );
      return;
    case 'ocr.page.span.upsert':
      updateTextPayload(queryClient, event.payload, (current) =>
        applySpanUpsert(current, event.payload),
      );
      return;
    case 'ocr.page.span.remove':
      updateTextPayload(queryClient, event.payload, (current) =>
        applySpanRemove(current, event.payload),
      );
      return;
    case 'ocr.page.metrics.changed':
    case 'ocr.page.stream.completed':
    case 'ocr.page.stream.failed':
      return;
    case 'log.appended':
      applyLogEvent(queryClient, event);
      return;
  }
}

function isActiveRunStatus(status: string) {
  return status === 'queued' || status === 'running';
}

function sameQueryKey(left: readonly unknown[], right: readonly unknown[]) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function uniqueSortedNumbers(values: number[]) {
  return [...new Set(values)].sort((left, right) => left - right);
}

function uniqueStrings(values: string[]) {
  return [...new Set(values)];
}
