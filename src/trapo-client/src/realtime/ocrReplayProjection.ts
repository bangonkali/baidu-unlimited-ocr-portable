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

export function applyProjectedOcrReplay(queryClient: QueryClient, events: RealtimeEvent[]) {
  const projection = projectOcrReplayEvents(events);
  for (const text of projection.texts.values()) {
    queryClient.setQueryData<DocumentTextPayload>(
      queryKeys.documentText(text.file_hash, text.run_id ?? undefined),
      (current) => mergeTextPayload(current, text),
    );
  }
  for (const region of projection.regions.values()) {
    queryClient.setQueryData<DocumentRegionsPayload>(
      queryKeys.documentRegions(region.payload.file_hash, region.payload.run_id ?? undefined),
      (current) => mergeRegionPayload(current, region),
    );
  }
}

function projectOcrReplayEvents(events: RealtimeEvent[]) {
  const texts = new Map<string, DocumentTextPayload>();
  const regions = new Map<string, ProjectedRegions>();
  for (const event of [...events].sort((left, right) => left.sequence - right.sequence)) {
    switch (event.type) {
      case 'ocr.page.stream.started':
        texts.set(
          replayScopeKey(event.payload.run_id, event.payload.file_hash),
          ensureTextPage(
            texts.get(replayScopeKey(event.payload.run_id, event.payload.file_hash)),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.text.patch':
        texts.set(
          replayScopeKey(event.payload.run_id, event.payload.file_hash),
          applyTextPatch(
            texts.get(replayScopeKey(event.payload.run_id, event.payload.file_hash)),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.span.upsert':
        texts.set(
          replayScopeKey(event.payload.run_id, event.payload.file_hash),
          applySpanUpsert(
            texts.get(replayScopeKey(event.payload.run_id, event.payload.file_hash)),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.span.remove':
        texts.set(
          replayScopeKey(event.payload.run_id, event.payload.file_hash),
          applySpanRemove(
            texts.get(replayScopeKey(event.payload.run_id, event.payload.file_hash)),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.region.upsert':
        setProjectedRegions(
          regions,
          event.payload.run_id,
          event.payload.file_hash,
          event.payload.page_no,
          applyRegionUpsert(
            projectedRegionPayload(regions, event.payload.run_id, event.payload.file_hash),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.region.remove':
        setProjectedRegions(
          regions,
          event.payload.run_id,
          event.payload.file_hash,
          event.payload.page_no,
          applyRegionRemove(
            projectedRegionPayload(regions, event.payload.run_id, event.payload.file_hash),
            event.payload,
          ),
        );
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
  fileHash: string,
) {
  return regions.get(replayScopeKey(runId, fileHash))?.payload;
}

function setProjectedRegions(
  regions: Map<string, ProjectedRegions>,
  runId: string,
  fileHash: string,
  pageNo: number,
  payload: DocumentRegionsPayload,
) {
  const scopeKey = replayScopeKey(runId, fileHash);
  const existing = regions.get(scopeKey);
  const pageNos = existing?.pageNos ?? new Set<number>();
  pageNos.add(pageNo);
  regions.set(scopeKey, { pageNos, payload });
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
    run_id: projected.payload.run_id,
  };
}

function replayScopeKey(runId: string | null | undefined, fileHash: string) {
  return `${runId ?? ''}:${fileHash}`;
}
