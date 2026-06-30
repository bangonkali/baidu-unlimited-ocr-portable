import { createRootRoute, Outlet } from '@tanstack/react-router';

export const Route = createRootRoute({
  component: RootLayout,
  notFoundComponent: () => <div className="emptyState">Page not found.</div>,
});

function RootLayout() {
  return <Outlet />;
}
