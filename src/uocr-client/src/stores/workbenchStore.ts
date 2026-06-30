import { Store, useStore } from '@tanstack/react-store';

interface WorkbenchSelection {
  fileHash?: string;
  pageNo: number;
  regionId?: string;
}

export type ActiveView = 'workbench' | 'models' | 'diagnostics';

interface WorkbenchState {
  activeView: ActiveView;
  selectedRoot: string;
  selectedProfile: string;
  selection: WorkbenchSelection;
  overlayVisible: boolean;
  labelsVisible: boolean;
  tourRun: boolean;
}

const initialState: WorkbenchState = {
  activeView: 'workbench',
  labelsVisible: true,
  overlayVisible: true,
  selectedProfile: 'experimental-exact-prefill-q4',
  selectedRoot: '',
  selection: {
    pageNo: 1,
  },
  tourRun: false,
};

const workbenchStore = new Store(initialState);

export function useWorkbenchState() {
  return useStore(workbenchStore, (state) => state);
}

export function setSelectedRoot(selectedRoot: string) {
  workbenchStore.setState((state) => ({ ...state, selectedRoot }));
}

export function setSelectedProfile(selectedProfile: string) {
  workbenchStore.setState((state) => ({ ...state, selectedProfile }));
}

export function setActiveView(activeView: WorkbenchState['activeView']) {
  workbenchStore.setState((state) => ({ ...state, activeView }));
}

export function setSelection(selection: Partial<WorkbenchSelection>) {
  workbenchStore.setState((state) => ({
    ...state,
    selection: { ...state.selection, ...selection },
  }));
}

export function setOverlayVisible(overlayVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, overlayVisible }));
}

export function setLabelsVisible(labelsVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, labelsVisible }));
}

export function setTourRun(tourRun: boolean) {
  workbenchStore.setState((state) => ({ ...state, tourRun }));
}
