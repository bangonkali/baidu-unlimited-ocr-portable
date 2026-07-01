import { useEffect } from 'react';

import type { DiagnosticsRouteSearch, WorkbenchRouteSearch } from '../../routeSearch';
import type { ActiveView, useWorkbenchState, WorkbenchState } from '../../stores/workbenchStore';
import {
  setAutoFollowRegions,
  setLabelsVisible,
  setOverlayVisible,
  setSelection,
} from '../../stores/workbenchStore';

export function useRouteSearchText(
  activeView: ActiveView,
  diagnosticsSearch: DiagnosticsRouteSearch | undefined,
  workbenchSearch: WorkbenchRouteSearch | undefined,
  searchText: string,
  setSearchText: (value: string) => void,
) {
  useEffect(() => {
    const routeText =
      activeView === 'diagnostics' ? (diagnosticsSearch?.q ?? '') : (workbenchSearch?.q ?? '');
    if (activeView !== 'diagnostics' && activeView !== 'workbench') {
      return;
    }
    if (routeText !== searchText) {
      setSearchText(routeText);
    }
  }, [activeView, diagnosticsSearch?.q, searchText, setSearchText, workbenchSearch?.q]);
}

export function useRouteSearchSync(args: {
  activeView: ActiveView;
  workbench: ReturnType<typeof useWorkbenchState>;
  workbenchSearch?: WorkbenchRouteSearch;
}) {
  const { activeView, workbench, workbenchSearch } = args;
  useEffect(() => {
    if (activeView !== 'workbench') {
      return;
    }
    const desiredAutoFollow = routeAutoFollowValue(activeView, workbenchSearch);
    const routeSelection = {
      fileHash: workbenchSearch?.file,
      pageNo: workbenchSearch?.page ?? workbench.selection.pageNo,
      regionId: workbenchSearch?.region,
    };
    if (
      routeSelection.fileHash !== undefined &&
      (routeSelection.fileHash !== workbench.selection.fileHash ||
        routeSelection.pageNo !== workbench.selection.pageNo ||
        routeSelection.regionId !== workbench.selection.regionId)
    ) {
      setSelection(routeSelection);
    }
    if (desiredAutoFollow !== undefined && desiredAutoFollow !== workbench.autoFollowRegions) {
      setAutoFollowRegions(desiredAutoFollow);
    }
    if (
      workbenchSearch?.labels !== undefined &&
      workbenchSearch.labels !== workbench.labelsVisible
    ) {
      setLabelsVisible(workbenchSearch.labels);
    }
    if (
      workbenchSearch?.overlays !== undefined &&
      workbenchSearch.overlays !== workbench.overlayVisible
    ) {
      setOverlayVisible(workbenchSearch.overlays);
    }
  }, [activeView, workbench, workbenchSearch]);
}

export function autoFollowEnabledForRoute(
  activeView: ActiveView,
  workbench: Pick<WorkbenchState, 'autoFollowRegions'>,
  workbenchSearch?: WorkbenchRouteSearch,
) {
  return routeAutoFollowValue(activeView, workbenchSearch) ?? workbench.autoFollowRegions;
}

function routeAutoFollowValue(
  activeView: ActiveView,
  workbenchSearch?: WorkbenchRouteSearch,
): boolean | undefined {
  if (activeView !== 'workbench') {
    return undefined;
  }
  if (workbenchSearch?.follow !== undefined) {
    return workbenchSearch.follow;
  }
  return routeHasManualFocus(workbenchSearch) ? false : undefined;
}

function routeHasManualFocus(workbenchSearch?: WorkbenchRouteSearch) {
  return (
    workbenchSearch?.file !== undefined ||
    workbenchSearch?.page !== undefined ||
    workbenchSearch?.region !== undefined
  );
}
