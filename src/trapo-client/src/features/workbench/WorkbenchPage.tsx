import type { ActiveView, useWorkbenchState } from '../../stores/workbenchStore';
import { setTourRun, togglePaneCollapsed } from '../../stores/workbenchStore';
import { CommandPalette } from '../commands/CommandPalette';
import { ActivityBar } from './ActivityBar';
import { GuidedTour } from './GuidedTour';
import type { useWorkbenchCommands } from './useWorkbenchCommands';
import type { WorkbenchPageProps } from './useWorkbenchPageController';
import { useWorkbenchPageController } from './useWorkbenchPageController';
import styles from './WorkbenchPage.module.css';
import { WorkbenchFooter, WorkbenchHeader } from './WorkbenchPageSupport';
import { WorkbenchViewContent } from './WorkbenchViewContent';

export function WorkbenchPage(props: WorkbenchPageProps) {
  const controller = useWorkbenchPageController(props);
  return (
    <div className={styles.shell}>
      <GuidedTour
        onViewChange={controller.commandController.navigateView}
        run={controller.workbench.tourRun}
      />
      <CommandPalette
        commands={controller.commandController.commands}
        onExecute={controller.commandController.executeCommand}
        onOpenChange={controller.commandController.setCommandOpen}
        open={controller.commandController.commandOpen}
      />
      <ActivityBar activeView={controller.activeView} />
      <main className={styles.main}>
        <PageHeader
          activeView={controller.activeView}
          commandController={controller.commandController}
          workbench={controller.workbench}
        />
        <div className={styles.body}>
          <WorkbenchViewContent {...controller.contentProps} />
        </div>
        <PageFooter {...controller.footerProps} />
      </main>
    </div>
  );
}

function PageHeader({
  activeView,
  commandController,
  workbench,
}: {
  activeView: ActiveView;
  commandController: ReturnType<typeof useWorkbenchCommands>;
  workbench: ReturnType<typeof useWorkbenchState>;
}) {
  return (
    <WorkbenchHeader
      activeView={activeView}
      onCommandOpen={() => commandController.setCommandOpen(true)}
      onStartGuide={() => setTourRun(true)}
      onTogglePane={togglePaneCollapsed}
      panesCollapsed={workbench.panesCollapsed}
      theme={workbench.theme}
    />
  );
}

function PageFooter({
  documentCount,
  realtimeState,
  selectedRoot,
  status,
}: {
  documentCount: number;
  realtimeState: string;
  selectedRoot: string;
  status?: {
    accelerator?: string;
    log_path?: string;
    runtime_platform?: string;
    state?: string;
  };
}) {
  return (
    <WorkbenchFooter
      accelerator={status?.accelerator}
      documentCount={documentCount}
      logPath={status?.log_path}
      realtimeState={realtimeState}
      runState={status?.state ?? 'offline'}
      runtimePlatform={status?.runtime_platform}
      selectedRoot={selectedRoot}
    />
  );
}
