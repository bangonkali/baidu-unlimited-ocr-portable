import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateSearchSearch } from '../routeSearch';

export const Route = createFileRoute('/search')({
  validateSearch: validateSearchSearch,
  component: SearchRoute,
});

function SearchRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="search" searchSearch={search} />;
}
