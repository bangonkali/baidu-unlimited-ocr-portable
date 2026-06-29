import { describe, expect, test } from 'bun:test';
import { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { LogsPayload, ModelsPayload } from '../api/types';
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
