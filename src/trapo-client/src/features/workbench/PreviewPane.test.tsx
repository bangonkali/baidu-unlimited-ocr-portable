import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureBoxes } from '../../stories/fixtures/workbenchFixtures';
import { centeredScrollOffset, nearestScrollOffset, PreviewPane } from './PreviewPane';

describe('PreviewPane', () => {
  test('centers selected overlay inside the preview scroll root', () => {
    expect(
      centeredScrollOffset({
        rootScroll: 100,
        rootSize: 500,
        rootStart: 20,
        targetSize: 40,
        targetStart: 420,
      }),
    ).toBe(270);
  });

  test('reveals whole page transitions without centering tall pages', () => {
    expect(
      nearestScrollOffset({
        rootScroll: 100,
        rootSize: 500,
        rootStart: 20,
        targetSize: 900,
        targetStart: 650,
      }),
    ).toBe(730);
  });

  test('uses nearest edge when revealing smaller pages', () => {
    expect(
      nearestScrollOffset({
        rootScroll: 100,
        rootSize: 500,
        rootStart: 20,
        targetSize: 120,
        targetStart: 520,
      }),
    ).toBe(220);
  });

  test('renders visible auto-follow state and active box', () => {
    const html = renderToString(
      <PreviewPane
        autoFollowRegions
        boxes={fixtureBoxes}
        fileHash="hash-invoice-014"
        getImageUrl={() => 'data:image/png;base64,'}
        labelsVisible
        overlayVisible
        pages={[1]}
        selectedPageNo={1}
        selectedRegionId="reg-total"
        onAutoFollowChange={() => undefined}
        onSelectRegion={() => undefined}
      />,
    );

    expect(html).toContain('Auto Follow On');
    expect(html).toContain('data-active="true"');
    expect(html).toContain('Invoice total');
  });

  test('uses annotation ids for overlay DOM identity', () => {
    const annotationId = '019086c9-8b0d-79af-9c3d-95c0c221b7e2';
    const html = renderToString(
      <PreviewPane
        autoFollowRegions
        boxes={[
          {
            annotation_id: annotationId,
            content_html: null,
            content_markdown: 'Invoice total',
            height_percent: 10,
            hidden: false,
            label: 'Invoice total',
            left_percent: 5,
            page_no: 1,
            region_id: 'src_invoice-total',
            top_percent: 6,
            width_percent: 20,
          },
        ]}
        fileHash="hash-invoice-014"
        getImageUrl={() => 'data:image/png;base64,'}
        labelsVisible
        overlayVisible
        pages={[1]}
        selectedPageNo={1}
        selectedRegionId={annotationId}
        onAutoFollowChange={() => undefined}
        onSelectRegion={() => undefined}
      />,
    );

    expect(html).toContain(`id="annotation-box-${annotationId}"`);
    expect(html).not.toContain(`data-annotation-id="${annotationId}"`);
    expect(html).not.toContain('data-region-id="src_invoice-total"');
    expect(html).toContain('data-active="true"');
    expect(countOccurrences(html, annotationId)).toBe(1);
  });
});

function countOccurrences(value: string, pattern: string) {
  return value.split(pattern).length - 1;
}
