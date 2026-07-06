import { Link } from '@tanstack/react-router';
import { Bot, Database, FileSearch, Hammer, Search, Settings } from 'lucide-react';
import type { ComponentType } from 'react';

import type { ActiveView } from '../../stores/workbenchStore';
import styles from './WorkbenchPage.module.css';

export function ActivityBar({
  activeView,
  onStartOcr,
}: {
  activeView: ActiveView;
  onStartOcr: () => void;
}) {
  return (
    <aside className={styles.activityBar} aria-label="Primary">
      <button
        aria-label="Start OCR"
        className={styles.startOcrButton}
        data-tour="start-ocr"
        onClick={onStartOcr}
        title="Start OCR"
        type="button"
      >
        <Hammer size={18} strokeWidth={1.8} />
      </button>
      <nav className={styles.activityNav}>
        <ActivityLink
          active={activeView === 'workbench'}
          icon={FileSearch}
          label="Workbench"
          to="/workbench"
        />
        <ActivityLink active={activeView === 'search'} icon={Search} label="Search" to="/search" />
        <div data-tour="models">
          <ActivityLink
            active={activeView === 'models'}
            icon={Database}
            label="Models"
            to="/models"
          />
        </div>
        <ActivityLink
          active={activeView === 'settings'}
          icon={Settings}
          label="Settings"
          to="/settings"
        />
        <ActivityLink
          active={activeView === 'diagnostics'}
          icon={Bot}
          label="Diagnostics"
          to="/diagnostics"
        />
      </nav>
    </aside>
  );
}

function ActivityLink({
  active,
  icon: Icon,
  label,
  to,
}: {
  active: boolean;
  icon: ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  to: '/workbench' | '/search' | '/models' | '/settings' | '/diagnostics';
}) {
  return (
    <Link
      aria-label={label}
      aria-pressed={active}
      className={styles.activityLink}
      title={label}
      to={to}
    >
      <Icon size={17} strokeWidth={1.8} />
    </Link>
  );
}
