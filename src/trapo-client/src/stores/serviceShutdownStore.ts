import { Store, useStore } from '@tanstack/react-store';

import type { ShutdownPayload } from '../api/types';

export type ServiceMode = 'offline' | 'online' | 'shutting_down';

interface ServiceShutdownState {
  graceMs?: number;
  message?: string;
  mode: ServiceMode;
  source?: string;
}

const defaultMessage =
  'The Trapo service is not responding. Restart trapo-server, then retry the connection.';

const serviceShutdownStore = new Store<ServiceShutdownState>({
  mode: 'online',
});

export function useServiceShutdownState() {
  return useStore(serviceShutdownStore, (state) => state);
}

export function markShutdownRequested(payload: ShutdownPayload) {
  serviceShutdownStore.setState((state) =>
    state.mode === 'shutting_down' && state.message === payload.message
      ? state
      : {
          graceMs: payload.grace_ms,
          message: payload.message,
          mode: 'shutting_down',
          source: payload.source,
        },
  );
}

export function markServiceOffline(message = defaultMessage) {
  serviceShutdownStore.setState((state) =>
    state.mode === 'offline' && state.message === message
      ? state
      : { ...state, message, mode: 'offline' },
  );
}

export function markServiceOnline() {
  serviceShutdownStore.setState((state) => (state.mode === 'online' ? state : { mode: 'online' }));
}
