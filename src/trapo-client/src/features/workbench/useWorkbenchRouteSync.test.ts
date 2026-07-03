import { describe, expect, test } from 'bun:test';
import type { ActiveView, WorkbenchState } from '../../stores/workbenchStore';
import { autoFollowEnabledForRoute } from './useWorkbenchRouteSync';
import { routeSearchFromSelection } from './useWorkbenchSelectionActions';

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

describe('routeSearchFromSelection', () => {
  test('keeps explicit follow state for focused workbench routes', () => {
    const state = workbenchState({ autoFollowRegions: true });
    expect(routeSearchFromSelection(state, state.selection, '')).toMatchObject({
      file: 'hash-doc',
      follow: true,
      page: 2,
    });

    const disabled = workbenchState({ autoFollowRegions: false });
    expect(routeSearchFromSelection(disabled, disabled.selection, '')).toMatchObject({
      file: 'hash-doc',
      follow: false,
      page: 2,
    });
  });
});

function workbenchState(patch: Partial<WorkbenchState>): WorkbenchState {
  return {
    activeView: 'workbench',
    autoFollowRegions: false,
    labelsVisible: true,
    overlayVisible: true,
    panesCollapsed: {
      details: true,
      diagnostics: true,
      explorer: false,
    },
    selectedProfile: 'experimental-exact-prefill-q4',
    selectedRoot: '',
    selection: {
      fileHash: 'hash-doc',
      pageNo: 2,
      regionId: 'reg-a',
    },
    theme: 'dark',
    tourRun: false,
    ...patch,
  };
}
