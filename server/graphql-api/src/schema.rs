use crate::auth::{hash_password, verify_password};
use crate::session_store::SessionStore;
use crate::store::{Account, Alert, AlertDirection, Store};
use async_graphql::{Context, Enum, Object, Result, SimpleObject, ID};
use common::types::{Order, OrderStatus, OrderType, Side, Symbol, Trade};
use matching_engine::EngineHandle;
use risk_engine::RiskEngine;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use crate::store::CandleSource;

    pub struct AppContext {
    pub store: Arc<dyn Store>,
    pub candles: Arc<dyn CandleSource>,
    pub engines: HashMap<Symbol, EngineHandle>,
    pub risk: RiskEngine,
    pub reference_prices: Arc<tokio::sync::RwLock<HashMap<Symbol, Decimal>>>,
    pub sessions: SessionStore,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SymbolGql {
    BtcUsd,
    EthUsd,
    Aapl,
    Tsla,
    Googl,
    Msft,
    Amzn,
}

impl From<SymbolGql> for Symbol {
    fn from(s: SymbolGql) -> Self {
        match s {
            SymbolGql::BtcUsd => Symbol::BtcUsd,
            SymbolGql::EthUsd => Symbol::EthUsd,
            SymbolGql::Aapl => Symbol::Aapl,
            SymbolGql::Tsla => Symbol::Tsla,
            SymbolGql::Googl => Symbol::Googl,
            SymbolGql::Msft => Symbol::Msft,
            SymbolGql::Amzn => Symbol::Amzn,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SideGql {
    Buy,
    Sell,
}

impl From<SideGql> for Side {
    fn from(s: SideGql) -> Self {
        match s {
            SideGql::Buy => Side::Buy,
            SideGql::Sell => Side::Sell,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderTypeGql {
    Market,
    Limit,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum AlertDirectionGql {
    Above,
    Below,
}

impl From<AlertDirectionGql> for AlertDirection {
    fn from(d: AlertDirectionGql) -> Self {
        match d {
            AlertDirectionGql::Above => AlertDirection::Above,
            AlertDirectionGql::Below => AlertDirection::Below,
        }
    }
}

#[derive(SimpleObject)]
pub struct CandleGql {
    pub bucket_start: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
}

impl From<common::types::Candle> for CandleGql {
    fn from(c: common::types::Candle) -> Self {
        Self {
            bucket_start: c.bucket_start.to_rfc3339(),
            open: c.open.to_string(),
            high: c.high.to_string(),
            low: c.low.to_string(),
            close: c.close.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountGql {
    pub id: ID,
    pub username: String,
    pub cash_balance: String,
}

impl From<Account> for AccountGql {
    fn from(a: Account) -> Self {
        Self { id: ID(a.id.to_string()), username: a.username, cash_balance: a.cash_balance.to_string() }
    }
}

#[derive(SimpleObject)]
pub struct SessionGql {
    pub token: String,
    pub account: AccountGql,
}

#[derive(SimpleObject)]
pub struct OrderGql {
    pub id: ID,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<String>,
    pub quantity: String,
    pub filled_quantity: String,
    pub status: String,
}

impl From<Order> for OrderGql {
    fn from(o: Order) -> Self {
        Self {
            id: ID(o.id.to_string()),
            symbol: o.symbol.as_str().to_string(),
            side: format!("{:?}", o.side),
            order_type: format!("{:?}", o.order_type),
            price: o.price.map(|p| p.to_string()),
            quantity: o.quantity.to_string(),
            filled_quantity: o.filled_quantity.to_string(),
            status: format!("{:?}", o.status),
        }
    }
}

#[derive(SimpleObject)]
pub struct TradeGql {
    pub id: ID,
    pub symbol: String,
    pub price: String,
    pub quantity: String,
    pub executed_at: String,
}

impl From<Trade> for TradeGql {
    fn from(t: Trade) -> Self {
        Self {
            id: ID(t.id.to_string()),
            symbol: t.symbol.as_str().to_string(),
            price: t.price.to_string(),
            quantity: t.quantity.to_string(),
            executed_at: t.executed_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct PositionGql {
    pub symbol: String,
    pub quantity: String,
    pub avg_cost: String,
    pub market_value: String,
    pub unrealized_pnl: String,
}

#[derive(SimpleObject)]
pub struct PortfolioGql {
    pub cash_balance: String,
    pub positions: Vec<PositionGql>,
    pub total_market_value: String,
    pub total_unrealized_pnl: String,
}

#[derive(SimpleObject)]
pub struct AlertGql {
    pub id: ID,
    pub symbol: String,
    pub target_price: String,
    pub direction: String,
    pub triggered: bool,
}

impl From<Alert> for AlertGql {
    fn from(a: Alert) -> Self {
        Self {
            id: ID(a.id.to_string()),
            symbol: a.symbol.as_str().to_string(),
            target_price: a.target_price.to_string(),
            direction: format!("{:?}", a.direction),
            triggered: a.triggered,
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn candles(&self, ctx: &Context<'_>, symbol: SymbolGql, limit: i32) -> Result<Vec<CandleGql>> {
        let app = ctx.data::<AppContext>()?;
        let symbol: Symbol = symbol.into();
        Ok(app.candles.get_candles(symbol, limit as i64).await.into_iter().map(Into::into).collect())
    }
    async fn me(&self, ctx: &Context<'_>, token: String) -> Result<Option<AccountGql>> {
        let app = ctx.data::<AppContext>()?;
        let Some(user_id) = app.sessions.resolve(&token).await? else {
            return Ok(None);
        };
        Ok(app.store.get_account(user_id).await.map(Into::into))
    }

    async fn account(&self, ctx: &Context<'_>, user_id: ID) -> Result<Option<AccountGql>> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;
        Ok(app.store.get_account(id).await.map(Into::into))
    }

    async fn portfolio(&self, ctx: &Context<'_>, user_id: ID) -> Result<PortfolioGql> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;

        let account = app.store.get_account(id).await.ok_or("account not found")?;
        let holdings = app.store.holdings(id).await;
        let prices = app.reference_prices.read().await;

        let mut positions = Vec::new();
        let mut total_market_value = Decimal::ZERO;
        let mut total_unrealized_pnl = Decimal::ZERO;

        for (symbol, quantity) in holdings {
            if quantity <= Decimal::ZERO {
                continue;
            }
            let price = prices.get(&symbol).copied().unwrap_or(Decimal::ZERO);
            let market_value = price * quantity;
            let avg_cost = Decimal::ZERO;
            let unrealized_pnl = market_value - (avg_cost * quantity);

            total_market_value += market_value;
            total_unrealized_pnl += unrealized_pnl;

            positions.push(PositionGql {
                symbol: symbol.as_str().to_string(),
                quantity: quantity.to_string(),
                avg_cost: avg_cost.to_string(),
                market_value: market_value.to_string(),
                unrealized_pnl: unrealized_pnl.to_string(),
            });
        }

        Ok(PortfolioGql {
            cash_balance: account.cash_balance.to_string(),
            positions,
            total_market_value: total_market_value.to_string(),
            total_unrealized_pnl: total_unrealized_pnl.to_string(),
        })
    }

    async fn order_history(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<OrderGql>> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;
        Ok(app.store.orders_for_user(id).await.into_iter().map(Into::into).collect())
    }

    async fn trade_history(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<TradeGql>> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;
        Ok(app.store.trades_for_user(id).await.into_iter().map(Into::into).collect())
    }

    async fn alerts(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<AlertGql>> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;
        Ok(app.store.alerts_for_user(id).await.into_iter().map(Into::into).collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(&self, ctx: &Context<'_>, username: String, password: String) -> Result<SessionGql> {
        let app = ctx.data::<AppContext>()?;

        if app.store.get_account_by_username(&username).await.is_some() {
            return Err("username already taken".into());
        }

        let hash = hash_password(&password);
        let account = app.store.create_account(username, hash).await;
        let token = app.sessions.create_session(account.id).await?;

        Ok(SessionGql { token, account: account.into() })
    }

    async fn login(&self, ctx: &Context<'_>, username: String, password: String) -> Result<SessionGql> {
        let app = ctx.data::<AppContext>()?;

        let account = app.store.get_account_by_username(&username).await.ok_or("invalid username or password")?;

        if !verify_password(&password, &account.password_hash) {
            return Err("invalid username or password".into());
        }

        let token = app.sessions.create_session(account.id).await?;
        Ok(SessionGql { token, account: account.into() })
    }

    async fn logout(&self, ctx: &Context<'_>, token: String) -> Result<bool> {
        let app = ctx.data::<AppContext>()?;
        app.sessions.destroy(&token).await?;
        Ok(true)
    }

    async fn deposit(&self, ctx: &Context<'_>, user_id: ID, amount: String) -> Result<AccountGql> {
        let app = ctx.data::<AppContext>()?;
        let id = Uuid::from_str(&user_id)?;
        let amount = Decimal::from_str(&amount)?;
        if amount <= Decimal::ZERO {
            return Err("deposit amount must be positive".into());
        }
        let account = app.store.deposit(id, amount).await.ok_or("account not found")?;
        Ok(account.into())
    }

    async fn place_order(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        symbol: SymbolGql,
        side: SideGql,
        order_type: OrderTypeGql,
        quantity: String,
        price: Option<String>,
    ) -> Result<OrderGql> {
        let app = ctx.data::<AppContext>()?;
        let user_id = Uuid::from_str(&user_id)?;
        let symbol: Symbol = symbol.into();
        let side: Side = side.into();
        let quantity = Decimal::from_str(&quantity)?;
        let price = price.map(|p| Decimal::from_str(&p)).transpose()?;

        let order_type = match order_type {
            OrderTypeGql::Market => OrderType::Market,
            OrderTypeGql::Limit => {
                if price.is_none() {
                    return Err("limit orders require a price".into());
                }
                OrderType::Limit
            }
        };

        let snapshot = app.store.account_snapshot(user_id).await.ok_or("account not found")?;
        let reference_price = app.reference_prices.read().await.get(&symbol).copied().unwrap_or(Decimal::ZERO);

        let order = Order {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::Open,
            created_at: chrono::Utc::now(),
        };

        app.risk.validate(&order, &snapshot, reference_price).map_err(|e| format!("{:?}", e))?;

        app.store.save_order(order.clone()).await;

        let engine = app.engines.get(&symbol).ok_or("no engine for symbol")?;
        let result = engine.submit(order.clone()).await;

        let mut updated = order.clone();
        updated.filled_quantity = order.quantity - result.remaining_quantity;
        updated.status = result.order_status;
        app.store.update_order(updated.clone()).await;

        if !result.trades.is_empty() {
            app.store.save_trades(&result.trades).await;
        }

        Ok(updated.into())
    }

    async fn cancel_order(
        &self,
        ctx: &Context<'_>,
        order_id: ID,
        symbol: SymbolGql,
        side: SideGql,
        price: String,
    ) -> Result<bool> {
        let app = ctx.data::<AppContext>()?;
        let order_id = Uuid::from_str(&order_id)?;
        let symbol: Symbol = symbol.into();
        let side: Side = side.into();
        let price = Decimal::from_str(&price)?;

        let engine = app.engines.get(&symbol).ok_or("no engine for symbol")?;
        Ok(engine.cancel(order_id, side, price).await)
    }

    async fn create_alert(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        symbol: SymbolGql,
        target_price: String,
        direction: AlertDirectionGql,
    ) -> Result<AlertGql> {
        let app = ctx.data::<AppContext>()?;
        let user_id = Uuid::from_str(&user_id)?;
        let symbol: Symbol = symbol.into();
        let target_price = Decimal::from_str(&target_price)?;

        Ok(app.store.create_alert(user_id, symbol, target_price, direction.into()).await.into())
    }
}

pub type AppSchema = async_graphql::Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;