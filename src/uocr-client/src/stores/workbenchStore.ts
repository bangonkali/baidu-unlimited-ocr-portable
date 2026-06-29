import { Store, useStore } from '@tanstack/react-store';

interface WorkbenchSelection {
  fileHash?: string;
  pageNo: number;
  regionId?: string;
}

interface WorkbenchState {
  selectedRoot: string;
  selectedProfile: string;
  selection: WorkbenchSelection;
  overlayVisible: boolean;
  labelsVisible: boolean;
}

const initialState: WorkbenchState = {
  labelsVisible: true,
  overlayVisible: true,
  selectedProfile: 'best-zero-empty-q4',
  selectedRoot: '',
  selection: {
    pageNo: 1,
  },
};

export const workbenchStore = new Store(initialState);

export function useWorkbenchState() {
  return useStore(workbenchStore, (state) => state);
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

export function setOverlayVisible(overlayVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, overlayVisible }));
}

export function setLabelsVisible(labelsVisible: boolean) {
  workbenchStore.setState((state) => ({ ...state, labelsVisible }));
}
