import { Store, useStore } from '@tanstack/react-store';

import type { OverlayBox, WorkbenchUiSettings, WorkbenchUiSettingsPatch } from '../api/types';
import {
  createRealtimeFocusScheduler,
  latestRegionFocusTarget,
  pageFocusTarget,
  stateAfterRealtimeFocus,
} from './workbenchRealtimeFocus';

interface WorkbenchSelection {
  fileHash?: string;
  pageNo: number;
  regionId?: string;
  runId?: string;
}

export type ActiveView = 'workbench' | 'search' | 'models' | 'settings' | 'diagnostics' | 'ingest';
export type WorkbenchSelectionSource = 'manual' | 'realtime';
export type ThemeMode = 'dark' | 'light';
export type WorkbenchPaneId = 'details' | 'diagnostics' | 'explorer';

export interface WorkbenchPaneState {
  details: boolean;
  diagnostics: boolean;
  explorer: boolean;
}

export interface WorkbenchState {
  activeView: ActiveView;
  focusRevision: number;
  folderDialogError?: string;
  selectedRoot: string;
  selectedProfile: string;
  selection: WorkbenchSelection;
  selectionSource: WorkbenchSelectionSource;
  autoFollowRegions: boolean;
  overlayVisible: boolean;
  labelsVisible: boolean;
  panesCollapsed: WorkbenchPaneState;
  theme: ThemeMode;
  tourRun: boolean;
}

const themeStorageKey = 'trapo.theme';

const initialState: WorkbenchState = {
  activeView: 'workbench',
  autoFollowRegions: true,
  focusRevision: 0,
  folderDialogError: undefined,
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
    pageNo: 1,
  },
  selectionSource: 'manual',
  theme: readThemePreference(),
  tourRun: false,
};

const workbenchStore = new Store(initialState);
const realtimeFocus = createRealtimeFocusScheduler((target) => {
  workbenchStore.setState((state) => stateAfterRealtimeFocus(state, target));
});

export function useWorkbenchState() {
  return useStore(workbenchStore, (state) => state);
}

export function getWorkbenchSnapshot() {
  return workbenchStore.state;
}

export function setSelectedRoot(selectedRoot: string) {
  workbenchStore.setState((state) =>
    state.selectedRoot === selectedRoot ? state : { ...state, selectedRoot },
  );
}

export function setFolderDialogError(folderDialogError: string) {
  workbenchStore.setState((state) =>
    state.folderDialogError === folderDialogError ? state : { ...state, folderDialogError },
  );
}

export function clearFolderDialogError() {
  workbenchStore.setState((state) =>
    state.folderDialogError === undefined ? state : { ...state, folderDialogError: undefined },
  );
}

export function setSelectedProfile(selectedProfile: string) {
  workbenchStore.setState((state) =>
    state.selectedProfile === selectedProfile ? state : { ...state, selectedProfile },
  );
}

export function setSelection(selection: Partial<WorkbenchSelection>) {
  realtimeFocus.cancel();
  workbenchStore.setState((state) => {
    const next = { ...state.selection, ...selection };
    const nextFocusRevision =
      selection.regionId !== undefined ? state.focusRevision + 1 : state.focusRevision;
    return sameSelection(state.selection, next) &&
      state.selectionSource === 'manual' &&
      nextFocusRevision === state.focusRevision
      ? state
      : {
          ...state,
          focusRevision: nextFocusRevision,
          selection: next,
          selectionSource: 'manual',
        };
  });
}

export function followLatestRegion(fileHash: string, boxes: OverlayBox[], runId?: string | null) {
  const target = latestRegionFocusTarget(fileHash, boxes, runId);
  if (!target || !workbenchStore.state.autoFollowRegions) {
    return;
  }
  realtimeFocus.schedule(target);
}

export function followLatestPage(fileHash: string, pageNo: number, runId?: string | null) {
  if (!workbenchStore.state.autoFollowRegions) {
    return;
  }
  realtimeFocus.schedule(pageFocusTarget(fileHash, pageNo, runId));
}

export function setOverlayVisible(overlayVisible: boolean) {
  workbenchStore.setState((state) =>
    state.overlayVisible === overlayVisible ? state : { ...state, overlayVisible },
  );
}

export function setAutoFollowRegions(autoFollowRegions: boolean) {
  if (!autoFollowRegions) {
    realtimeFocus.cancel();
  }
  workbenchStore.setState((state) =>
    state.autoFollowRegions === autoFollowRegions ? state : { ...state, autoFollowRegions },
  );
}

export function setLabelsVisible(labelsVisible: boolean) {
  workbenchStore.setState((state) =>
    state.labelsVisible === labelsVisible ? state : { ...state, labelsVisible },
  );
}

export function hydrateWorkbenchUiSettings(settings: WorkbenchUiSettings) {
  if (!settings.auto_follow_regions) {
    realtimeFocus.cancel();
  }
  applyThemePreference(settings.theme);
  persistThemePreference(settings.theme);
  workbenchStore.setState((state) => {
    const panesCollapsed = settings.panes_collapsed;
    const unchanged =
      state.autoFollowRegions === settings.auto_follow_regions &&
      state.labelsVisible === settings.labels_visible &&
      state.overlayVisible === settings.overlay_visible &&
      state.theme === settings.theme &&
      samePanes(state.panesCollapsed, panesCollapsed);
    return unchanged
      ? state
      : {
          ...state,
          autoFollowRegions: settings.auto_follow_regions,
          labelsVisible: settings.labels_visible,
          overlayVisible: settings.overlay_visible,
          panesCollapsed,
          theme: settings.theme,
        };
  });
}

export function workbenchUiSettingsFromState(
  state = workbenchStore.state,
): WorkbenchUiSettingsPatch {
  return {
    auto_follow_regions: state.autoFollowRegions,
    labels_visible: state.labelsVisible,
    overlay_visible: state.overlayVisible,
    panes_collapsed: state.panesCollapsed,
    theme: state.theme,
  };
}

export function setPaneCollapsed(pane: WorkbenchPaneId, collapsed: boolean) {
  workbenchStore.setState((state) =>
    state.panesCollapsed[pane] === collapsed
      ? state
      : {
          ...state,
          panesCollapsed: { ...state.panesCollapsed, [pane]: collapsed },
        },
  );
}

export function togglePaneCollapsed(pane: WorkbenchPaneId) {
  workbenchStore.setState((state) => ({
    ...state,
    panesCollapsed: {
      ...state.panesCollapsed,
      [pane]: !state.panesCollapsed[pane],
    },
  }));
}

export function setTheme(theme: ThemeMode) {
  if (workbenchStore.state.theme === theme) {
    return;
  }
  persistThemePreference(theme);
  applyThemePreference(theme);
  workbenchStore.setState((state) => ({ ...state, theme }));
}

export function applyThemePreference(theme: ThemeMode) {
  if (typeof document === 'undefined') {
    return;
  }
  document.documentElement.dataset.theme = theme;
}

export function readThemePreference(): ThemeMode {
  if (typeof window === 'undefined') {
    return 'dark';
  }
  const stored = window.localStorage.getItem(themeStorageKey);
  if (stored === 'dark' || stored === 'light') {
    return stored;
  }
  return window.matchMedia?.('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

export function setTourRun(tourRun: boolean) {
  workbenchStore.setState((state) => (state.tourRun === tourRun ? state : { ...state, tourRun }));
}

export function flushRealtimeFocusForTest() {
  realtimeFocus.flush();
}

export function resetRealtimeFocusThrottleForTest() {
  realtimeFocus.reset();
}

function persistThemePreference(theme: ThemeMode) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(themeStorageKey, theme);
}

function sameSelection(left: WorkbenchSelection, right: WorkbenchSelection) {
  return (
    left.fileHash === right.fileHash &&
    left.pageNo === right.pageNo &&
    left.regionId === right.regionId &&
    left.runId === right.runId
  );
}

function samePanes(left: WorkbenchPaneState, right: WorkbenchPaneState) {
  return (
    left.details === right.details &&
    left.diagnostics === right.diagnostics &&
    left.explorer === right.explorer
  );
}
