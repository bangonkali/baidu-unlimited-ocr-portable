import { describe, expect, test } from 'bun:test';
import { QueryClient } from '@tanstack/react-query';

import { queryKeys } from '../api/queryKeys';
import type { DocumentTextPayload, OcrReplayPayload } from '../api/types';
import { createRealtimeDispatcher } from './RealtimeBridge';
import type { RealtimeEvent } from './realtimeTypes';

describe('RealtimeBridge dispatcher recovery', () => {
  test('recovers one sequence gap without repeated replay or broad invalidation', async () => {
    const client = new QueryClient();
    const replayCalls: number[] = [];
    const invalidations: string[] = [];
    const dispatcher = createRealtimeDispatcher(client, {
      fetchReplay: async (sinceSequence) => {
        replayCalls.push(sinceSequence);
        return replayPayload([textPatchEvent(2, 'A', 0), textPatchEvent(3, 'B', 1)]);
      },
      invalidateStateBackedQueries: () => invalidations.push('invalidate'),
    });

    await dispatcher.enqueue(connectionReadyEvent(1));
    await dispatcher.enqueue(textPatchEvent(4, 'C', 2));
    await dispatcher.enqueue(textPatchEvent(4, 'C', 2));

    expect(replayCalls).toEqual([1]);
    expect(invalidations).toEqual([]);
    expect(
      client.getQueryData<DocumentTextPayload>(queryKeys.documentText('file-a', 'run-a')),
    ).toEqual({
      file_hash: 'file-a',
      pages: [{ page_no: 1, spans: [], text: 'ABC' }],
      run_id: 'run-a',
    });
    expect(dispatcher.getSequenceState().lastAppliedSequence).toBe(4);
    expect(dispatcher.getSequenceState().lastAppliedOcrSequence).toBe(4);
  });

  test('does not replay or invalidate for non-OCR sequence gaps', async () => {
    const client = new QueryClient();
    const replayCalls: number[] = [];
    const invalidations: string[] = [];
    const dispatcher = createRealtimeDispatcher(client, {
      fetchReplay: async (sinceSequence) => {
        replayCalls.push(sinceSequence);
        return replayPayload([]);
      },
      invalidateStateBackedQueries: () => invalidations.push('invalidate'),
    });

    await dispatcher.enqueue(connectionReadyEvent(1));
    await dispatcher.enqueue(statusEvent(5, 'downloading'));
    await dispatcher.enqueue(modelEvent(9, 'downloading'));

    expect(replayCalls).toEqual([]);
    expect(invalidations).toEqual([]);
    expect(client.getQueryData(queryKeys.status)).toMatchObject({ state: 'downloading' });
    expect(dispatcher.getSequenceState()).toMatchObject({
      lastAppliedSequence: 9,
      lastAppliedOcrSequence: 1,
      lastRecoveredSequence: 8,
    });
  });

  test('treats sparse OCR replay as caught up without broad invalidation', async () => {
    const client = new QueryClient();
    const replayCalls: number[] = [];
    const invalidations: string[] = [];
    const dispatcher = createRealtimeDispatcher(client, {
      fetchReplay: async (sinceSequence) => {
        replayCalls.push(sinceSequence);
        return replayPayload([]);
      },
      invalidateStateBackedQueries: () => invalidations.push('invalidate'),
    });

    await dispatcher.enqueue(connectionReadyEvent(1));
    await dispatcher.enqueue(textPatchEvent(5, 'A', 0));

    expect(replayCalls).toEqual([1]);
    expect(invalidations).toEqual([]);
    expect(dispatcher.getSequenceState().lastAppliedOcrSequence).toBe(5);
    expect(dispatcher.getSequenceState().lastRecoveredSequence).toBe(4);
    expect(dispatcher.getSequenceState().lastAppliedSequence).toBe(5);
  });

  test('invalidates state-backed queries once when OCR replay fetch fails', async () => {
    const client = new QueryClient();
    const invalidations: string[] = [];
    const dispatcher = createRealtimeDispatcher(client, {
      fetchReplay: async () => {
        throw new Error('replay unavailable');
      },
      invalidateStateBackedQueries: () => invalidations.push('invalidate'),
    });

    await dispatcher.enqueue(connectionReadyEvent(1));
    await dispatcher.enqueue(textPatchEvent(5, 'A', 0));

    expect(invalidations).toEqual(['invalidate']);
    expect(dispatcher.getSequenceState().lastRecoveredSequence).toBe(4);
    expect(dispatcher.getSequenceState().lastAppliedSequence).toBe(5);
  });
});

function replayPayload(events: RealtimeEvent[]): OcrReplayPayload {
  return {
    events: events.map((event) => ({
      occurred_at: event.occurred_at,
      file_hash: 'file-a',
      page_no: 1,
      payload: event.payload as Record<string, unknown>,
      run_id: 'run-a',
      sequence: event.sequence,
      type: event.type,
    })),
    next_since_sequence: events.at(-1)?.sequence ?? null,
  };
}

function connectionReadyEvent(lastSequence: number): RealtimeEvent {
  return {
    occurred_at: '2026-07-05T00:00:00Z',
    payload: { last_sequence: lastSequence },
    sequence: lastSequence,
    type: 'connection.ready',
    version: 1,
  };
}

function statusEvent(sequence: number, state: string): RealtimeEvent {
  return {
    occurred_at: '2026-07-05T00:00:01Z',
    payload: {
      default_profile: 'experimental-exact-prefill-q4',
      state,
      supported_inputs: ['.pdf'],
    },
    sequence,
    type: 'status.changed',
    version: 1,
  };
}

function modelEvent(sequence: number, status: string): RealtimeEvent {
  return {
    occurred_at: '2026-07-05T00:00:01Z',
    payload: {
      accelerator: 'cuda',
      current_file: 'model.gguf',
      files: [],
      model_id: 'unlimited-ocr-bf16',
      model_label: 'Unlimited OCR BF16',
      runtime_compatible: true,
      selected: true,
      status,
    },
    sequence,
    type: 'model.changed',
    version: 1,
  };
}

function textPatchEvent(sequence: number, text: string, start: number): RealtimeEvent {
  return {
    occurred_at: '2026-07-05T00:00:02Z',
    payload: {
      end: start,
      file_hash: 'file-a',
      op: 'append',
      page_no: 1,
      run_id: 'run-a',
      start,
      text,
    },
    sequence,
    type: 'ocr.page.text.patch',
    version: 1,
  };
}
