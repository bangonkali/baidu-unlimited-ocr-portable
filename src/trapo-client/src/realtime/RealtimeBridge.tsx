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

const replayPageLimit = 10_000;

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

interface RealtimeDispatcherOptions {
  fetchReplay?: (sinceSequence: number) => Promise<OcrReplayPayload>;
  invalidateStateBackedQueries?: (queryClient: QueryClient) => void;
}

export function createRealtimeDispatcher(
  queryClient: QueryClient,
  options: RealtimeDispatcherOptions = {},
) {
  const fetchReplay = options.fetchReplay ?? fetchOcrReplay;
  const invalidate = options.invalidateStateBackedQueries ?? invalidateStateBackedQueries;
  let hasReady = false;
  let lastAppliedSequence = 0;
  let lastAppliedOcrSequence = 0;
  let lastRecoveredSequence = 0;
  let lastSeenSequence = 0;
  let queue = Promise.resolve();

  const applyEvent = (event: RealtimeEvent) => {
    markRealtimeEvent(event);
    applyRealtimeEventToQueryClient(queryClient, event);
    if (event.type !== 'connection.ready') {
      lastAppliedSequence = Math.max(lastAppliedSequence, event.sequence);
      if (isOcrReplayEvent(event)) {
        lastAppliedOcrSequence = Math.max(lastAppliedOcrSequence, event.sequence);
      }
    }
  };

  const recoverOcrThrough = async (targetSequence: number) => {
    if (targetSequence <= lastAppliedOcrSequence || targetSequence <= lastRecoveredSequence) {
      return;
    }
    let cursor = lastAppliedOcrSequence;

    try {
      for (;;) {
        const replay = await fetchReplay(cursor);
        const events = replay.events
          .flatMap((record) => {
            const event = realtimeEventFromRecord(record);
            return event && isOcrReplayEvent(event) && event.sequence > lastAppliedOcrSequence
              ? [event]
              : [];
          })
          .sort((left, right) => left.sequence - right.sequence);

        for (const event of events) {
          applyEvent(event);
        }

        const nextSequence =
          replay.next_since_sequence ??
          replay.events.at(-1)?.sequence ??
          events.at(-1)?.sequence ??
          cursor;
        if (
          lastAppliedOcrSequence >= targetSequence ||
          replay.events.length < replayPageLimit ||
          nextSequence <= cursor
        ) {
          break;
        }
        cursor = nextSequence;
      }
    } catch {
      invalidate(queryClient);
      lastRecoveredSequence = Math.max(lastRecoveredSequence, targetSequence);
      return;
    }

    lastRecoveredSequence = Math.max(lastRecoveredSequence, targetSequence, lastAppliedOcrSequence);
  };

  const handleEvent = async (event: RealtimeEvent) => {
    if (event.type === 'connection.ready') {
      markRealtimeEvent(event);
      const readySequence = event.payload.last_sequence ?? event.sequence;
      lastSeenSequence = Math.max(lastSeenSequence, readySequence);
      if (!hasReady) {
        hasReady = true;
        lastAppliedSequence = Math.max(lastAppliedSequence, readySequence);
        lastAppliedOcrSequence = Math.max(lastAppliedOcrSequence, readySequence);
        lastRecoveredSequence = Math.max(lastRecoveredSequence, readySequence);
        return;
      }
      invalidate(queryClient);
      if (readySequence > lastAppliedOcrSequence) {
        await recoverOcrThrough(readySequence);
      }
      return;
    }
    lastSeenSequence = Math.max(lastSeenSequence, event.sequence);
    if (
      lastAppliedSequence > 0 &&
      event.sequence > lastAppliedSequence + 1 &&
      shouldRecoverOcrBefore(event)
    ) {
      await recoverOcrThrough(event.sequence - 1);
    } else if (lastAppliedSequence > 0 && event.sequence > lastAppliedSequence + 1) {
      lastRecoveredSequence = Math.max(lastRecoveredSequence, event.sequence - 1);
    }
    if (isOcrReplayEvent(event) && event.sequence <= lastAppliedOcrSequence) {
      return;
    }
    if (!isOcrReplayEvent(event) && event.sequence <= lastAppliedSequence) {
      return;
    }
    applyEvent(event);
  };

  return {
    enqueue: (event: RealtimeEvent) => {
      queue = queue
        .then(() => handleEvent(event))
        .catch(() => {
          invalidate(queryClient);
        });
      return queue;
    },
    getSequenceState: () => ({
      lastAppliedSequence,
      lastAppliedOcrSequence,
      lastRecoveredSequence,
      lastSeenSequence,
    }),
  };
}

function isOcrReplayEvent(event: RealtimeEvent) {
  return event.type.startsWith('ocr.page.');
}

function shouldRecoverOcrBefore(event: RealtimeEvent) {
  return (
    isOcrReplayEvent(event) ||
    event.type === 'document.changed' ||
    event.type === 'document.page.changed' ||
    event.type === 'document.regions.changed' ||
    event.type === 'document.text.changed' ||
    event.type === 'run.changed'
  );
}

async function fetchOcrReplay(sinceSequence: number) {
  return getJson<OcrReplayPayload>(
    buildApiUrl('/api/ocr/events', {
      limit: replayPageLimit,
      since_sequence: sinceSequence,
    }),
  );
}

export function invalidateStateBackedQueries(queryClient: QueryClient) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.status });
  void queryClient.invalidateQueries({ queryKey: queryKeys.models });
  void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
  void queryClient.invalidateQueries({ queryKey: ['documents'] });
  void queryClient.invalidateQueries({ queryKey: ['document-text'] });
  void queryClient.invalidateQueries({ queryKey: ['document-regions'] });
  void queryClient.invalidateQueries({ queryKey: ['document-preview-images'] });
  void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
}
