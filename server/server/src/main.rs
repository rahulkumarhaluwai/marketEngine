use common::types::{AlertDirectionEvent, AlertTriggeredEvent, Symbol};
use graphql_api::schema::AppContext;
use graphql_api::session_store::SessionStore;
use market_data::MarketDataFeed;
use matching_engine::EngineHandle;
use postgres_store::{PostgresStore, PriceCache};
use risk_engine::{RiskConfig, RiskEngine};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use ws_server::AlertFeed;
use graphql_api::store::Store;
use graphql_api::store::CandleSource;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("starting trading platform backend");

    let market_feed = MarketDataFeed::new(1024);
    let alert_feed = AlertFeed::new(256);
    let reference_prices = Arc::new(tokio::sync::RwLock::new(HashMap::<Symbol, Decimal>::new()));

    let store = Arc::new(
        PostgresStore::connect("postgres://postgres:postgres@localhost/trading")
            .await
            .expect("connect to Postgres"),
    );
    let price_cache = PriceCache::new("redis://127.0.0.1/").expect("connect to Redis (price cache)");
    let sessions = SessionStore::new("redis://127.0.0.1/").expect("connect to Redis (sessions)");

    let mut engines: HashMap<Symbol, EngineHandle> = HashMap::new();
    engines.insert(Symbol::BtcUsd, matching_engine::spawn(Symbol::BtcUsd));
    engines.insert(Symbol::EthUsd, matching_engine::spawn(Symbol::EthUsd));

    let market_data_task = {
        let feed = market_feed.clone();
        tokio::spawn(async move {
            tracing::info!("market data service starting");
            market_data::run(feed).await;
        })
    };

    let tick_writer_task = {
        let mut rx = market_feed.subscribe();
        let store = store.clone();
        let price_cache = price_cache.clone();
        let reference_prices = reference_prices.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(tick) => {
                        reference_prices.write().await.insert(tick.symbol, tick.price);
                        let _ = price_cache.set_price(tick.symbol, tick.price).await;
                        if let Err(e) = store.record_tick(tick.symbol, tick.price, tick.timestamp).await {
                            tracing::warn!("failed to persist tick: {e}");
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("tick writer lagged, dropped {n} ticks — DB writes falling behind");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::error!("market feed closed, tick writer stopping");
                        break;
                    }
                }
            }
        })
    };

    let alert_checker_task = {
        let mut rx = market_feed.subscribe();
        let store = store.clone();
        let alert_feed = alert_feed.clone();
        tokio::spawn(async move {
            loop {
                let tick = match rx.recv().await {
                    Ok(tick) => tick,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("alert checker lagged, dropped {n} ticks");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::error!("market feed closed, alert checker stopping");
                        break;
                    }
                };

                let candidates = store.untriggered_alerts_for_symbol(tick.symbol).await;

                for alert in candidates {
                    let crossed = match alert.direction {
                        graphql_api::store::AlertDirection::Above => tick.price >= alert.target_price,
                        graphql_api::store::AlertDirection::Below => tick.price <= alert.target_price,
                    };

                    if crossed {
                        store.mark_alert_triggered(alert.id).await;

                        let direction = match alert.direction {
                            graphql_api::store::AlertDirection::Above => AlertDirectionEvent::Above,
                            graphql_api::store::AlertDirection::Below => AlertDirectionEvent::Below,
                        };

                        alert_feed.publish(AlertTriggeredEvent {
                            alert_id: alert.id,
                            user_id: alert.user_id,
                            symbol: tick.symbol,
                            target_price: alert.target_price,
                            direction,
                            price_at_trigger: tick.price,
                        });
                    }
                }
            }
        })
    };

    for symbol in Symbol::all(){
        if let Some(engine) = engines.get(&symbol) {
            market_maker::spawn(
                symbol,
                store.clone(),
                engine.clone(),
                reference_prices.clone(),
                market_maker::MarketMakerConfig::default(),
            );
        }
    }

    let ws_server_task = {
        let feed = market_feed.clone();
        let alert_feed = alert_feed.clone();
        tokio::spawn(async move {
            tracing::info!("websocket server starting");
            let addr: SocketAddr = "0.0.0.0:9001".parse().unwrap();
            ws_server::run(addr, feed, alert_feed).await;
        })
    };

    let graphql_task = {
        let engines = engines.clone();
        let reference_prices = reference_prices.clone();
        let store = store.clone();
        tokio::spawn(async move {
            tracing::info!("graphql api starting");
            let candles: Arc<dyn CandleSource> = store.clone();
            let context = AppContext {
                store,
                candles,
                engines,
                risk: RiskEngine::new(RiskConfig::default()),
                reference_prices,
                sessions,
            };
            let schema = graphql_api::build_schema(context);
            let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
            graphql_api::run(addr, schema).await;
        })
    };

    tokio::select! {
        _ = market_data_task => tracing::error!("market data task exited"),
        _ = tick_writer_task => tracing::error!("tick writer task exited"),
        _ = alert_checker_task => tracing::error!("alert checker task exited"),
        _ = ws_server_task => tracing::error!("ws server task exited"),
        _ = graphql_task => tracing::error!("graphql task exited"),
        _ = signal::ctrl_c() => tracing::info!("shutdown signal received"),
    }
}