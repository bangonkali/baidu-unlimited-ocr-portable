import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateDiagnosticsSearch } from '../routeSearch';

export const Route = createFileRoute('/diagnostics')({
  validateSearch: validateDiagnosticsSearch,
  component: DiagnosticsRoute,
});

function DiagnosticsRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="diagnostics" diagnosticsSearch={search} />;
}
