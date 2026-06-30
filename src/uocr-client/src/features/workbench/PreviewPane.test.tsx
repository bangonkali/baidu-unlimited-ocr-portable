import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureBoxes } from '../../stories/fixtures/workbenchFixtures';
import { PreviewPane } from './PreviewPane';

describe('PreviewPane', () => {
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
      />,
    );

    expect(html).toContain('Auto Follow On');
    expect(html).toContain('data-active="true"');
    expect(html).toContain('Invoice total');
  });
});
