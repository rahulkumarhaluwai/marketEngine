type TickMessage = {
  channel: string;
  symbol: string;
  price: string;
  timestamp: string;
};

type TickHandler = (tick: TickMessage) => void;

export class MarketWsClient {
  private socket: WebSocket | null = null;
  private handlers = new Map<string, Set<TickHandler>>();
  private pendingSubscriptions = new Set<string>();
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  connect() {
    if (this.socket) return;

    this.socket = new WebSocket(this.url);

    this.socket.onopen = () => {
      for (const channel of this.pendingSubscriptions) {
        this.sendSubscribe(channel);
      }
    };

    this.socket.onmessage = (event) => {
      const data: TickMessage = JSON.parse(event.data);
      const subs = this.handlers.get(data.channel);
      subs?.forEach((handler) => handler(data));
    };

    this.socket.onclose = () => {
      this.socket = null;
      // simple reconnect after a short delay
      setTimeout(() => this.connect(), 1000);
    };
  }

  private sendSubscribe(channel: string) {
    this.socket?.send(JSON.stringify({ type: "subscribe", channel }));
  }

  subscribe(channel: string, handler: TickHandler) {
    if (!this.handlers.has(channel)) {
      this.handlers.set(channel, new Set());
    }
    this.handlers.get(channel)!.add(handler);

    if (this.socket?.readyState === WebSocket.OPEN) {
      this.sendSubscribe(channel);
    } else {
      this.pendingSubscriptions.add(channel);
    }

    return () => this.unsubscribe(channel, handler);
  }

  unsubscribe(channel: string, handler: TickHandler) {
    this.handlers.get(channel)?.delete(handler);
  }
}

let singleton: MarketWsClient | null = null;

export function getWsClient(): MarketWsClient {
  if (!singleton) {
    singleton = new MarketWsClient(process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:9001");
    singleton.connect();
  }
  return singleton;
}