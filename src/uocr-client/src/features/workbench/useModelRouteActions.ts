import type { useNavigate } from '@tanstack/react-router';

import type { ModelRouteSearch } from '../../routeSearch';

export function useModelRouteActions(
  navigate: ReturnType<typeof useNavigate>,
  modelScope: 'library' | 'downloads',
  modelSearch?: ModelRouteSearch,
) {
  const updateModelRouteSearch = (patch: Partial<ModelRouteSearch>) => {
    if (modelScope === 'downloads') {
      void navigate({ search: (current) => ({ ...current, ...patch }), to: '/models/downloads' });
      return;
    }
    void navigate({ search: (current) => ({ ...current, ...patch }), to: '/models' });
  };
  const changeModelScope = (scope: 'library' | 'downloads') => {
    void navigate({
      search: modelSearch ?? {},
      to: scope === 'downloads' ? '/models/downloads' : '/models',
    });
  };
  return { changeModelScope, updateModelRouteSearch };
}
