import { beforeEach, describe, expect, test } from 'bun:test';

import {
  flushRealtimeFocusForTest,
  followLatestPage,
  followLatestRegion,
  getWorkbenchSnapshot,
  resetRealtimeFocusThrottleForTest,
  setAutoFollowRegions,
  setSelection,
} from './workbenchStore';

beforeEach(() => {
  setAutoFollowRegions(true);
  setSelection({ fileHash: 'start-doc', pageNo: 1, regionId: undefined, runId: undefined });
  resetRealtimeFocusThrottleForTest();
});

describe('workbenchStore realtime focus throttling', () => {
  test('throttles realtime page focus and flushes the latest pending page', () => {
    withMockedClock(10_000, () => {
      followLatestPage('live-doc', 2, 'run-a');
      followLatestPage('live-doc', 3, 'run-a');
    });

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 2,
      regionId: undefined,
      runId: 'run-a',
    });

    flushRealtimeFocusForTest();

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 3,
      regionId: undefined,
      runId: 'run-a',
    });
  });

  test('throttles realtime region focus and flushes the latest pending region', () => {
    withMockedClock(20_000, () => {
      followLatestRegion('live-doc', [box('reg-first', 1)], 'run-a');
      followLatestRegion('live-doc', [box('reg-first', 1), box('reg-latest', 2)], 'run-a');
    });

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 1,
      regionId: 'reg-first',
      runId: 'run-a',
    });

    flushRealtimeFocusForTest();

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 2,
      regionId: 'reg-latest',
      runId: 'run-a',
    });
  });

  test('manual selection cancels pending realtime focus', () => {
    withMockedClock(30_000, () => {
      followLatestPage('live-doc', 2, 'run-a');
      followLatestPage('live-doc', 3, 'run-a');
    });

    setSelection({ fileHash: 'manual-doc', pageNo: 9, regionId: 'manual-region' });
    flushRealtimeFocusForTest();

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'manual-doc',
      pageNo: 9,
      regionId: 'manual-region',
    });
    expect(getWorkbenchSnapshot().selectionSource).toBe('manual');
  });

  test('turning auto-follow off cancels pending realtime focus', () => {
    withMockedClock(40_000, () => {
      followLatestPage('live-doc', 2, 'run-a');
      followLatestPage('live-doc', 3, 'run-a');
    });

    setAutoFollowRegions(false);
    flushRealtimeFocusForTest();

    expect(getWorkbenchSnapshot().selection).toMatchObject({
      fileHash: 'live-doc',
      pageNo: 2,
      runId: 'run-a',
    });
  });
});

function box(regionId: string, pageNo: number) {
  return {
    height_percent: 10,
    label: regionId,
    left_percent: 10,
    page_no: pageNo,
    region_id: regionId,
    top_percent: 10,
    width_percent: 10,
  };
}

function withMockedClock(now: number, run: () => void) {
  const dateNow = Date.now;
  Date.now = () => now;
  try {
    run();
  } finally {
    Date.now = dateNow;
  }
}
