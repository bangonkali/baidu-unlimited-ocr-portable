import type { useNavigate } from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';

import type { ModelsPayload, StatusPayload } from '../../api/types';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import type { WorkbenchState } from '../../stores/workbenchStore';
import { setTheme, setTourRun, togglePaneCollapsed } from '../../stores/workbenchStore';
import type { AppCommandAction, CommandRouteTarget } from '../commands/appCommands';
import { buildAppCommands } from '../commands/appCommands';
import { workbenchSearchFromState } from './workbenchRouteState';

interface ModelMutationInput {
  force?: boolean;
  modelId: string;
}

interface MutationLike<TInput> {
  mutate: (input: TInput) => void;
}

interface UseWorkbenchCommandsArgs {
  cancelModelDownload: MutationLike<string>;
  downloadModel: MutationLike<ModelMutationInput>;
  models?: ModelsPayload;
  navigate: ReturnType<typeof useNavigate>;
  selectModel: MutationLike<string>;
  setSearchText: (value: string) => void;
  status?: StatusPayload;
  workbench: WorkbenchState;
}

export function useWorkbenchCommands({
  cancelModelDownload,
  downloadModel,
  models,
  navigate,
  selectModel,
  setSearchText,
  status,
  workbench,
}: UseWorkbenchCommandsArgs) {
  const [commandOpen, setCommandOpen] = useState(false);
  useCommandShortcut(setCommandOpen);

  const commands = useMemo(
    () =>
      buildAppCommands({
        models,
        panesCollapsed: workbench.panesCollapsed,
        status,
        theme: workbench.theme,
      }),
    [models, status, workbench.panesCollapsed, workbench.theme],
  );

  const navigateView = (view: WorkbenchState['activeView']) => {
    switch (view) {
      case 'diagnostics':
        void navigate({ to: '/diagnostics' });
        return;
      case 'ingest':
        void navigate({ to: '/ingest/start' });
        return;
      case 'models':
        void navigate({ to: '/models' });
        return;
      case 'search':
        void navigate({ to: '/search' });
        return;
      case 'settings':
        void navigate({ to: '/settings' });
        return;
      case 'workbench':
        void navigate({ to: '/workbench' });
        return;
    }
  };

  const updateDiagnosticsRouteSearch = (patch: Partial<DiagnosticsRouteSearch>) => {
    void navigate({
      search: (current) => ({ ...current, ...patch }),
      to: '/diagnostics',
    });
  };

  const executeCommand = (action: AppCommandAction) => {
    switch (action.kind) {
      case 'cancelModelDownload':
        cancelModelDownload.mutate(action.modelId);
        return;
      case 'downloadModel':
        downloadModel.mutate({ force: action.force, modelId: action.modelId });
        return;
      case 'filterDocuments':
        setSearchText(action.q);
        void navigate({
          search: () => workbenchSearchFromState(workbench, action.q),
          to: '/workbench',
        });
        return;
      case 'navigate':
        navigateToCommandTarget(navigate, action.target);
        return;
      case 'openGuide':
        setTourRun(true);
        return;
      case 'selectModel':
        selectModel.mutate(action.modelId);
        return;
      case 'setTheme':
        setTheme(action.theme);
        return;
      case 'togglePane':
        togglePaneCollapsed(action.pane);
        return;
    }
  };

  return {
    commandOpen,
    commands,
    executeCommand,
    navigateView,
    setCommandOpen,
    updateDiagnosticsRouteSearch,
  };
}

function useCommandShortcut(onOpenChange: (open: boolean) => void) {
  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        onOpenChange(true);
      }
    };
    window.addEventListener('keydown', listener);
    return () => window.removeEventListener('keydown', listener);
  }, [onOpenChange]);
}

function navigateToCommandTarget(
  navigate: ReturnType<typeof useNavigate>,
  target: CommandRouteTarget,
) {
  switch (target.to) {
    case '/diagnostics':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/ingest/start':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/models':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/models/$modelId':
      void navigate({ params: target.params, search: target.search ?? {}, to: target.to });
      return;
    case '/models/downloads':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/search':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/settings':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
    case '/workbench':
      void navigate({ search: target.search ?? {}, to: target.to });
      return;
  }
}
