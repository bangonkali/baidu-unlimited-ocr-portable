import type { useNavigate } from '@tanstack/react-router';

import type { ModelRouteSearch, RootRouteSearch } from '../../routeSearch';

type ModelSearchWithRoot = ModelRouteSearch & RootRouteSearch;

export function useModelRouteActions(
  navigate: ReturnType<typeof useNavigate>,
  modelScope: 'library' | 'downloads',
  modelSearch?: ModelRouteSearch,
) {
  const updateModelRouteSearch = (patch: Partial<ModelRouteSearch>) => {
    const nextSearch: ModelSearchWithRoot = { ...modelSearch, ...patch };
    if (modelScope === 'downloads') {
      void navigate({ search: nextSearch, to: '/models/downloads' });
      return;
    }
    void navigate({ search: nextSearch, to: '/models' });
  };
  const changeModelScope = (scope: 'library' | 'downloads') => {
    const nextSearch: ModelSearchWithRoot = {
      ...modelSearch,
      downloads: scope === 'downloads' ? true : undefined,
    };
    void navigate({
      search: nextSearch,
      to: scope === 'downloads' ? '/models/downloads' : '/models',
    });
  };
  return { changeModelScope, updateModelRouteSearch };
}
