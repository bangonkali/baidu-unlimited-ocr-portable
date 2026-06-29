import type { RealtimeConnectionState } from './realtimeStore';
import type { RealtimeEvent } from './realtimeTypes';
import { parseRealtimeEvent } from './realtimeTypes';

interface RealtimeClientOptions {
  onEvent: (event: RealtimeEvent) => void;
  onStateChange: (state: RealtimeConnectionState, error?: string) => void;
  path?: string;
}

function websocketUrl(path: string) {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}${path}`;
}

export class RealtimeClient {
  private reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  private retryCount = 0;
  private socket: WebSocket | undefined;
  private stopped = false;

  constructor(private readonly options: RealtimeClientOptions) {}

  connect() {
    this.stopped = false;
    this.clearReconnect();
    this.options.onStateChange('connecting');
    const socket = new WebSocket(websocketUrl(this.options.path ?? '/api/events'));
    this.socket = socket;

    socket.onopen = () => {
      this.retryCount = 0;
      this.options.onStateChange('connected');
    };
    socket.onmessage = (message) => {
      if (typeof message.data !== 'string') {
        return;
      }
      const event = parseRealtimeEvent(message.data);
      if (event) {
        this.options.onEvent(event);
      }
    };
    socket.onerror = () => {
      this.options.onStateChange('disconnected', 'websocket error');
    };
    socket.onclose = () => {
      this.socket = undefined;
      if (this.stopped) {
        this.options.onStateChange('disconnected');
        return;
      }
      this.options.onStateChange('connecting');
      this.scheduleReconnect();
    };
  }

  close() {
    this.stopped = true;
    this.clearReconnect();
    this.socket?.close();
    this.socket = undefined;
  }

  private scheduleReconnect() {
    this.clearReconnect();
    const delay = Math.min(1000 * 2 ** this.retryCount, 10000);
    this.retryCount += 1;
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }

  private clearReconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }
  }
}
