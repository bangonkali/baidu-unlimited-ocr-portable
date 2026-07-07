import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateDiagnosticsSearch } from '../routeSearch';

export const Route = createFileRoute('/diagnostics/$workUnitId')({
  validateSearch: validateDiagnosticsSearch,
  component: DiagnosticWorkUnitRoute,
});

function DiagnosticWorkUnitRoute() {
  const { workUnitId } = Route.useParams();
  const search = Route.useSearch();
  return (
    <WorkbenchPage
      activeView="diagnostics"
      diagnosticsSearch={search}
      diagnosticWorkUnitId={workUnitId}
    />
  );
}
