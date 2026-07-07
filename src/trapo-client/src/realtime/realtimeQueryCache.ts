import type { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { DocumentRegionsPayload, DocumentTextPayload } from '../api/types';

export function invalidatePreviewResults(
  queryClient: QueryClient,
  runId: string | null | undefined,
  fileHash: string,
) {
  if (!runId) {
    return;
  }
  void queryClient.invalidateQueries({ queryKey: queryKeys.previewResults(runId, fileHash) });
}

export function setTextPayload(queryClient: QueryClient, payload: DocumentTextPayload) {
  for (const queryKey of textPayloadKeys(
    payload.file_hash,
    payload.run_id,
    payload.run_engine_id,
  )) {
    queryClient.setQueryData(queryKey, payload);
  }
}

export function setRegionsPayload(queryClient: QueryClient, payload: DocumentRegionsPayload) {
  for (const queryKey of regionPayloadKeys(
    payload.file_hash,
    payload.run_id,
    payload.run_engine_id,
  )) {
    queryClient.setQueryData(queryKey, payload);
  }
}

export function updateTextPayload(
  queryClient: QueryClient,
  context: { file_hash: string; run_engine_id?: string; run_id: string },
  update: (current: DocumentTextPayload | undefined) => DocumentTextPayload,
) {
  for (const queryKey of textPayloadKeys(
    context.file_hash,
    context.run_id,
    context.run_engine_id,
  )) {
    queryClient.setQueryData<DocumentTextPayload>(queryKey, update);
  }
}

export function updateRegionsPayload(
  queryClient: QueryClient,
  context: { file_hash: string; run_engine_id?: string; run_id: string },
  update: (current: DocumentRegionsPayload | undefined) => DocumentRegionsPayload,
) {
  for (const queryKey of regionPayloadKeys(
    context.file_hash,
    context.run_id,
    context.run_engine_id,
  )) {
    queryClient.setQueryData<DocumentRegionsPayload>(queryKey, update);
  }
}

function textPayloadKeys(fileHash: string, runId?: string | null, runEngineId?: string | null) {
  return runEngineId
    ? [
        queryKeys.documentText(fileHash, runId ?? undefined),
        queryKeys.documentText(fileHash, runId ?? undefined, runEngineId),
      ]
    : [queryKeys.documentText(fileHash, runId ?? undefined)];
}

function regionPayloadKeys(fileHash: string, runId?: string | null, runEngineId?: string | null) {
  return runEngineId
    ? [
        queryKeys.documentRegions(fileHash, runId ?? undefined),
        queryKeys.documentRegions(fileHash, runId ?? undefined, runEngineId),
      ]
    : [queryKeys.documentRegions(fileHash, runId ?? undefined)];
}
