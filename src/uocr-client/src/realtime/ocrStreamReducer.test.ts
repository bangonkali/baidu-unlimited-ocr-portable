import { describe, expect, test } from 'bun:test';

import {
  applyMetricPatch,
  applyRegionRemove,
  applyRegionUpsert,
  applySpanRemove,
  applySpanUpsert,
  applyTextPatch,
  ensureTextPage,
} from './ocrStreamReducer';

describe('ocr stream reducer', () => {
  test('reconstructs page text, spans, and regions from atoms', () => {
    const context = { file_hash: 'file-a', page_no: 1, run_id: 'run-a' };
    let text = ensureTextPage(undefined, context);
    text = applyTextPatch(text, { ...context, end: 0, op: 'append', start: 0, text: 'Invoice' });
    text = applyTextPatch(text, { ...context, end: 7, op: 'append', start: 7, text: ' total' });
    text = applySpanUpsert(text, {
      ...context,
      span: { end: 13, page_no: 1, region_id: 'reg-total', start: 0 },
    });

    let regions = applyRegionUpsert(undefined, {
      ...context,
      region: {
        height_percent: 10,
        label: 'Invoice total',
        left_percent: 1,
        page_no: 1,
        region_id: 'reg-total',
        top_percent: 2,
        width_percent: 9,
      },
    });

    expect(text.pages[0]?.text).toBe('Invoice total');
    expect(text.pages[0]?.spans).toHaveLength(1);
    expect(regions.boxes).toHaveLength(1);

    text = applySpanRemove(text, { ...context, region_id: 'reg-total' });
    regions = applyRegionRemove(regions, { ...context, region_id: 'reg-total' });

    expect(text.pages[0]?.spans).toHaveLength(0);
    expect(regions.boxes).toHaveLength(0);
  });

  test('rolls page metrics up to file and run nodes', () => {
    const metrics = applyMetricPatch(undefined, {
      avg_tps: 20,
      chunk_count: 20,
      elapsed_ms: 1100,
      file_hash: 'file-a',
      first_token_latency_ms: 100,
      generation_duration_ms: 1000,
      max_tps: 24,
      min_tps: 18,
      page_no: 1,
      run_id: 'run-a',
      status: 'running',
      token_count: 20,
    });

    const run = metrics.nodes[0];
    const file = run?.children?.[0];
    const page = file?.children?.[0];

    expect(run?.token_count).toBe(20);
    expect(run?.avg_tps).toBe(20);
    expect(run?.status).toBe('running');
    expect(file?.page_count).toBe(1);
    expect(page?.label).toBe('Page 1');
  });
});
