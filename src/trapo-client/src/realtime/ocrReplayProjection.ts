import type { QueryClient } from '@tanstack/react-query';

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
      queryKeys.documentText(text.file_hash),
      (current) => mergeTextPayload(current, text),
    );
  }
  for (const region of projection.regions.values()) {
    queryClient.setQueryData<DocumentRegionsPayload>(
      queryKeys.documentRegions(region.payload.file_hash),
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
          event.payload.file_hash,
          ensureTextPage(texts.get(event.payload.file_hash), event.payload),
        );
        break;
      case 'ocr.page.text.patch':
        texts.set(
          event.payload.file_hash,
          applyTextPatch(texts.get(event.payload.file_hash), event.payload),
        );
        break;
      case 'ocr.page.span.upsert':
        texts.set(
          event.payload.file_hash,
          applySpanUpsert(texts.get(event.payload.file_hash), event.payload),
        );
        break;
      case 'ocr.page.span.remove':
        texts.set(
          event.payload.file_hash,
          applySpanRemove(texts.get(event.payload.file_hash), event.payload),
        );
        break;
      case 'ocr.page.region.upsert':
        setProjectedRegions(
          regions,
          event.payload.file_hash,
          event.payload.page_no,
          applyRegionUpsert(
            projectedRegionPayload(regions, event.payload.file_hash),
            event.payload,
          ),
        );
        break;
      case 'ocr.page.region.remove':
        setProjectedRegions(
          regions,
          event.payload.file_hash,
          event.payload.page_no,
          applyRegionRemove(
            projectedRegionPayload(regions, event.payload.file_hash),
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

function projectedRegionPayload(regions: Map<string, ProjectedRegions>, fileHash: string) {
  return regions.get(fileHash)?.payload;
}

function setProjectedRegions(
  regions: Map<string, ProjectedRegions>,
  fileHash: string,
  pageNo: number,
  payload: DocumentRegionsPayload,
) {
  const existing = regions.get(fileHash);
  const pageNos = existing?.pageNos ?? new Set<number>();
  pageNos.add(pageNo);
  regions.set(fileHash, { pageNos, payload });
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
        left.page_no - right.page_no || left.region_id.localeCompare(right.region_id),
    ),
    file_hash: projected.payload.file_hash,
  };
}
