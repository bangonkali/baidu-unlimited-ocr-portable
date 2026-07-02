import { describe, expect, test } from 'bun:test';

import {
  applyRegionRemove,
  applyRegionUpsert,
  applySpanRemove,
  applySpanUpsert,
  applyTextPatch,
  ensureTextPage,
} from './ocrStreamReducer';

describe('ocr stream reducer', () => {
  test('reconstructs live page text, spans, and regions from stream atoms', () => {
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
        content_html: null,
        content_markdown: 'Invoice total',
        height_percent: 10,
        hidden: false,
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
});
