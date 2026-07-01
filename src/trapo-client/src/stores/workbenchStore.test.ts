import { describe, expect, test } from 'bun:test';

import {
  followLatestRegion,
  getWorkbenchSnapshot,
  hydrateWorkbenchUiSettings,
  setAutoFollowRegions,
  setSelection,
  setTheme,
  workbenchUiSettingsFromState,
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

describe('workbenchStore theme', () => {
  test('stores the selected theme without requiring browser globals', () => {
    setTheme('light');
    expect(getWorkbenchSnapshot().theme).toBe('light');
    setTheme('dark');
    expect(getWorkbenchSnapshot().theme).toBe('dark');
  });
});

describe('workbenchStore UI settings', () => {
  test('defaults to explorer open with details and diagnostics collapsed', () => {
    hydrateWorkbenchUiSettings({
      auto_follow_regions: true,
      labels_visible: true,
      overlay_visible: true,
      panes_collapsed: {
        details: true,
        diagnostics: true,
        explorer: false,
      },
      theme: 'dark',
    });

    expect(getWorkbenchSnapshot().panesCollapsed).toEqual({
      details: true,
      diagnostics: true,
      explorer: false,
    });
  });

  test('hydrates and serializes persisted UI settings', () => {
    hydrateWorkbenchUiSettings({
      auto_follow_regions: false,
      labels_visible: false,
      overlay_visible: true,
      panes_collapsed: {
        details: false,
        diagnostics: true,
        explorer: true,
      },
      theme: 'light',
    });

    expect(workbenchUiSettingsFromState(getWorkbenchSnapshot())).toEqual({
      auto_follow_regions: false,
      labels_visible: false,
      overlay_visible: true,
      panes_collapsed: {
        details: false,
        diagnostics: true,
        explorer: true,
      },
      theme: 'light',
    });
  });
});
