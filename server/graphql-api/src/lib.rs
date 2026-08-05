pub mod auth;
pub mod billing_routes;
pub mod leaderboard;
pub mod memory_store;
pub mod rate_limiter;
pub mod schema;
pub mod session_store;
pub mod store;

use async_graphql::Schema;
use async_graphql_axum::GraphQL;
use axum::{routing::get, Router};
use schema::{AppContext, AppSchema, MutationRoot, QueryRoot};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub fn build_schema(context: AppContext) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(context)
        .finish()
}

pub async fn run(addr: SocketAddr, schema: AppSchema, store: std::sync::Arc<dyn store::Store>, stripe_webhook_secret: String) {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post_service(GraphQL::new(schema)))
        .merge(billing_routes::billing_router(store, stripe_webhook_secret))
        .layer(cors);

    tracing::info!("graphql api listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind graphql server");
    axum::serve(listener, app).await.expect("graphql server error");
}

async fn graphiql() -> axum::response::Html<String> {
    axum::response::Html(async_graphql::http::GraphiQLSource::build().endpoint("/graphql").finish())
}