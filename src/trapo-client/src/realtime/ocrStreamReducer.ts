import { annotationIdOf } from '../api/annotationIdentity';
import type {
  DocumentRegionsPayload,
  DocumentTextPayload,
  PageTextRecord,
  TextRegionSpan,
} from '../api/types';
import type {
  OcrPageRegionRemovePayload,
  OcrPageRegionUpsertPayload,
  OcrPageSpanRemovePayload,
  OcrPageSpanUpsertPayload,
  OcrPageStreamContext,
  OcrPageTextPatchPayload,
} from './realtimeTypes';

export function ensureTextPage(
  current: DocumentTextPayload | undefined,
  context: OcrPageStreamContext,
): DocumentTextPayload {
  const payload = current ?? { file_hash: context.file_hash, pages: [], run_id: context.run_id };
  return {
    ...payload,
    file_hash: context.file_hash,
    run_id: context.run_id,
    pages: upsertPage(payload.pages, { page_no: context.page_no, spans: [], text: '' }),
  };
}

export function applyTextPatch(
  current: DocumentTextPayload | undefined,
  patch: OcrPageTextPatchPayload,
): DocumentTextPayload {
  const payload = ensureTextPage(current, patch);
  return {
    ...payload,
    pages: payload.pages.map((page) =>
      page.page_no === patch.page_no
        ? {
            ...page,
            text:
              patch.op === 'append'
                ? replaceRange(page.text, patch.start, patch.end, patch.text)
                : patch.text,
          }
        : page,
    ),
  };
}

export function applySpanUpsert(
  current: DocumentTextPayload | undefined,
  payload: OcrPageSpanUpsertPayload,
): DocumentTextPayload {
  const textPayload = ensureTextPage(current, payload);
  return {
    ...textPayload,
    pages: textPayload.pages.map((page) =>
      page.page_no === payload.page_no
        ? {
            ...page,
            spans: upsertSpan(page.spans, payload.span),
          }
        : page,
    ),
  };
}

export function applySpanRemove(
  current: DocumentTextPayload | undefined,
  payload: OcrPageSpanRemovePayload,
): DocumentTextPayload {
  const textPayload = ensureTextPage(current, payload);
  return {
    ...textPayload,
    pages: textPayload.pages.map((page) =>
      page.page_no === payload.page_no
        ? {
            ...page,
            spans: page.spans.filter((span) => annotationIdOf(span) !== payload.region_id),
          }
        : page,
    ),
  };
}

export function applyRegionUpsert(
  current: DocumentRegionsPayload | undefined,
  payload: OcrPageRegionUpsertPayload,
): DocumentRegionsPayload {
  const existing = current ?? {
    boxes: [],
    file_hash: payload.file_hash,
    run_id: payload.run_id,
  };
  const incomingId = annotationIdOf(payload.region);
  const boxes = existing.boxes.some((box) => annotationIdOf(box) === incomingId)
    ? existing.boxes.map((box) => (annotationIdOf(box) === incomingId ? payload.region : box))
    : [...existing.boxes, payload.region];
  return { boxes, file_hash: payload.file_hash, run_id: payload.run_id };
}

export function applyRegionRemove(
  current: DocumentRegionsPayload | undefined,
  payload: OcrPageRegionRemovePayload,
): DocumentRegionsPayload {
  const existing = current ?? {
    boxes: [],
    file_hash: payload.file_hash,
    run_id: payload.run_id,
  };
  return {
    boxes: existing.boxes.filter((box) => annotationIdOf(box) !== payload.region_id),
    file_hash: payload.file_hash,
    run_id: payload.run_id,
  };
}

function upsertPage(pages: PageTextRecord[], page: PageTextRecord) {
  return pages.some((item) => item.page_no === page.page_no)
    ? pages
    : [...pages, page].sort((left, right) => left.page_no - right.page_no);
}

function replaceRange(value: string, start: number, end: number, text: string) {
  return `${value.slice(0, Math.max(0, start))}${text}${value.slice(Math.max(start, end))}`;
}

function upsertSpan(spans: TextRegionSpan[], span: TextRegionSpan) {
  const spanId = annotationIdOf(span);
  const next = spans.some((item) => annotationIdOf(item) === spanId)
    ? spans.map((item) => (annotationIdOf(item) === spanId ? span : item))
    : [...spans, span];
  return next.sort((left, right) => left.start - right.start || left.end - right.end);
}
