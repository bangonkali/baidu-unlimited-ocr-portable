import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateModelSearch } from '../routeSearch';

export const Route = createFileRoute('/models/downloads')({
  validateSearch: validateModelSearch,
  component: DownloadsRoute,
});

function DownloadsRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="models" modelSearch={search} modelScope="downloads" />;
}
