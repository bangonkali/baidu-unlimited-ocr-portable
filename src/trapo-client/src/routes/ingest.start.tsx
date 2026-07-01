import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateIngestSearch } from '../routeSearch';

export const Route = createFileRoute('/ingest/start')({
  validateSearch: validateIngestSearch,
  component: IngestStartRoute,
});

function IngestStartRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="ingest" ingestSearch={search} />;
}
