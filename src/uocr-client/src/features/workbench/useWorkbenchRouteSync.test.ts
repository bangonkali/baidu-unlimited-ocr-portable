import { describe, expect, test } from 'bun:test';

import type { ActiveView } from '../../stores/workbenchStore';
import { autoFollowEnabledForRoute } from './useWorkbenchRouteSync';

describe('autoFollowEnabledForRoute', () => {
  test('disables auto-follow for manual workbench deep links unless follow is explicit', () => {
    const activeView: ActiveView = 'workbench';
    const state = { autoFollowRegions: true };

    expect(
      autoFollowEnabledForRoute(activeView, state, {
        file: 'a55931e7bc7e2e14',
        page: 1,
      }),
    ).toBe(false);
    expect(
      autoFollowEnabledForRoute(activeView, state, {
        file: 'a55931e7bc7e2e14',
        follow: false,
        page: 1,
      }),
    ).toBe(false);
    expect(
      autoFollowEnabledForRoute(activeView, state, {
        file: 'a55931e7bc7e2e14',
        follow: true,
        page: 1,
      }),
    ).toBe(true);
  });

  test('uses store state when the route does not focus a file, page, or region', () => {
    expect(autoFollowEnabledForRoute('workbench', { autoFollowRegions: true })).toBe(true);
    expect(
      autoFollowEnabledForRoute('models', { autoFollowRegions: false }, { follow: true }),
    ).toBe(false);
  });
});
