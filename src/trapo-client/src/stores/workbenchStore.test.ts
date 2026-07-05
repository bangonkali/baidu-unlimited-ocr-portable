import { describe, expect, test } from 'bun:test';

import {
  followLatestPage,
  followLatestRegion,
  getWorkbenchSnapshot,
  hydrateWorkbenchUiSettings,
  setAutoFollowRegions,
  setLabelsVisible,
  setOverlayVisible,
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

  test('marks realtime and manual selection sources', () => {
    setAutoFollowRegions(true);
    setSelection({ fileHash: 'manual-doc', pageNo: 1, regionId: undefined });
    expect(getWorkbenchSnapshot().selectionSource).toBe('manual');

    followLatestPage('live-doc', 4);
    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 4,
    });
    expect(getWorkbenchSnapshot().selectionSource).toBe('realtime');

    setSelection({ pageNo: 2 });
    expect(getWorkbenchSnapshot().selectionSource).toBe('manual');
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

describe('workbenchStore no-op guards', () => {
  test('keeps the same state object for no-op realtime selection and visibility updates', () => {
    setAutoFollowRegions(true);
    setSelection({ fileHash: 'file-a', pageNo: 3, regionId: undefined });

    const selectionState = getWorkbenchSnapshot();
    setSelection({ fileHash: 'file-a', pageNo: 3, regionId: undefined });
    followLatestPage('file-a', 3);
    expect(getWorkbenchSnapshot()).toBe(selectionState);

    const visibilityState = getWorkbenchSnapshot();
    setOverlayVisible(visibilityState.overlayVisible);
    setLabelsVisible(visibilityState.labelsVisible);
    setAutoFollowRegions(visibilityState.autoFollowRegions);
    expect(getWorkbenchSnapshot()).toBe(visibilityState);
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
