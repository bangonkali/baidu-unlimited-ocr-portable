import type {
  DocumentRegionsPayload,
  DocumentSummary,
  DocumentTextPayload,
  IngestRunRecord,
  LogRecord,
  ModelAssetRecord,
  OverlayBox,
  RealtimeEventRecord,
  StatusPayload,
  TextRegionSpan,
} from '../api/types';

export type RealtimeEventType =
  | 'connection.ready'
  | 'status.changed'
  | 'model.changed'
  | 'run.changed'
  | 'document.changed'
  | 'document.page.changed'
  | 'document.regions.changed'
  | 'document.text.changed'
  | 'ocr.page.stream.started'
  | 'ocr.page.raw.delta'
  | 'ocr.page.text.patch'
  | 'ocr.page.region.upsert'
  | 'ocr.page.region.remove'
  | 'ocr.page.span.upsert'
  | 'ocr.page.span.remove'
  | 'ocr.page.metrics.changed'
  | 'ocr.page.stream.completed'
  | 'ocr.page.stream.failed'
  | 'log.appended';

export interface ConnectionReadyPayload {
  path?: string;
  heartbeat?: string;
  last_sequence?: number;
  supported_types?: RealtimeEventType[];
}

export interface DocumentPageEventPayload {
  file_hash: string;
  page_no: number;
  status: string;
  error?: string | null;
  width_px?: number;
  height_px?: number;
  dpi?: number;
  preview_available?: boolean;
  text_available?: boolean;
  region_count?: number;
}

export interface OcrPageStreamContext {
  run_id: string;
  file_hash: string;
  page_no: number;
  engine_id?: string;
  profile_id?: string;
  model_id?: string;
  runtime_id?: string;
  runtime_platform?: string;
  accelerator?: string;
}

export interface OcrPageStreamStartedPayload extends OcrPageStreamContext {
  started_at?: string;
}

export interface OcrPageRawDeltaPayload extends OcrPageStreamContext {
  token_index: number;
  delta: string;
  raw_start: number;
  raw_end: number;
  elapsed_ms: number;
  avg_tps: number;
}

export interface OcrPageTextPatchPayload extends OcrPageStreamContext {
  op: 'append' | 'replace';
  start: number;
  end: number;
  text: string;
}

export interface OcrPageRegionUpsertPayload extends OcrPageStreamContext {
  region: OverlayBox;
}

export interface OcrPageRegionRemovePayload extends OcrPageStreamContext {
  region_id: string;
}

export interface OcrPageSpanUpsertPayload extends OcrPageStreamContext {
  span: TextRegionSpan;
}

export interface OcrPageSpanRemovePayload extends OcrPageStreamContext {
  region_id: string;
}

export interface OcrPageMetricsPayload extends OcrPageStreamContext {
  status: string;
  error?: string | null;
  token_count?: number;
  avg_tps?: number;
  elapsed_ms?: number;
}

export interface RealtimeEnvelope<TType extends RealtimeEventType, TPayload> {
  version: 1;
  sequence: number;
  type: TType;
  occurred_at: string;
  payload: TPayload;
}

export type RealtimeEvent =
  | RealtimeEnvelope<'connection.ready', ConnectionReadyPayload>
  | RealtimeEnvelope<'status.changed', StatusPayload>
  | RealtimeEnvelope<'model.changed', ModelAssetRecord>
  | RealtimeEnvelope<'run.changed', IngestRunRecord>
  | RealtimeEnvelope<'document.changed', DocumentSummary>
  | RealtimeEnvelope<'document.page.changed', DocumentPageEventPayload>
  | RealtimeEnvelope<'document.regions.changed', DocumentRegionsPayload>
  | RealtimeEnvelope<'document.text.changed', DocumentTextPayload>
  | RealtimeEnvelope<'ocr.page.stream.started', OcrPageStreamStartedPayload>
  | RealtimeEnvelope<'ocr.page.raw.delta', OcrPageRawDeltaPayload>
  | RealtimeEnvelope<'ocr.page.text.patch', OcrPageTextPatchPayload>
  | RealtimeEnvelope<'ocr.page.region.upsert', OcrPageRegionUpsertPayload>
  | RealtimeEnvelope<'ocr.page.region.remove', OcrPageRegionRemovePayload>
  | RealtimeEnvelope<'ocr.page.span.upsert', OcrPageSpanUpsertPayload>
  | RealtimeEnvelope<'ocr.page.span.remove', OcrPageSpanRemovePayload>
  | RealtimeEnvelope<'ocr.page.metrics.changed', OcrPageMetricsPayload>
  | RealtimeEnvelope<'ocr.page.stream.completed', OcrPageMetricsPayload>
  | RealtimeEnvelope<'ocr.page.stream.failed', OcrPageMetricsPayload>
  | RealtimeEnvelope<'log.appended', LogRecord>;

const realtimeEventTypes = new Set<RealtimeEventType>([
  'connection.ready',
  'status.changed',
  'model.changed',
  'run.changed',
  'document.changed',
  'document.page.changed',
  'document.regions.changed',
  'document.text.changed',
  'ocr.page.stream.started',
  'ocr.page.raw.delta',
  'ocr.page.text.patch',
  'ocr.page.region.upsert',
  'ocr.page.region.remove',
  'ocr.page.span.upsert',
  'ocr.page.span.remove',
  'ocr.page.metrics.changed',
  'ocr.page.stream.completed',
  'ocr.page.stream.failed',
  'log.appended',
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

export function isRealtimeEvent(value: unknown): value is RealtimeEvent {
  if (!isRecord(value)) {
    return false;
  }
  return (
    value.version === 1 &&
    typeof value.sequence === 'number' &&
    typeof value.type === 'string' &&
    realtimeEventTypes.has(value.type as RealtimeEventType) &&
    typeof value.occurred_at === 'string' &&
    isRecord(value.payload)
  );
}

export function parseRealtimeEvent(data: string): RealtimeEvent | null {
  try {
    const parsed = JSON.parse(data) as unknown;
    return isRealtimeEvent(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function realtimeEventFromRecord(record: RealtimeEventRecord): RealtimeEvent | null {
  const candidate = {
    occurred_at: record.occurred_at,
    payload: record.payload,
    sequence: record.sequence,
    type: record.type,
    version: 1,
  };
  return isRealtimeEvent(candidate) ? candidate : null;
}
