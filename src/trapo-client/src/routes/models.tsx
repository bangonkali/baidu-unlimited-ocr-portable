import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateModelSearch } from '../routeSearch';

export const Route = createFileRoute('/models')({
  validateSearch: validateModelSearch,
  component: ModelsRoute,
});

function ModelsRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="models" modelSearch={search} modelScope="library" />;
}
