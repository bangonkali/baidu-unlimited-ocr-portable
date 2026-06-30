import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateSettingsSearch } from '../routeSearch';

export const Route = createFileRoute('/settings')({
  validateSearch: validateSettingsSearch,
  component: SettingsRoute,
});

function SettingsRoute() {
  const search = Route.useSearch();
  return <WorkbenchPage activeView="settings" settingsSearch={search} />;
}
