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

    expect(html).toContain('data-region-id="reg-title"');
    expect(html).toContain('data-active="true"');
    expect(html).toContain('</span>Title B');
    expect(html).not.toContain('>Title B</button>');
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
