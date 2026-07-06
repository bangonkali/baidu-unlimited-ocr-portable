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
    expect(text.run_id).toBe('run-a');
    expect(text.pages[0]?.spans).toHaveLength(1);
    expect(regions.boxes).toHaveLength(1);
    expect(regions.run_id).toBe('run-a');

    text = applySpanRemove(text, { ...context, region_id: 'reg-total' });
    regions = applyRegionRemove(regions, { ...context, region_id: 'reg-total' });

    expect(text.pages[0]?.spans).toHaveLength(0);
    expect(regions.boxes).toHaveLength(0);
  });

  test('uses annotation ids to update and remove streamed regions', () => {
    const context = { file_hash: 'file-a', page_no: 1, run_id: 'run-a' };
    const annotationId = '019086c9-8b0d-79af-9c3d-95c0c221b7e2';
    let text = applySpanUpsert(undefined, {
      ...context,
      span: {
        annotation_id: annotationId,
        end: 5,
        page_no: 1,
        region_id: 'src_region-total',
        start: 0,
      },
    });
    let regions = applyRegionUpsert(undefined, {
      ...context,
      region: {
        annotation_id: annotationId,
        content_html: null,
        content_markdown: 'Total',
        height_percent: 10,
        hidden: false,
        label: 'Total',
        left_percent: 1,
        page_no: 1,
        region_id: 'src_region-total',
        top_percent: 2,
        width_percent: 9,
      },
    });

    text = applySpanRemove(text, { ...context, region_id: annotationId });
    regions = applyRegionRemove(regions, { ...context, region_id: annotationId });

    expect(text.pages[0]?.spans).toHaveLength(0);
    expect(regions.boxes).toHaveLength(0);
  });
});
