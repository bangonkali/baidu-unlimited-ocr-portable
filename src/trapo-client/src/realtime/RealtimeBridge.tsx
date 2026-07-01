import type { QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { RealtimeClient } from './realtimeClient';
import { applyRealtimeEventToQueryClient } from './realtimeQueryBridge';
import { markRealtimeEvent, setRealtimeConnectionState } from './realtimeStore';

interface RealtimeBridgeProps {
  queryClient: QueryClient;
}

export function RealtimeBridge({ queryClient }: RealtimeBridgeProps) {
  useEffect(() => {
    const client = new RealtimeClient({
      onEvent: (event) => {
        markRealtimeEvent(event);
        applyRealtimeEventToQueryClient(queryClient, event);
      },
      onStateChange: setRealtimeConnectionState,
    });
    client.connect();
    return () => client.close();
  }, [queryClient]);

  return null;
}
