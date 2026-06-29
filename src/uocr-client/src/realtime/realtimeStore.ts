import { Store, useStore } from '@tanstack/react-store';

import type { RealtimeEvent } from './realtimeTypes';

export type RealtimeConnectionState = 'connecting' | 'connected' | 'disconnected';

interface RealtimeState {
  connectionState: RealtimeConnectionState;
  lastError?: string;
  lastEventAt?: string;
  lastSequence?: number;
}

const realtimeStore = new Store<RealtimeState>({
  connectionState: 'disconnected',
});

export function useRealtimeState() {
  return useStore(realtimeStore, (state) => state);
}

export function setRealtimeConnectionState(
  connectionState: RealtimeConnectionState,
  lastError?: string,
) {
  realtimeStore.setState((state) => ({ ...state, connectionState, lastError }));
}

export function markRealtimeEvent(event: RealtimeEvent) {
  realtimeStore.setState((state) => ({
    ...state,
    lastError: undefined,
    lastEventAt: event.occurred_at,
    lastSequence: event.sequence,
  }));
}
