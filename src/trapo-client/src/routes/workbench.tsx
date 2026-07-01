import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateWorkbenchSearch } from '../routeSearch';

export const Route = createFileRoute('/workbench')({
  validateSearch: validateWorkbenchSearch,
  component: WorkbenchRoute,
});

function WorkbenchRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="workbench" workbenchSearch={search} />;
}
