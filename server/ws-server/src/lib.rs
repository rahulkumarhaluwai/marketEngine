use common::types::{Symbol, Tick};
use futures::{SinkExt, StreamExt};
use market_data::MarketDataFeed;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use common::types::AlertTriggeredEvent;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
}

#[derive(Clone)]
pub struct AlertFeed {
    sender: tokio::sync::broadcast::Sender<AlertTriggeredEvent>,
}

impl AlertFeed {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AlertTriggeredEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: AlertTriggeredEvent) {
        let _ = self.sender.send(event);
    }
}

#[derive(Debug, Serialize)]
struct AlertMessage {
    channel: String,
    alert_id: String,
    symbol: &'static str,
    target_price: String,
    price_at_trigger: String,
    direction: String,
}

fn alert_channel_for(user_id: uuid::Uuid) -> String {
    format!("alerts:{}", user_id)
}

#[derive(Debug, Serialize)]
struct TickMessage {
    channel: String,
    symbol: &'static str,
    price: String,
    timestamp: String,
}

fn channel_for(symbol: Symbol) -> String {
    symbol.channel()
}

fn to_client_message(tick: &Tick) -> TickMessage {
    TickMessage {
        channel: channel_for(tick.symbol),
        symbol: tick.symbol.as_str(),
        price: tick.price.to_string(),
        timestamp: tick.timestamp.to_rfc3339(),
    }
}

pub async fn run(addr: SocketAddr, feed: MarketDataFeed, alert_feed: AlertFeed) {
    let listener = TcpListener::bind(addr).await.expect("bind ws server");
    tracing::info!("ws server listening on {addr}");

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("accept error: {e}");
                continue;
            }
        };
        let feed = feed.clone();
        let alert_feed = alert_feed.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer, feed, alert_feed).await {
                tracing::warn!("connection {peer} closed: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    feed: MarketDataFeed,
    alert_feed: AlertFeed,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut subscriptions: HashSet<String> = HashSet::new();
    let mut tick_rx = feed.subscribe();
    let mut alert_rx = alert_feed.subscribe();

    tracing::info!("client connected: {peer}");

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::Subscribe { channel } => { subscriptions.insert(channel); }
                                ClientMessage::Unsubscribe { channel } => { subscriptions.remove(&channel); }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { tracing::warn!("read error from {peer}: {e}"); break; }
                    _ => {}
                }
            }

            tick = tick_rx.recv() => {
                match tick {
                    Ok(tick) => {
                        let channel = channel_for(tick.symbol);
                        if subscriptions.contains(&channel) {
                            let payload = to_client_message(&tick);
                            write.send(Message::Text(serde_json::to_string(&payload)?)).await?;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("client {peer} lagged, skipped {n} ticks");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }

            alert_event = alert_rx.recv() => {
                match alert_event {
                    Ok(event) => {
                        let channel = alert_channel_for(event.user_id);
                        if subscriptions.contains(&channel) {
                            let payload = AlertMessage {
                                channel,
                                alert_id: event.alert_id.to_string(),
                                symbol: event.symbol.as_str(),
                                target_price: event.target_price.to_string(),
                                price_at_trigger: event.price_at_trigger.to_string(),
                                direction: format!("{:?}", event.direction),
                            };
                            write.send(Message::Text(serde_json::to_string(&payload)?)).await?;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    tracing::info!("client disconnected: {peer}");
    Ok(())
}