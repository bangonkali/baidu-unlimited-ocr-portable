import { describe, expect, test } from 'bun:test';

import {
  followLatestRegion,
  getWorkbenchSnapshot,
  setAutoFollowRegions,
  setSelection,
} from './workbenchStore';

describe('workbenchStore auto-follow', () => {
  test('selects the newest region only when auto-follow is enabled', () => {
    setAutoFollowRegions(true);
    setSelection({ fileHash: 'initial', pageNo: 1, regionId: 'old' });

    followLatestRegion('hash-doc', [
      {
        height_percent: 10,
        label: 'first',
        left_percent: 10,
        page_no: 1,
        region_id: 'reg-first',
        top_percent: 10,
        width_percent: 10,
      },
      {
        height_percent: 8,
        label: 'latest',
        left_percent: 20,
        page_no: 2,
        region_id: 'reg-latest',
        top_percent: 20,
        width_percent: 12,
      },
    ]);

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'hash-doc',
      pageNo: 2,
      regionId: 'reg-latest',
    });

    setAutoFollowRegions(false);
    followLatestRegion('other-doc', [
      {
        height_percent: 4,
        label: 'ignored',
        left_percent: 1,
        page_no: 1,
        region_id: 'reg-ignored',
        top_percent: 1,
        width_percent: 4,
      },
    ]);

    expect(getWorkbenchSnapshot().selection.regionId).toBe('reg-latest');
    setAutoFollowRegions(true);
  });
});
