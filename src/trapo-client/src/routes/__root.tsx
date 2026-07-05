import { createRootRoute, Outlet, retainSearchParams } from '@tanstack/react-router';
import { useCallback, useEffect, useMemo, useRef } from 'react';

import { useCancelModelDownload, useModels } from '../api/hooks';
import { activeDownloadItems, DownloadManager } from '../features/workbench/DownloadManager';
import type { DownloadsPaneContextValue } from '../features/workbench/downloadsPaneContext';
import { DownloadsPaneContext } from '../features/workbench/downloadsPaneContext';
import { validateRootSearch } from '../routeSearch';

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
  const navigate = Route.useNavigate();
  const models = useModels();
  const cancelDownload = useCancelModelDownload();
  const activeFileCount = activeDownloadItems(models.data?.models ?? []).length;
  const previousActiveFileCountRef = useRef(0);
  const isOpen = search.downloads === true;
  const setOpen = useCallback(
    (open: boolean) => {
      void navigate({
        replace: true,
        search: (current) => ({
          ...current,
          downloads: open ? true : undefined,
        }),
      });
    },
    [navigate],
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

  return (
    <DownloadsPaneContext.Provider value={pane}>
      <Outlet />
      {isOpen ? (
        <DownloadManager
          busy={cancelDownload.isPending}
          models={models.data?.models ?? []}
          onCancelModel={cancelDownload.mutate}
          onClose={pane.close}
        />
      ) : null}
    </DownloadsPaneContext.Provider>
  );
}
