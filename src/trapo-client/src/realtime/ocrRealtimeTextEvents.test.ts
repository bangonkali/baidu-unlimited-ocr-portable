import { describe, expect, test } from 'bun:test';
import { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { DocumentTextPayload } from '../api/types';
import { applyRealtimeEventToQueryClient } from './realtimeQueryBridge';

describe('realtime OCR text patches', () => {
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
        avg_tps: 125,
        delta: 'Invoice',
        elapsed_ms: 8,
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

    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'Invoice' }],
      run_id: 'run-a',
    });
  });
});

describe('realtime OCR text isolation', () => {
  test('keeps live OCR text caches isolated by run id for the same file', () => {
    const client = new QueryClient();

    for (const [runId, text] of [
      ['run-a', 'First run'],
      ['run-b', 'Second run'],
    ] as const) {
      applyRealtimeEventToQueryClient(client, {
        occurred_at: '2026-06-30T00:00:05Z',
        payload: {
          file_hash: 'abc',
          page_no: 1,
          run_id: runId,
        },
        sequence: runId === 'run-a' ? 12 : 13,
        type: 'ocr.page.stream.started',
        version: 1,
      });
      applyRealtimeEventToQueryClient(client, {
        occurred_at: '2026-06-30T00:00:07Z',
        payload: {
          end: 0,
          file_hash: 'abc',
          op: 'append',
          page_no: 1,
          run_id: runId,
          start: 0,
          text,
        },
        sequence: runId === 'run-a' ? 14 : 15,
        type: 'ocr.page.text.patch',
        version: 1,
      });
    }

    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'First run' }],
      run_id: 'run-a',
    });
    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-b')),
    ).toEqual({
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'Second run' }],
      run_id: 'run-b',
    });
  });

  test('applies live scoped OCR text only to the selected engine cache', () => {
    const client = new QueryClient();
    client.setQueryData<DocumentTextPayload>(queryKeys.documentText('abc', 'run-a'), {
      file_hash: 'abc',
      pages: [{ page_no: 1, spans: [], text: 'unscoped text' }],
      run_id: 'run-a',
    });

    applyRealtimeEventToQueryClient(client, {
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
    });
    applyRealtimeEventToQueryClient(client, {
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
    });

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
