import { describe, expect, test } from 'bun:test';
import type { ActiveView, WorkbenchState } from '../../stores/workbenchStore';
import { autoFollowEnabledForRoute, routeSelectionPatchForSync } from './useWorkbenchRouteSync';
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

describe('routeSelectionPatchForSync', () => {
  test('does not fight realtime auto-follow after a follow route is seeded', () => {
    const state = workbenchState({
      selection: {
        fileHash: 'hash-doc',
        pageNo: 6,
        regionId: 'reg-live',
      },
    });

    expect(
      routeSelectionPatchForSync(
        'workbench',
        state,
        {
          file: 'hash-doc',
          follow: true,
          page: 1,
          run: 'run-live',
        },
        'run-live:hash-doc',
      ),
    ).toBeUndefined();
  });

  test('seeds explicit follow route selection once', () => {
    const state = workbenchState({
      selection: {
        fileHash: 'other-doc',
        pageNo: 6,
        regionId: 'reg-live',
      },
    });

    expect(
      routeSelectionPatchForSync('workbench', state, {
        file: 'hash-doc',
        follow: true,
        page: 1,
        run: 'run-a',
      }),
    ).toEqual({
      fileHash: 'hash-doc',
      pageNo: 1,
      regionId: undefined,
      runId: 'run-a',
    });
  });

  test('does not re-anchor follow route after realtime advances documents', () => {
    const state = workbenchState({
      selection: {
        fileHash: 'live-doc',
        pageNo: 3,
        regionId: undefined,
      },
    });

    expect(
      routeSelectionPatchForSync(
        'workbench',
        state,
        {
          file: 'hash-doc',
          follow: true,
          page: 1,
          run: 'run-a',
        },
        'run-a:hash-doc',
      ),
    ).toBeUndefined();
  });

  test('still applies route selection for manual deep links', () => {
    const state = workbenchState({
      selection: {
        fileHash: 'hash-doc',
        pageNo: 6,
        regionId: 'reg-live',
      },
    });

    expect(
      routeSelectionPatchForSync('workbench', state, {
        file: 'hash-doc',
        page: 1,
        run: 'run-manual',
      }),
    ).toEqual({
      fileHash: 'hash-doc',
      pageNo: 1,
      regionId: undefined,
      runId: 'run-manual',
    });
  });

  test('clears stale region focus when the route changes runs for the same file and page', () => {
    const state = workbenchState({
      selection: {
        fileHash: 'hash-doc',
        pageNo: 2,
        regionId: 'old-run-region',
        runId: 'run-old',
      },
    });

    expect(
      routeSelectionPatchForSync('workbench', state, {
        file: 'hash-doc',
        page: 2,
        run: 'run-new',
      }),
    ).toEqual({
      fileHash: 'hash-doc',
      pageNo: 2,
      regionId: undefined,
      runId: 'run-new',
    });
  });
});

describe('routeSearchFromSelection', () => {
  test('keeps explicit follow state for focused workbench routes', () => {
    const state = workbenchState({ autoFollowRegions: true });
    expect(routeSearchFromSelection(state, state.selection, '')).toMatchObject({
      file: 'hash-doc',
      follow: true,
      page: 2,
      run: 'run-a',
    });

    const disabled = workbenchState({ autoFollowRegions: false });
    expect(routeSearchFromSelection(disabled, disabled.selection, '')).toMatchObject({
      file: 'hash-doc',
      follow: false,
      page: 2,
      run: 'run-a',
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
      runId: 'run-a',
    },
    selectionSource: 'manual',
    theme: 'dark',
    tourRun: false,
    ...patch,
  };
}
