import type { WorkbenchRouteSearch } from '../../routeSearch';
import type { WorkbenchState } from '../../stores/workbenchStore';

export function workbenchSearchFromState(
  state: WorkbenchState,
  searchText: string,
): WorkbenchRouteSearch {
  return {
    file: state.selection.fileHash,
    follow: state.selection.fileHash ? state.autoFollowRegions : undefined,
    labels: state.labelsVisible ? undefined : false,
    overlays: state.overlayVisible ? undefined : false,
    page: state.selection.fileHash ? state.selection.pageNo : undefined,
    q: searchText.trim() || undefined,
    region: state.selection.regionId,
  };
}
