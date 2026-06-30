import { createFileRoute } from '@tanstack/react-router';

import { WorkbenchPage } from '../features/workbench/WorkbenchPage';
import { validateModelSearch } from '../routeSearch';

export const Route = createFileRoute('/models/$modelId')({
  validateSearch: validateModelSearch,
  component: ModelDetailRoute,
});

function ModelDetailRoute() {
  const { modelId } = Route.useParams();
  const search = Route.useSearch();
  return (
    <WorkbenchPage
      activeView="models"
      modelDetailId={modelId}
      modelSearch={{ ...search, model: modelId }}
      modelScope="library"
    />
  );
}
