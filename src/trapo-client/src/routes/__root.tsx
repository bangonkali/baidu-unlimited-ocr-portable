import { useQuery } from '@tanstack/react-query';
import { createRootRoute, Outlet, retainSearchParams, useLocation } from '@tanstack/react-router';
import { useCallback, useEffect, useMemo, useRef } from 'react';

import { useCancelModelDownload, useModels, useSettings } from '../api/hooks';
import { getJson } from '../api/http';
import type { StatusPayload } from '../api/types';
import { activeDownloadItems, DownloadManager } from '../features/workbench/DownloadManager';
import type { DownloadsPaneContextValue } from '../features/workbench/downloadsPaneContext';
import { DownloadsPaneContext } from '../features/workbench/downloadsPaneContext';
import { ServiceOfflinePage } from '../features/workbench/ServiceOfflinePage';
import type { RootRouteSearch } from '../routeSearch';
import { validateRootSearch, withDownloadsPaneSearch } from '../routeSearch';
import {
  markServiceOffline,
  markServiceOnline,
  useServiceShutdownState,
} from '../stores/serviceShutdownStore';

export const Route = createRootRoute({
  component: RootLayout,
  notFoundComponent: () => <div className="emptyState">Page not found.</div>,
  search: {
    middlewares: [retainSearchParams(['downloads'])],
  },
  validateSearch: validateRootSearch,
});

function RootLayout() {
  const search = Route.useSearch();
  const location = useLocation();
  const navigate = Route.useNavigate();
  const models = useModels();
  const settings = useSettings();
  const cancelDownload = useCancelModelDownload();
  const service = useServiceShutdownState();
  const statusProbe = useServiceStatusProbe(service.mode);
  const activeFileCount = activeDownloadItems(models.data?.models ?? []).length;
  const previousActiveFileCountRef = useRef(0);
  const isOpen = search.downloads === true;
  const setOpen = useCallback(
    (open: boolean) => {
      void navigate({
        replace: true,
        search: (current: RootRouteSearch & Record<string, unknown>) =>
          withDownloadsPaneSearch(current, open),
        to: location.pathname,
      });
    },
    [location.pathname, navigate],
  );
  const pane = useMemo<DownloadsPaneContextValue>(
    () => ({
      activeFileCount,
      close: () => setOpen(false),
      isOpen,
      open: () => setOpen(true),
      toggle: () => setOpen(!isOpen),
    }),
    [activeFileCount, isOpen, setOpen],
  );

  useEffect(() => {
    if (!models.isSuccess) {
      return;
    }
    const previousCount = previousActiveFileCountRef.current;
    previousActiveFileCountRef.current = activeFileCount;
    if (activeFileCount > previousCount && !isOpen) {
      setOpen(true);
    }
  }, [activeFileCount, isOpen, models.isSuccess, setOpen]);

  useEffect(() => {
    if (statusProbe.isError && service.mode !== 'shutting_down') {
      markServiceOffline();
    }
    if (statusProbe.isSuccess && service.mode === 'offline') {
      markServiceOnline();
    }
  }, [service.mode, statusProbe.isError, statusProbe.isSuccess]);

  const retryService = useCallback(() => {
    void statusProbe.refetch().then((result) => {
      if (result.isSuccess) {
        markServiceOnline();
      } else {
        markServiceOffline();
      }
    });
  }, [statusProbe]);

  return (
    <DownloadsPaneContext.Provider value={pane}>
      {service.mode === 'online' ? (
        <Outlet />
      ) : (
        <ServiceOfflinePage
          busy={statusProbe.isFetching}
          message={service.message}
          mode={service.mode}
          onRetry={retryService}
        />
      )}
      {isOpen && service.mode === 'online' ? (
        <DownloadManager
          busy={cancelDownload.isPending}
          downloadConcurrency={settings.data?.download_concurrency}
          models={models.data?.models ?? []}
          onCancelModel={cancelDownload.mutate}
          onClose={pane.close}
        />
      ) : null}
    </DownloadsPaneContext.Provider>
  );
}

function useServiceStatusProbe(mode: string) {
  return useQuery({
    queryFn: ({ signal }) => getJson<StatusPayload>('/api/status', signal),
    queryKey: ['service-status-probe'],
    refetchInterval: mode === 'online' ? 5000 : 2000,
    retry: false,
  });
}
