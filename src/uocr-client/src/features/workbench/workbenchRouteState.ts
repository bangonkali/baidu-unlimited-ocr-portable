import type {
  DiagnosticsRouteSearch,
  ModelRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
  WorkbenchView,
} from '../../routeSearch';
import type { WorkbenchState } from '../../stores/workbenchStore';

export type RouteSearchUpdate =
  | Partial<WorkbenchRouteSearch>
  | Partial<ModelRouteSearch>
  | Partial<SettingsRouteSearch>
  | Partial<DiagnosticsRouteSearch>;

export function viewPath(
  view: WorkbenchView,
): '/workbench' | '/models' | '/settings' | '/diagnostics' {
  switch (view) {
    case 'models':
      return '/models';
    case 'settings':
      return '/settings';
    case 'diagnostics':
      return '/diagnostics';
    case 'workbench':
      return '/workbench';
  }
}

export function workbenchSearchFromState(
  state: WorkbenchState,
  searchText: string,
): WorkbenchRouteSearch {
  return {
    file: state.selection.fileHash,
    follow: state.autoFollowRegions ? undefined : false,
    labels: state.labelsVisible ? undefined : false,
    overlays: state.overlayVisible ? undefined : false,
    page: state.selection.fileHash ? state.selection.pageNo : undefined,
    q: searchText.trim() || undefined,
    region: state.selection.regionId,
  };
}

export function sameWorkbenchSearch(
  left: WorkbenchRouteSearch | undefined,
  right: WorkbenchRouteSearch,
) {
  return (
    (left?.file ?? undefined) === (right.file ?? undefined) &&
    (left?.follow ?? undefined) === (right.follow ?? undefined) &&
    (left?.labels ?? undefined) === (right.labels ?? undefined) &&
    (left?.overlays ?? undefined) === (right.overlays ?? undefined) &&
    (left?.page ?? undefined) === (right.page ?? undefined) &&
    (left?.q ?? undefined) === (right.q ?? undefined) &&
    (left?.region ?? undefined) === (right.region ?? undefined)
  );
}
