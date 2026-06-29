import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router';

import { WorkbenchPage } from './features/workbench/WorkbenchPage';

function RootLayout() {
  return <Outlet />;
}

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  component: WorkbenchPage,
  getParentRoute: () => rootRoute,
  path: '/',
});

const settingsRoute = createRoute({
  component: WorkbenchPage,
  getParentRoute: () => rootRoute,
  path: '/settings',
});

const routeTree = rootRoute.addChildren([indexRoute, settingsRoute]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
