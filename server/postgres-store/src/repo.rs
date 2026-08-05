use async_trait::async_trait;
use chrono::Utc;
use common::types::{Order, OrderStatus, OrderType, Side, Symbol, Trade};
use graphql_api::store::{Account, Alert, AlertDirection, Store};
use risk_engine::AccountSnapshot;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;
use common::types::Candle;
use graphql_api::store::CandleSource;

pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    Ok(Self { pool })
}

    pub async fn record_tick(&self, symbol: Symbol, price: Decimal, ts: chrono::DateTime<Utc>) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO market_ticks (symbol, ts, price) VALUES ($1, $2, $3)")
            .bind(symbol.as_str())
            .bind(ts)
            .bind(price)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn record_order_event(&self, order_id: Uuid, event_type: &str, detail: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO order_events (event_id, order_id, event_type, detail, event_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(order_id)
        .bind(event_type)
        .bind(detail)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    fn parse_symbol(s: &str) -> Symbol {
        match s {
            "BTC-USD" => Symbol::BtcUsd,
            "ETH-USD" => Symbol::EthUsd,
            other => panic!("unknown symbol in store: {other}"),
        }
    }

    fn parse_side(s: &str) -> Side {
        match s {
            "Buy" => Side::Buy,
            "Sell" => Side::Sell,
            other => panic!("unknown side in store: {other}"),
        }
    }

    fn parse_order_type(s: &str) -> OrderType {
        match s {
            "Market" => OrderType::Market,
            "Limit" => OrderType::Limit,
            other => panic!("unknown order type in store: {other}"),
        }
    }

    fn parse_status(s: &str) -> OrderStatus {
        match s {
            "Open" => OrderStatus::Open,
            "PartiallyFilled" => OrderStatus::PartiallyFilled,
            "Filled" => OrderStatus::Filled,
            "Cancelled" => OrderStatus::Cancelled,
            other => panic!("unknown status in store: {other}"),
        }
    }
}

#[async_trait]
impl Store for PostgresStore {
    async fn create_account(&self, username: String, password_hash: String) -> Account {
        let account = Account { id: Uuid::new_v4(), username, cash_balance: Decimal::ZERO, password_hash };
        sqlx::query(
            "INSERT INTO accounts (user_id, username, cash_balance, password_hash) VALUES ($1, $2, $3, $4)",
        )
        .bind(account.id)
        .bind(&account.username)
        .bind(account.cash_balance)
        .bind(&account.password_hash)
        .execute(&self.pool)
        .await
        .expect("insert account");
        account
    }

    async fn all_user_ids(&self) -> Vec<Uuid> {
        let Ok(rows) = sqlx::query("SELECT user_id FROM accounts").fetch_all(&self.pool).await else {
            return vec![];
        };
        rows.into_iter().map(|row| row.get("user_id")).collect()
    }

    async fn get_account(&self, user_id: Uuid) -> Option<Account> {
        let row = sqlx::query(
            "SELECT user_id, username, cash_balance, password_hash FROM accounts WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(Account {
            id: row.get("user_id"),
            username: row.get("username"),
            cash_balance: row.get("cash_balance"),
            password_hash: row.get("password_hash"),
        })
    }

    async fn get_account_by_username(&self, username: &str) -> Option<Account> {
        let row = sqlx::query(
            "SELECT user_id, username, cash_balance, password_hash FROM accounts WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(Account {
            id: row.get("user_id"),
            username: row.get("username"),
            cash_balance: row.get("cash_balance"),
            password_hash: row.get("password_hash"),
        })
    }

    async fn untriggered_alerts_for_symbol(&self, symbol: Symbol) -> Vec<Alert> {
        let Ok(rows) = sqlx::query(
            "SELECT alert_id, user_id, target_price, direction, triggered
             FROM alerts WHERE symbol = $1 AND triggered = false",
        )
        .bind(symbol.as_str())
        .fetch_all(&self.pool)
        .await
        else {
            return vec![];
        };

        rows.into_iter()
            .map(|row| {
                let direction: String = row.get("direction");
                Alert {
                    id: row.get("alert_id"),
                    user_id: row.get("user_id"),
                    symbol,
                    target_price: row.get("target_price"),
                    direction: if direction == "Above" { AlertDirection::Above } else { AlertDirection::Below },
                    triggered: row.get("triggered"),
                }
            })
            .collect()
    }

    async fn mark_alert_triggered(&self, alert_id: Uuid) {
        let _ = sqlx::query("UPDATE alerts SET triggered = true WHERE alert_id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await;
    }

    async fn deposit(&self, user_id: Uuid, amount: Decimal) -> Option<Account> {
        sqlx::query("UPDATE accounts SET cash_balance = cash_balance + $1 WHERE user_id = $2")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .ok()?;
        self.get_account(user_id).await
    }

    async fn account_snapshot(&self, user_id: Uuid) -> Option<AccountSnapshot> {
        let account = self.get_account(user_id).await?;
        let holdings: HashMap<String, Decimal> = self
            .holdings(user_id)
            .await
            .into_iter()
            .map(|(s, q)| (s.as_str().to_string(), q))
            .collect();

        let open_orders = self.orders_for_user(user_id).await;
        let open_orders: Vec<&Order> = open_orders
            .iter()
            .filter(|o| matches!(o.status, OrderStatus::Open | OrderStatus::PartiallyFilled))
            .collect();

        let reserved_cash = open_orders
            .iter()
            .filter(|o| o.side == Side::Buy)
            .map(|o| o.price.unwrap_or(Decimal::ZERO) * (o.quantity - o.filled_quantity))
            .sum();

        let mut reserved_assets: HashMap<String, Decimal> = HashMap::new();
        for o in open_orders.iter().filter(|o| o.side == Side::Sell) {
            *reserved_assets.entry(o.symbol.as_str().to_string()).or_insert(Decimal::ZERO) +=
                o.quantity - o.filled_quantity;
        }

        Some(AccountSnapshot {
            cash_balance: account.cash_balance,
            holdings,
            reserved_cash,
            reserved_assets,
            open_order_count: open_orders.len(),
        })
    }

    async fn holdings(&self, user_id: Uuid) -> Vec<(Symbol, Decimal)> {
        let Ok(rows) = sqlx::query("SELECT symbol, quantity FROM holdings WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        else {
            return vec![];
        };

        rows.into_iter()
            .map(|row| {
                let symbol: String = row.get("symbol");
                let quantity: Decimal = row.get("quantity");
                (Self::parse_symbol(&symbol), quantity)
            })
            .collect()
    }

    async fn save_order(&self, order: Order) {
        let _ = sqlx::query(
            "INSERT INTO orders (order_id, user_id, symbol, side, order_type, price, quantity, filled_quantity, status, created_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(order.id)
        .bind(order.user_id)
        .bind(order.symbol.as_str())
        .bind(format!("{:?}", order.side))
        .bind(format!("{:?}", order.order_type))
        .bind(order.price)
        .bind(order.quantity)
        .bind(order.filled_quantity)
        .bind(format!("{:?}", order.status))
        .bind(order.created_at)
        .execute(&self.pool)
        .await;

        let _ = self.record_order_event(order.id, "created", "order submitted").await;
    }

    async fn update_order(&self, order: Order) {
        let _ = sqlx::query("UPDATE orders SET filled_quantity = $1, status = $2 WHERE order_id = $3")
            .bind(order.filled_quantity)
            .bind(format!("{:?}", order.status))
            .bind(order.id)
            .execute(&self.pool)
            .await;

        let _ = self.record_order_event(order.id, "status_update", &format!("{:?}", order.status)).await;
    }

    async fn orders_for_user(&self, user_id: Uuid) -> Vec<Order> {
        let Ok(rows) = sqlx::query(
            "SELECT order_id, symbol, side, order_type, price, quantity, filled_quantity, status, created_at
             FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        else {
            return vec![];
        };

        rows.into_iter()
            .map(|row| {
                let symbol: String = row.get("symbol");
                let side: String = row.get("side");
                let order_type: String = row.get("order_type");
                let status: String = row.get("status");

                Order {
                    id: row.get("order_id"),
                    user_id,
                    symbol: Self::parse_symbol(&symbol),
                    side: Self::parse_side(&side),
                    order_type: Self::parse_order_type(&order_type),
                    price: row.get("price"),
                    quantity: row.get("quantity"),
                    filled_quantity: row.get("filled_quantity"),
                    status: Self::parse_status(&status),
                    created_at: row.get("created_at"),
                }
            })
            .collect()
    }

    async fn save_trades(&self, trades: &[Trade]) {
        for trade in trades {
            let _ = sqlx::query(
                "INSERT INTO trades (trade_id, symbol, buy_order_id, sell_order_id, price, quantity, executed_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(trade.id)
            .bind(trade.symbol.as_str())
            .bind(trade.buy_order_id)
            .bind(trade.sell_order_id)
            .bind(trade.price)
            .bind(trade.quantity)
            .bind(trade.executed_at)
            .execute(&self.pool)
            .await;

            for (order_id, side) in [(trade.buy_order_id, Side::Buy), (trade.sell_order_id, Side::Sell)] {
                let Ok(Some(row)) = sqlx::query("SELECT user_id FROM orders WHERE order_id = $1")
                    .bind(order_id)
                    .fetch_optional(&self.pool)
                    .await
                else {
                    continue;
                };
                let user_id: Uuid = row.get("user_id");

                let qty_delta = if side == Side::Buy { trade.quantity } else { -trade.quantity };
                let _ = sqlx::query(
                    "INSERT INTO holdings (user_id, symbol, quantity) VALUES ($1, $2, $3)
                     ON CONFLICT (user_id, symbol) DO UPDATE SET quantity = holdings.quantity + $3",
                )
                .bind(user_id)
                .bind(trade.symbol.as_str())
                .bind(qty_delta)
                .execute(&self.pool)
                .await;

                let cash_delta = if side == Side::Buy {
                    -(trade.price * trade.quantity)
                } else {
                    trade.price * trade.quantity
                };
                let _ = sqlx::query("UPDATE accounts SET cash_balance = cash_balance + $1 WHERE user_id = $2")
                    .bind(cash_delta)
                    .bind(user_id)
                    .execute(&self.pool)
                    .await;
            }
        }
    }

    async fn trades_for_user(&self, user_id: Uuid) -> Vec<Trade> {
        let Ok(rows) = sqlx::query(
            "SELECT t.trade_id, t.symbol, t.buy_order_id, t.sell_order_id, t.price, t.quantity, t.executed_at
             FROM trades t
             JOIN orders o ON o.order_id = t.buy_order_id OR o.order_id = t.sell_order_id
             WHERE o.user_id = $1
             ORDER BY t.executed_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        else {
            return vec![];
        };

        rows.into_iter()
            .map(|row| {
                let symbol: String = row.get("symbol");
                Trade {
                    id: row.get("trade_id"),
                    symbol: Self::parse_symbol(&symbol),
                    buy_order_id: row.get("buy_order_id"),
                    sell_order_id: row.get("sell_order_id"),
                    price: row.get("price"),
                    quantity: row.get("quantity"),
                    executed_at: row.get("executed_at"),
                }
            })
            .collect()
    }

    async fn create_alert(
        &self,
        user_id: Uuid,
        symbol: Symbol,
        target_price: Decimal,
        direction: AlertDirection,
    ) -> Alert {
        let alert = Alert { id: Uuid::new_v4(), user_id, symbol, target_price, direction, triggered: false };
        let _ = sqlx::query(
            "INSERT INTO alerts (alert_id, user_id, symbol, target_price, direction, triggered) VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(alert.id)
        .bind(user_id)
        .bind(symbol.as_str())
        .bind(target_price)
        .bind(format!("{:?}", direction))
        .bind(false)
        .execute(&self.pool)
        .await;
        alert
    }

    async fn alerts_for_user(&self, user_id: Uuid) -> Vec<Alert> {
        let Ok(rows) = sqlx::query(
            "SELECT alert_id, symbol, target_price, direction, triggered FROM alerts WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        else {
            return vec![];
        };

        rows.into_iter()
            .map(|row| {
                let symbol: String = row.get("symbol");
                let direction: String = row.get("direction");
                Alert {
                    id: row.get("alert_id"),
                    user_id,
                    symbol: Self::parse_symbol(&symbol),
                    target_price: row.get("target_price"),
                    direction: if direction == "Above" { AlertDirection::Above } else { AlertDirection::Below },
                    triggered: row.get("triggered"),
                }
            })
            .collect()
    }
}

#[async_trait]
impl CandleSource for PostgresStore {
    async fn get_candles(&self, symbol: Symbol, limit: i64) -> Vec<Candle> {
        let Ok(rows) = sqlx::query(
            "SELECT bucket, open, high, low, close FROM candles_1m
             WHERE symbol = $1 ORDER BY bucket DESC LIMIT $2",
        )
        .bind(symbol.as_str())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        else {
            return vec![];
        };

        let mut candles: Vec<Candle> = rows
            .into_iter()
            .map(|row| Candle {
                symbol,
                bucket_start: row.get("bucket"),
                open: row.get("open"),
                high: row.get("high"),
                low: row.get("low"),
                close: row.get("close"),
            })
            .collect();

        candles.reverse(); // chronological order for charting
        candles
    }
}