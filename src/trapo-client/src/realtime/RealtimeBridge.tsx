import type { QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { buildApiUrl, getJson } from '../api/http';
import { queryKeys } from '../api/queryKeys';
import type { OcrReplayPayload } from '../api/types';
import { RealtimeClient } from './realtimeClient';
import { applyRealtimeEventToQueryClient } from './realtimeQueryBridge';
import { markRealtimeEvent, setRealtimeConnectionState } from './realtimeStore';
import type { RealtimeEvent } from './realtimeTypes';
import { realtimeEventFromRecord } from './realtimeTypes';

interface RealtimeBridgeProps {
  queryClient: QueryClient;
}

export function RealtimeBridge({ queryClient }: RealtimeBridgeProps) {
  useEffect(() => {
    const dispatcher = createRealtimeDispatcher(queryClient);
    const client = new RealtimeClient({
      onEvent: (event) => {
        dispatcher.enqueue(event);
      },
      onStateChange: setRealtimeConnectionState,
    });
    client.connect();
    return () => client.close();
  }, [queryClient]);

  return null;
}

function createRealtimeDispatcher(queryClient: QueryClient) {
  let lastAppliedSequence = 0;
  let queue = Promise.resolve();

  const applyEvent = (event: RealtimeEvent) => {
    markRealtimeEvent(event);
    applyRealtimeEventToQueryClient(queryClient, event);
    if (event.type !== 'connection.ready') {
      lastAppliedSequence = Math.max(lastAppliedSequence, event.sequence);
    }
  };

  const recoverSince = async (sinceSequence: number) => {
    const replay = await getJson<OcrReplayPayload>(
      buildApiUrl('/api/ocr/events', { limit: 100_000, since_sequence: sinceSequence }),
    );
    for (const record of replay.events) {
      const event = realtimeEventFromRecord(record);
      if (event && event.sequence > lastAppliedSequence) {
        applyEvent(event);
      }
    }
    invalidateStateBackedQueries(queryClient);
  };

  const recoverBestEffort = async (sinceSequence: number) => {
    try {
      await recoverSince(sinceSequence);
    } catch {
      invalidateStateBackedQueries(queryClient);
    }
  };

  const handleEvent = async (event: RealtimeEvent) => {
    if (event.type === 'connection.ready') {
      markRealtimeEvent(event);
      invalidateStateBackedQueries(queryClient);
      if (lastAppliedSequence > 0 && (event.payload.last_sequence ?? 0) > lastAppliedSequence) {
        await recoverBestEffort(lastAppliedSequence);
      }
      return;
    }
    if (lastAppliedSequence > 0 && event.sequence > lastAppliedSequence + 1) {
      await recoverBestEffort(lastAppliedSequence);
    }
    if (event.sequence <= lastAppliedSequence) {
      return;
    }
    applyEvent(event);
  };

  return {
    enqueue: (event: RealtimeEvent) => {
      queue = queue
        .then(() => handleEvent(event))
        .catch(() => {
          invalidateStateBackedQueries(queryClient);
        });
    },
  };
}

function invalidateStateBackedQueries(queryClient: QueryClient) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.status });
  void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
  void queryClient.invalidateQueries({ queryKey: ['documents'], refetchType: 'active' });
  void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
}
