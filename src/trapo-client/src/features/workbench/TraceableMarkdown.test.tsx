import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import type { OverlayBox, PageTextRecord } from '../../api/types';
import { TraceableMarkdown } from './TraceableMarkdown';

describe('TraceableMarkdown', () => {
  test('renders region anchors without wrapping the scoped text', () => {
    const html = renderToString(
      <TraceableMarkdown
        onRegionSelect={() => undefined}
        page={pageWithText('A Title B', 'reg-title', 2)}
        regions={[region('reg-title')]}
        selectedRegionId="reg-title"
      />,
    );

    expect(html).toContain('id="annotation-text-reg-title"');
    expect(html).toContain('data-active="true"');
    expect(html).toContain('Title B</span>');
    expect(html).not.toContain('>Title B</button>');
    expect(html).not.toContain('data-region-id="reg-title"');
  });

  test('uses annotation ids for text anchor DOM identity', () => {
    const annotationId = '019086c9-8b0d-79af-9c3d-95c0c221b7e2';
    const html = renderToString(
      <TraceableMarkdown
        onRegionSelect={() => undefined}
        page={{
          page_no: 1,
          spans: [
            {
              annotation_id: annotationId,
              end: 7,
              page_no: 1,
              region_id: 'src_title',
              start: 2,
            },
          ],
          text: 'A Title B',
        }}
        regions={[region('src_title', { annotation_id: annotationId })]}
        selectedRegionId={annotationId}
      />,
    );

    expect(html).toContain(`id="annotation-text-${annotationId}"`);
    expect(html).not.toContain(`data-annotation-id="${annotationId}"`);
    expect(html).not.toContain('data-region-id="src_title"');
    expect(html).toContain('data-active="true"');
    expect(countOccurrences(html, annotationId)).toBe(1);
  });

  test('marks the selected text scope for focus glow without marking adjacent scopes', () => {
    const html = renderToString(
      <TraceableMarkdown
        glowRegionId="reg-first"
        onRegionSelect={() => undefined}
        page={{
          page_no: 1,
          spans: [
            { end: 5, page_no: 1, region_id: 'reg-first', start: 0 },
            { end: 16, page_no: 1, region_id: 'reg-second', start: 6 },
          ],
          text: 'First Second',
        }}
        regions={[region('reg-first'), region('reg-second')]}
        selectedRegionId="reg-first"
      />,
    );

    expect(html).toContain('data-glow="true"');
    expect(html).toContain('First');
    expect(html).toContain('Second');
    expect(countOccurrences(html, 'data-glow="true"')).toBe(1);
  });

  test('renders trusted OCR table markup as table elements without raw html injection', () => {
    const html = renderToString(
      <TraceableMarkdown
        onRegionSelect={() => undefined}
        page={{
          page_no: 1,
          spans: [],
          text: '<table><tr><th>Name</th></tr><tr><td>Total &amp; tax</td></tr></table>',
        }}
        regions={[]}
      />,
    );

    expect(html).toContain('<table>');
    expect(html).toContain('<th>');
    expect(html).toContain('Name');
    expect(html).toContain('Total &amp; tax');
  });

  test('embeds generated image snippets beside image region anchors', () => {
    const html = renderToString(
      <TraceableMarkdown
        onRegionSelect={() => undefined}
        page={pageWithText('Figure 1', 'reg-figure', 0)}
        regions={[
          region('reg-figure', {
            content_html: 'image-snippet',
            content_markdown: '![Figure 1](/api/documents/hash/regions/reg-figure/snippet)',
            label: 'Figure 1',
          }),
        ]}
      />,
    );

    expect(html).toContain('src="/api/documents/hash/regions/reg-figure/snippet"');
    expect(html).toContain('alt="Figure 1"');
  });
});

function pageWithText(text: string, regionId: string, start: number): PageTextRecord {
  return {
    page_no: 1,
    spans: [{ end: start, page_no: 1, region_id: regionId, start }],
    text,
  };
}

function region(regionId: string, overrides: Partial<OverlayBox> = {}): OverlayBox {
  return {
    content_html: null,
    content_markdown: 'Title B',
    height_percent: 10,
    hidden: false,
    label: 'Title',
    left_percent: 0,
    page_no: 1,
    region_id: regionId,
    top_percent: 0,
    width_percent: 10,
    ...overrides,
  };
}

function countOccurrences(value: string, pattern: string) {
  return value.split(pattern).length - 1;
}
