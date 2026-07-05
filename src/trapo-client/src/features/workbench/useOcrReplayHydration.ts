import type { QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { useOcrReplay } from '../../api/hooks';
import type { OcrReplayPayload } from '../../api/types';
import { applyProjectedOcrReplay } from '../../realtime/ocrReplayProjection';
import { realtimeEventFromRecord } from '../../realtime/realtimeTypes';

export function useSelectedPageReplay(args: {
  enabled: boolean;
  fileHash?: string;
  pageNo: number;
  queryClient: QueryClient;
}) {
  const replay = useOcrReplay(selectedPageReplayRequest(args));
  useReplayHydration(args.queryClient, replay.data);
}

export function selectedPageReplayRequest(args: {
  enabled: boolean;
  fileHash?: string;
  pageNo: number;
}) {
  return {
    enabled: args.enabled,
    file_hash: args.fileHash,
    limit: 10_000,
    page_no: args.pageNo,
  };
}

function useReplayHydration(queryClient: QueryClient, replay: OcrReplayPayload | undefined) {
  useEffect(() => {
    const events = (replay?.events ?? []).flatMap((record) => {
      const event = realtimeEventFromRecord(record);
      return event ? [event] : [];
    });
    applyProjectedOcrReplay(queryClient, events);
  }, [queryClient, replay]);
}
