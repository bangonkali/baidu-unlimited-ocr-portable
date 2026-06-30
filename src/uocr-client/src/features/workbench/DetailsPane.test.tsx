import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureBoxes, fixtureDocuments } from '../../stories/fixtures/workbenchFixtures';
import { DetailsPane } from './DetailsPane';

describe('DetailsPane', () => {
  test('renders selected overlay content from persisted region payloads', () => {
    const html = renderToString(
      <DetailsPane
        document={fixtureDocuments[0]}
        labelsVisible
        overlayVisible
        selectedRegion={fixtureBoxes[0]}
        selectedRegionId="reg-total"
      />,
    );

    expect(html).toContain('Selected Box');
    expect(html).toContain('Invoice total: 1,240.00');
  });
});
