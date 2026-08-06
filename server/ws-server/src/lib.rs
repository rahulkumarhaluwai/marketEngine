use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use common::types::{AlertTriggeredEvent, Symbol, Tick};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
}

#[derive(Debug, Serialize)]
struct TickMessage {
    channel: String,
    symbol: &'static str,
    price: String,
    timestamp: String,
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

fn channel_for(symbol: Symbol) -> String {
    symbol.channel()
}

fn alert_channel_for(user_id: Uuid) -> String {
    format!("alerts:{}", user_id)
}

fn to_client_message(tick: &Tick) -> TickMessage {
    TickMessage {
        channel: channel_for(tick.symbol),
        symbol: tick.symbol.as_str(),
        price: tick.price.to_string(),
        timestamp: tick.timestamp.to_rfc3339(),
    }
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

#[derive(Clone)]
struct WsState {
    feed: market_data::MarketDataFeed,
    alert_feed: AlertFeed,
}

/// Builds a `/ws` route that can be `.merge()`d into an existing Axum
/// Router — shares the same port as GraphQL/HTTP.
pub fn ws_router(feed: market_data::MarketDataFeed, alert_feed: AlertFeed) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(WsState { feed, alert_feed })
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.feed, state.alert_feed))
}

async fn handle_socket(socket: WebSocket, feed: market_data::MarketDataFeed, alert_feed: AlertFeed) {
    let (mut write, mut read) = socket.split();
    let mut subscriptions: HashSet<String> = HashSet::new();
    let mut tick_rx = feed.subscribe();
    let mut alert_rx = alert_feed.subscribe();

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
                    Some(Err(e)) => {
                        tracing::warn!("ws read error: {e}");
                        break;
                    }
                    _ => {}
                }
            }

            tick = tick_rx.recv() => {
                match tick {
                    Ok(tick) => {
                        let channel = channel_for(tick.symbol);
                        if subscriptions.contains(&channel) {
                            let payload = to_client_message(&tick);
                            if let Ok(json) = serde_json::to_string(&payload) {
                                if write.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("ws client lagged, skipped {n} ticks");
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
                            if let Ok(json) = serde_json::to_string(&payload) {
                                if write.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}