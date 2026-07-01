import { Store, useStore } from '@tanstack/react-store';

import type { OverlayBox, WorkbenchUiSettings, WorkbenchUiSettingsPatch } from '../api/types';

interface WorkbenchSelection {
  fileHash?: string;
  pageNo: number;
  regionId?: string;
}

export type ActiveView = 'workbench' | 'models' | 'settings' | 'diagnostics' | 'ingest';
export type ThemeMode = 'dark' | 'light';
export type WorkbenchPaneId = 'details' | 'diagnostics' | 'explorer';

export interface WorkbenchPaneState {
  details: boolean;
  diagnostics: boolean;
  explorer: boolean;
}

export interface WorkbenchState {
  activeView: ActiveView;
  selectedRoot: string;
  selectedProfile: string;
  selection: WorkbenchSelection;
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
  theme: readThemePreference(),
  tourRun: false,
};

const workbenchStore = new Store(initialState);

export function useWorkbenchState() {
  return useStore(workbenchStore, (state) => state);
}

export function getWorkbenchSnapshot() {
  return workbenchStore.state;
}

export function setSelectedRoot(selectedRoot: string) {
  workbenchStore.setState((state) => ({ ...state, selectedRoot }));
}

export function setSelectedProfile(selectedProfile: string) {
  workbenchStore.setState((state) => ({ ...state, selectedProfile }));
}

export function setSelection(selection: Partial<WorkbenchSelection>) {
  workbenchStore.setState((state) => ({
    ...state,
    selection: { ...state.selection, ...selection },
  }));
}

export function followLatestRegion(fileHash: string, boxes: OverlayBox[]) {
  const latest = boxes.at(-1);
  if (!latest || !workbenchStore.state.autoFollowRegions) {
    return;
  }
  workbenchStore.setState((state) => ({
    ...state,
    activeView: 'workbench',
    selection: {
      ...state.selection,
      fileHash,
      pageNo: latest.page_no,
      regionId: latest.region_id,
    },
  }));
}

export function setOverlayVisible(overlayVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, overlayVisible }));
}

export function setAutoFollowRegions(autoFollowRegions: boolean) {
  workbenchStore.setState((state) => ({ ...state, autoFollowRegions }));
}

export function setLabelsVisible(labelsVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, labelsVisible }));
}

export function hydrateWorkbenchUiSettings(settings: WorkbenchUiSettings) {
  applyThemePreference(settings.theme);
  persistThemePreference(settings.theme);
  workbenchStore.setState((state) => ({
    ...state,
    autoFollowRegions: settings.auto_follow_regions,
    labelsVisible: settings.labels_visible,
    overlayVisible: settings.overlay_visible,
    panesCollapsed: settings.panes_collapsed,
    theme: settings.theme,
  }));
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
  workbenchStore.setState((state) => ({
    ...state,
    panesCollapsed: { ...state.panesCollapsed, [pane]: collapsed },
  }));
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
  workbenchStore.setState((state) => ({ ...state, tourRun }));
}

function persistThemePreference(theme: ThemeMode) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(themeStorageKey, theme);
}
