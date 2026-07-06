import { Power, RotateCw, ServerOff } from 'lucide-react';

import type { ServiceMode } from '../../stores/serviceShutdownStore';
import styles from './ServiceOfflinePage.module.css';

interface ServiceOfflinePageProps {
  busy?: boolean;
  message?: string;
  mode: ServiceMode;
  onRetry: () => void;
}

export function ServiceOfflinePage({ busy, message, mode, onRetry }: ServiceOfflinePageProps) {
  const cleanShutdown = mode === 'shutting_down';
  return (
    <main className={styles.page}>
      <section className={styles.panel} aria-label="Trapo service status">
        <div className={styles.icon}>
          {cleanShutdown ? <Power size={22} /> : <ServerOff size={22} />}
        </div>
        <h1>{cleanShutdown ? 'Trapo is shutting down' : 'Trapo service is offline'}</h1>
        <p>
          {message ??
            (cleanShutdown
              ? 'The local server accepted the shutdown request and is releasing local resources.'
              : 'The workbench cannot reach the local server right now.')}
        </p>
        <p className={styles.detail}>
          Restart trapo-server, then retry the connection. The browser view can recover without
          losing your place once the local API is available again.
        </p>
        <button className={styles.retry} disabled={busy} onClick={onRetry} type="button">
          <RotateCw size={14} />
          <span>{busy ? 'Checking' : 'Retry'}</span>
        </button>
      </section>
    </main>
  );
}
