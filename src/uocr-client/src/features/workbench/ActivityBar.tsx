import { Bot, Database, FileSearch } from 'lucide-react';

import { IconButton } from '../../components/IconButton';
import type { ActiveView } from '../../stores/workbenchStore';
import { setActiveView } from '../../stores/workbenchStore';
import styles from './WorkbenchPage.module.css';

export function ActivityBar({ activeView }: { activeView: ActiveView }) {
  return (
    <aside className={styles.activityBar} aria-label="Primary">
      <div className={styles.brand}>U</div>
      <nav className={styles.activityNav}>
        <IconButton
          icon={FileSearch}
          label="Workbench"
          onClick={() => setActiveView('workbench')}
          pressed={activeView === 'workbench'}
        />
        <div data-tour="models">
          <IconButton
            icon={Database}
            label="Models"
            onClick={() => setActiveView('models')}
            pressed={activeView === 'models'}
          />
        </div>
        <IconButton
          icon={Bot}
          label="Diagnostics"
          onClick={() => setActiveView('diagnostics')}
          pressed={activeView === 'diagnostics'}
        />
      </nav>
    </aside>
  );
}
