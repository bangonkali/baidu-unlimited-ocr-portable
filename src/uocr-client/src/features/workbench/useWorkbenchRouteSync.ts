import type { useNavigate } from '@tanstack/react-router';
import { useEffect } from 'react';

import type { DiagnosticsRouteSearch, WorkbenchRouteSearch } from '../../routeSearch';
import type { ActiveView, useWorkbenchState } from '../../stores/workbenchStore';
import {
  setAutoFollowRegions,
  setLabelsVisible,
  setOverlayVisible,
  setSelection,
} from '../../stores/workbenchStore';
import { sameWorkbenchSearch, workbenchSearchFromState } from './workbenchRouteState';

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
  navigate: ReturnType<typeof useNavigate>;
  searchText: string;
  workbench: ReturnType<typeof useWorkbenchState>;
  workbenchSearch?: WorkbenchRouteSearch;
}) {
  const { activeView, navigate, searchText, workbench, workbenchSearch } = args;
  useEffect(() => {
    if (activeView !== 'workbench') {
      return;
    }
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
    if (
      workbenchSearch?.follow !== undefined &&
      workbenchSearch.follow !== workbench.autoFollowRegions
    ) {
      setAutoFollowRegions(workbenchSearch.follow);
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

  useEffect(() => {
    if (activeView !== 'workbench') {
      return;
    }
    const next = workbenchSearchFromState(workbench, searchText);
    if (sameWorkbenchSearch(workbenchSearch, next)) {
      return;
    }
    void navigate({
      replace: true,
      search: (current) => ({ ...current, ...next }),
      to: '/workbench',
    });
  }, [activeView, navigate, searchText, workbench, workbenchSearch]);
}
