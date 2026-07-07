import { describe, expect, test } from 'bun:test';
import { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { DocumentRegionsPayload, DocumentTextPayload } from '../api/types';
import { applyProjectedOcrReplay } from './ocrReplayProjection';
import { applyRealtimeEventToQueryClient } from './realtimeQueryBridge';
import type { RealtimeEvent } from './realtimeTypes';

describe('realtime OCR replay projection', () => {
  test('projects replayed OCR patches idempotently', () => {
    const client = new QueryClient();
    const events: RealtimeEvent[] = [
      {
        occurred_at: '2026-06-30T00:00:05Z',
        payload: {
          file_hash: 'abc',
          page_no: 1,
          run_id: 'run-a',
        },
        sequence: 12,
        type: 'ocr.page.stream.started',
        version: 1,
      },
      {
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
      },
    ];

    applyProjectedOcrReplay(client, events);
    applyProjectedOcrReplay(client, events);

    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'Invoice' }],
      run_id: 'run-a',
    });
  });

  test('projects scoped replay payloads only into scoped query keys', () => {
    const client = new QueryClient();
    client.setQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a'), {
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'unscoped text' }],
      run_id: 'run-a',
    });
    const events: RealtimeEvent[] = [
      {
        occurred_at: '2026-06-30T00:00:05Z',
        payload: {
          file_hash: 'abc',
          page_no: 1,
          run_engine_id: 'engine-a',
          run_id: 'run-a',
        },
        sequence: 12,
        type: 'ocr.page.stream.started',
        version: 1,
      },
      {
        occurred_at: '2026-06-30T00:00:07Z',
        payload: {
          end: 0,
          file_hash: 'abc',
          op: 'append',
          page_no: 1,
          run_engine_id: 'engine-a',
          run_id: 'run-a',
          start: 0,
          text: 'engine text',
        },
        sequence: 14,
        type: 'ocr.page.text.patch',
        version: 1,
      },
    ];

    applyProjectedOcrReplay(client, events);

    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'unscoped text' }],
      run_id: 'run-a',
    });
    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a', 'engine-a')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'engine text' }],
      run_engine_id: 'engine-a',
      run_id: 'run-a',
    });
  });
});

describe('realtime OCR region events', () => {
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

    expect(
      client.getQueryData<DocumentRegionsPayload>(queryKeys.documentRegions('abc', 'run-a')),
    ).toEqual({
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
      run_id: 'run-a',
    });
  });
});
