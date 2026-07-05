import { useEffect, useRef } from 'react';

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
  const seededFollowFileRef = useRef<string | undefined>(undefined);
  useEffect(() => {
    if (activeView !== 'workbench') {
      return;
    }
    const desiredAutoFollow = routeAutoFollowValue(activeView, workbenchSearch);
    const routeSelection = routeSelectionPatchForSync(
      activeView,
      workbench,
      workbenchSearch,
      seededFollowFileRef.current,
    );
    if (routeSelection) {
      setSelection(routeSelection);
    }
    seededFollowFileRef.current = desiredAutoFollow === true ? workbenchSearch?.file : undefined;
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

export function routeSelectionPatchForSync(
  activeView: ActiveView,
  workbench: Pick<WorkbenchState, 'selection'>,
  workbenchSearch?: WorkbenchRouteSearch,
  seededFollowFile?: string,
) {
  if (activeView !== 'workbench' || workbenchSearch?.file === undefined) {
    return undefined;
  }
  const follow = routeAutoFollowValue(activeView, workbenchSearch) === true;
  if (follow && seededFollowFile === workbenchSearch.file) {
    return undefined;
  }
  const routeSelection = {
    fileHash: workbenchSearch.file,
    pageNo: workbenchSearch.page ?? workbench.selection.pageNo,
    regionId: follow ? undefined : workbenchSearch.region,
  };
  return routeSelection.fileHash !== workbench.selection.fileHash || // skylos: ignore[SKY-D253] fileHash is route UI state, not a secret token.
    routeSelection.pageNo !== workbench.selection.pageNo ||
    routeSelection.regionId !== workbench.selection.regionId
    ? routeSelection
    : undefined;
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
