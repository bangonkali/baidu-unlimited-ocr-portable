import { CommandPalette } from '../commands/CommandPalette';
import { ActivityBar } from './ActivityBar';
import { GuidedTour } from './GuidedTour';
import type { WorkbenchPageProps } from './useWorkbenchPageController';
import { useWorkbenchPageController } from './useWorkbenchPageController';
import styles from './WorkbenchPage.module.css';
import { WorkbenchFooter } from './WorkbenchPageSupport';
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
      <ActivityBar activeView={controller.activeView} onStartOcr={controller.startOcr} />
      <main className={styles.main}>
        <div className={styles.body}>
          <WorkbenchViewContent {...controller.contentProps} />
        </div>
        <PageFooter {...controller.footerProps} />
      </main>
    </div>
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
