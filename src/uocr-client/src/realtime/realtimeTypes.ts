import type {
  DocumentRegionsPayload,
  DocumentSummary,
  DocumentTextPayload,
  IngestRunRecord,
  LogRecord,
  ModelAssetRecord,
  StatusPayload,
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
  | 'log.appended';

export interface ConnectionReadyPayload {
  path?: string;
  heartbeat?: string;
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
