import type { QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { useOcrReplay } from '../../api/hooks';
import type { OcrReplayPayload } from '../../api/types';
import { applyRealtimeEventToQueryClient } from '../../realtime/realtimeQueryBridge';
import { realtimeEventFromRecord } from '../../realtime/realtimeTypes';

export function useSelectedPageReplay(args: {
  enabled: boolean;
  fileHash?: string;
  pageNo: number;
  queryClient: QueryClient;
}) {
  const replay = useOcrReplay({
    enabled: args.enabled,
    file_hash: args.fileHash,
    limit: 10_000,
    page_no: args.pageNo,
  });
  useReplayHydration(args.queryClient, replay.data);
}

function useReplayHydration(queryClient: QueryClient, replay: OcrReplayPayload | undefined) {
  useEffect(() => {
    for (const record of replay?.events ?? []) {
      const event = realtimeEventFromRecord(record);
      if (event) {
        applyRealtimeEventToQueryClient(queryClient, event);
      }
    }
  }, [queryClient, replay]);
}
