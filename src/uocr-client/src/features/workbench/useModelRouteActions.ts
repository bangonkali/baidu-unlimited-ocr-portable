import type { useNavigate } from '@tanstack/react-router';

import type { ModelRouteSearch } from '../../routeSearch';

export function useModelRouteActions(
  navigate: ReturnType<typeof useNavigate>,
  modelScope: 'library' | 'downloads',
  modelSearch?: ModelRouteSearch,
) {
  const updateModelRouteSearch = (patch: Partial<ModelRouteSearch>) => {
    const nextSearch: ModelRouteSearch = { ...modelSearch, ...patch };
    if (modelScope === 'downloads') {
      void navigate({ search: nextSearch, to: '/models/downloads' });
      return;
    }
    void navigate({ search: nextSearch, to: '/models' });
  };
  const changeModelScope = (scope: 'library' | 'downloads') => {
    void navigate({
      search: modelSearch ?? {},
      to: scope === 'downloads' ? '/models/downloads' : '/models',
    });
  };
  return { changeModelScope, updateModelRouteSearch };
}
