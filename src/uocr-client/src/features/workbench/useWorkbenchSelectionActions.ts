import type { useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';

import type { WorkbenchRouteSearch } from '../../routeSearch';
import type { WorkbenchState } from '../../stores/workbenchStore';
import { setAutoFollowRegions, setSelection } from '../../stores/workbenchStore';

interface WorkbenchSelectionPatch {
  fileHash?: string;
  pageNo?: number;
  regionId?: string;
}

interface UseWorkbenchSelectionActionsArgs {
  navigate: ReturnType<typeof useNavigate>;
  searchText: string;
  workbench: WorkbenchState;
}

export function useWorkbenchSelectionActions({
  navigate,
  searchText,
  workbench,
}: UseWorkbenchSelectionActionsArgs) {
  const selectWorkbenchTarget = useCallback(
    (patch: WorkbenchSelectionPatch) => {
      const nextSelection = selectionFromPatch(workbench, patch);
      setAutoFollowRegions(false);
      setSelection(nextSelection);
      void navigate({
        replace: true,
        search: () => routeSearchFromSelection(workbench, nextSelection, searchText),
        to: '/workbench',
      });
    },
    [navigate, searchText, workbench],
  );

  const selectDocument = useCallback(
    (fileHash: string, pageNo = 1) =>
      selectWorkbenchTarget({ fileHash, pageNo, regionId: undefined }),
    [selectWorkbenchTarget],
  );

  const selectRegion = useCallback(
    (pageNo: number, regionId: string) => selectWorkbenchTarget({ pageNo, regionId }),
    [selectWorkbenchTarget],
  );

  return { selectDocument, selectRegion, selectWorkbenchTarget };
}

function selectionFromPatch(state: WorkbenchState, patch: WorkbenchSelectionPatch) {
  const fileChanged = patch.fileHash !== undefined && patch.fileHash !== state.selection.fileHash;
  const pageChanged = patch.pageNo !== undefined && patch.pageNo !== state.selection.pageNo;
  return {
    fileHash: patch.fileHash ?? state.selection.fileHash,
    pageNo: patch.pageNo ?? state.selection.pageNo,
    regionId:
      patch.regionId !== undefined
        ? patch.regionId
        : fileChanged || pageChanged
          ? undefined
          : state.selection.regionId,
  };
}

function routeSearchFromSelection(
  state: WorkbenchState,
  selection: ReturnType<typeof selectionFromPatch>,
  searchText: string,
): WorkbenchRouteSearch {
  return {
    file: selection.fileHash,
    follow: selection.fileHash ? false : undefined,
    labels: state.labelsVisible ? undefined : false,
    overlays: state.overlayVisible ? undefined : false,
    page: selection.fileHash ? selection.pageNo : undefined,
    q: searchText.trim() || undefined,
    region: selection.regionId,
  };
}
