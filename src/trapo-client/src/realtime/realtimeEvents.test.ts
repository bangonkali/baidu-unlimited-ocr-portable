import { describe, expect, test } from 'bun:test';
import { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type {
  DocumentRegionsPayload,
  DocumentTextPayload,
  LogsPayload,
  ModelsPayload,
} from '../api/types';
import { applyRealtimeEventToQueryClient } from './realtimeQueryBridge';
import type { RealtimeEvent } from './realtimeTypes';
import { parseRealtimeEvent } from './realtimeTypes';

function envelope(event: RealtimeEvent): string {
  return JSON.stringify(event);
}

describe('realtime events', () => {
  test('parses typed websocket envelopes', () => {
    const parsed = parseRealtimeEvent(
      envelope({
        occurred_at: '2026-06-30T00:00:00Z',
        payload: {
          model_id: 'unlimited-ocr-q4-k-m',
          display_name: 'Unlimited-OCR',
          status: 'downloading',
        },
        sequence: 7,
        type: 'model.changed',
        version: 1,
      }),
    );

    expect(parsed?.type).toBe('model.changed');
    expect(parsed?.sequence).toBe(7);
  });

  test('updates model and log query data from events', () => {
    const client = new QueryClient();
    client.setQueryData<ModelsPayload>(queryKeys.models, { models: [], profiles: [] });
    client.setQueryData<LogsPayload>([...queryKeys.logs, 20], { logs: [] });

    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:01Z',
      payload: {
        display_name: 'Unlimited-OCR',
        model_id: 'unlimited-ocr-q4-k-m',
        overall_percent: 42,
        status: 'downloading',
      },
      sequence: 8,
      type: 'model.changed',
      version: 1,
    });
    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:02Z',
      payload: {
        component: 'ocr',
        level: 'INFO',
        message: 'page completed',
        timestamp: '2026-06-30T00:00:02Z',
      },
      sequence: 9,
      type: 'log.appended',
      version: 1,
    });

    expect(client.getQueryData<ModelsPayload>(queryKeys.models)?.models[0]?.overall_percent).toBe(
      42,
    );
    expect(client.getQueryData<LogsPayload>([...queryKeys.logs, 20])?.logs[0]?.message).toBe(
      'page completed',
    );
  });

  test('stores document text and region payloads by file hash', () => {
    const client = new QueryClient();
    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:03Z',
      payload: {
        boxes: [
          {
            height_percent: 4,
            label: 'Total',
            left_percent: 1,
            page_no: 1,
            region_id: 'r1',
            top_percent: 2,
            width_percent: 3,
          },
        ],
        file_hash: 'abc',
      },
      sequence: 10,
      type: 'document.regions.changed',
      version: 1,
    });
    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:04Z',
      payload: {
        file_hash: 'abc',
        pages: [
          { page_no: 1, spans: [{ end: 5, page_no: 1, region_id: 'r1', start: 0 }], text: 'Total' },
        ],
      },
      sequence: 11,
      type: 'document.text.changed',
      version: 1,
    });

    expect(client.getQueryData(queryKeys.documentRegions('abc'))).toEqual({
      boxes: [
        {
          height_percent: 4,
          label: 'Total',
          left_percent: 1,
          page_no: 1,
          region_id: 'r1',
          top_percent: 2,
          width_percent: 3,
        },
      ],
      file_hash: 'abc',
    });
    expect(client.getQueryData(queryKeys.documentText('abc'))).toEqual({
      file_hash: 'abc',
      pages: [
        { page_no: 1, spans: [{ end: 5, page_no: 1, region_id: 'r1', start: 0 }], text: 'Total' },
      ],
    });
  });
});

describe('realtime OCR stream events', () => {
  test('applies live OCR text patches to document text cache', () => {
    const client = new QueryClient();

    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:05Z',
      payload: {
        file_hash: 'abc',
        page_no: 1,
        run_id: 'run-a',
      },
      sequence: 12,
      type: 'ocr.page.stream.started',
      version: 1,
    });
    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:06Z',
      payload: {
        delta: 'Invoice',
        elapsed_ms: 8,
        avg_tps: 125,
        file_hash: 'abc',
        page_no: 1,
        raw_end: 7,
        raw_start: 0,
        run_id: 'run-a',
        token_index: 0,
      },
      sequence: 13,
      type: 'ocr.page.raw.delta',
      version: 1,
    });
    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:07Z',
      payload: {
        end: 0,
        file_hash: 'abc',
        op: 'append',
        page_no: 1,
        run_id: 'run-a',
        start: 0,
        text: 'Invoice',
      },
      sequence: 14,
      type: 'ocr.page.text.patch',
      version: 1,
    });

    expect(client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc'))).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'Invoice' }],
    });
  });

  test('applies live OCR region upserts to document region cache', () => {
    const client = new QueryClient();

    applyRealtimeEventToQueryClient(client, {
      occurred_at: '2026-06-30T00:00:08Z',
      payload: {
        file_hash: 'abc',
        page_no: 1,
        region: {
          content_html: null,
          content_markdown: 'Total',
          height_percent: 10,
          hidden: false,
          label: 'Total',
          left_percent: 1,
          page_no: 1,
          region_id: 'r-live',
          top_percent: 2,
          width_percent: 20,
        },
        run_id: 'run-a',
      },
      sequence: 15,
      type: 'ocr.page.region.upsert',
      version: 1,
    });

    expect(client.getQueryData<DocumentRegionsPayload>(queryKeys.documentRegions('abc'))).toEqual({
      boxes: [
        {
          content_html: null,
          content_markdown: 'Total',
          height_percent: 10,
          hidden: false,
          label: 'Total',
          left_percent: 1,
          page_no: 1,
          region_id: 'r-live',
          top_percent: 2,
          width_percent: 20,
        },
      ],
      file_hash: 'abc',
    });
  });
});
