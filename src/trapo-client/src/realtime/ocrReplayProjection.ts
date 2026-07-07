import type { QueryClient } from '@tanstack/react-query';
import { annotationIdOf } from '../api/annotationIdentity';
import { queryKeys } from '../api/queryKeys';
import type { DocumentRegionsPayload, DocumentTextPayload } from '../api/types';
import {
  applyRegionRemove,
  applyRegionUpsert,
  applySpanRemove,
  applySpanUpsert,
  applyTextPatch,
  ensureTextPage,
} from './ocrStreamReducer';
import type { RealtimeEvent } from './realtimeTypes';

interface ProjectedRegions {
  pageNos: Set<number>;
  payload: DocumentRegionsPayload;
}

interface ProjectedRegionUpdate {
  fileHash: string;
  pageNo: number;
  payload: DocumentRegionsPayload;
  runEngineId: string | undefined;
  runId: string;
}

export function applyProjectedOcrReplay(queryClient: QueryClient, events: RealtimeEvent[]) {
  const projection = projectOcrReplayEvents(events);
  for (const text of projection.texts.values()) {
    for (const queryKey of textProjectionKeys(text)) {
      queryClient.setQueryData<DocumentTextPayload>(queryKey, (current) =>
        mergeTextPayload(current, text),
      );
    }
  }
  for (const region of projection.regions.values()) {
    for (const queryKey of regionProjectionKeys(region.payload)) {
      queryClient.setQueryData<DocumentRegionsPayload>(queryKey, (current) =>
        mergeRegionPayload(current, region),
      );
    }
  }
}

function projectOcrReplayEvents(events: RealtimeEvent[]) {
  const texts = new Map<string, DocumentTextPayload>();
  const regions = new Map<string, ProjectedRegions>();
  for (const event of [...events].sort((left, right) => left.sequence - right.sequence)) {
    switch (event.type) {
      case 'ocr.page.stream.started':
        texts.set(
          replayScopeKey(
            event.payload.run_id,
            event.payload.file_hash,
            event.payload.run_engine_id,
          ),
          ensureTextPage(
            texts.get(
              replayScopeKey(
                event.payload.run_id,
                event.payload.file_hash,
                event.payload.run_engine_id,
              ),
            ),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.text.patch':
        texts.set(
          replayScopeKey(
            event.payload.run_id,
            event.payload.file_hash,
            event.payload.run_engine_id,
          ),
          applyTextPatch(
            texts.get(
              replayScopeKey(
                event.payload.run_id,
                event.payload.file_hash,
                event.payload.run_engine_id,
              ),
            ),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.span.upsert':
        texts.set(
          replayScopeKey(
            event.payload.run_id,
            event.payload.file_hash,
            event.payload.run_engine_id,
          ),
          applySpanUpsert(
            texts.get(
              replayScopeKey(
                event.payload.run_id,
                event.payload.file_hash,
                event.payload.run_engine_id,
              ),
            ),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.span.remove':
        texts.set(
          replayScopeKey(
            event.payload.run_id,
            event.payload.file_hash,
            event.payload.run_engine_id,
          ),
          applySpanRemove(
            texts.get(
              replayScopeKey(
                event.payload.run_id,
                event.payload.file_hash,
                event.payload.run_engine_id,
              ),
            ),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.region.upsert':
        setProjectedRegions(regions, {
          fileHash: event.payload.file_hash,
          pageNo: event.payload.page_no,
          payload: applyRegionUpsert(
            projectedRegionPayload(
              regions,
              event.payload.run_id,
              event.payload.run_engine_id,
              event.payload.file_hash,
            ),
            event.payload,
          ),
          runEngineId: event.payload.run_engine_id,
          runId: event.payload.run_id,
        });
        break;
      case 'ocr.page.region.remove':
        setProjectedRegions(regions, {
          fileHash: event.payload.file_hash,
          pageNo: event.payload.page_no,
          payload: applyRegionRemove(
            projectedRegionPayload(
              regions,
              event.payload.run_id,
              event.payload.run_engine_id,
              event.payload.file_hash,
            ),
            event.payload,
          ),
          runEngineId: event.payload.run_engine_id,
          runId: event.payload.run_id,
        });
        break;
      default:
        break;
    }
  }
  return { regions, texts };
}

function projectedRegionPayload(
  regions: Map<string, ProjectedRegions>,
  runId: string,
  runEngineId: string | undefined,
  fileHash: string,
) {
  return regions.get(replayScopeKey(runId, fileHash, runEngineId))?.payload;
}

function setProjectedRegions(
  regions: Map<string, ProjectedRegions>,
  update: ProjectedRegionUpdate,
) {
  const scopeKey = replayScopeKey(update.runId, update.fileHash, update.runEngineId);
  const existing = regions.get(scopeKey);
  const pageNos = existing?.pageNos ?? new Set<number>();
  pageNos.add(update.pageNo);
  regions.set(scopeKey, { pageNos, payload: update.payload });
}

function mergeTextPayload(
  current: DocumentTextPayload | undefined,
  projected: DocumentTextPayload,
): DocumentTextPayload {
  const pagesByNumber = new Map((current?.pages ?? []).map((page) => [page.page_no, page]));
  for (const page of projected.pages) {
    pagesByNumber.set(page.page_no, page);
  }
  return {
    file_hash: projected.file_hash,
    ...(projected.run_engine_id ? { run_engine_id: projected.run_engine_id } : {}),
    run_id: projected.run_id,
    pages: [...pagesByNumber.values()].sort((left, right) => left.page_no - right.page_no),
  };
}

function mergeRegionPayload(
  current: DocumentRegionsPayload | undefined,
  projected: ProjectedRegions,
): DocumentRegionsPayload {
  const projectedPageNos = projected.pageNos;
  const retained = (current?.boxes ?? []).filter((box) => !projectedPageNos.has(box.page_no));
  return {
    boxes: [...retained, ...projected.payload.boxes].sort(
      (left, right) =>
        left.page_no - right.page_no || annotationIdOf(left).localeCompare(annotationIdOf(right)),
    ),
    file_hash: projected.payload.file_hash,
    ...(projected.payload.run_engine_id ? { run_engine_id: projected.payload.run_engine_id } : {}),
    run_id: projected.payload.run_id,
  };
}

function replayScopeKey(
  runId: string | null | undefined,
  fileHash: string,
  runEngineId?: string | null,
) {
  return `${runId ?? ''}:${runEngineId ?? ''}:${fileHash}`;
}

function textProjectionKeys(payload: DocumentTextPayload) {
  return payload.run_engine_id
    ? [
        queryKeys.documentText(
          payload.file_hash,
          payload.run_id ?? undefined,
          payload.run_engine_id,
        ),
      ]
    : [queryKeys.documentText(payload.file_hash, payload.run_id ?? undefined)];
}

function regionProjectionKeys(payload: DocumentRegionsPayload) {
  return payload.run_engine_id
    ? [
        queryKeys.documentRegions(
          payload.file_hash,
          payload.run_id ?? undefined,
          payload.run_engine_id,
        ),
      ]
    : [queryKeys.documentRegions(payload.file_hash, payload.run_id ?? undefined)];
}
