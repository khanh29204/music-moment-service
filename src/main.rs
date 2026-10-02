mod config;
mod error;
mod middleware;
mod models;
mod routes;
mod state;
mod utils;

use axum::Router;
use config::Config;
use mongodb::{Client, IndexModel, bson::doc, options::IndexOptions};
use state::AppState;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Lấy RUST_LOG nếu có, luôn đè thêm các directive chặn noise driver
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
            "{filter},rustls=warn,mongodb=warn,hyper=warn"
        )))
        .init();
    let config = Config::from_env()?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!(addr = %listener.local_addr()?, "music-moment-service listening");
    let mongo = Client::with_uri_str(&config.mongodb_uri).await?;
    let db = mongo.database(&config.mongodb_db);
    // Index creation chạy background, không chặn serve (Atlas chết cũng không treo app)
    let db_index = db.clone();
    tokio::spawn(async move {
        if let Err(e) = db_index
            .collection::<models::moment::MomentClip>("moment_clips")
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"momentId": 1})
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await
        {
            tracing::warn!(error = %e, "create_index failed, mongo unavailable");
        }
    });
    let redis_pool = deadpool_redis::Config::from_url(config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let state = Arc::new(AppState {
        db,
        redis_pool,
        http_client: reqwest::Client::new(),
        api_key: config.api_key,
    });
    let app: Router = routes::router(state).layer(
        TraceLayer::new_for_http()
            .make_span_with(|req: &axum::extract::Request| {
                tracing::info_span!(
                    "http",
                    method = req.method().as_str(),
                    path = req.uri().path(),
                )
            })
            .on_response(
                |res: &axum::response::Response, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(status = res.status().as_u16(), latency_ms = latency.as_millis() as u64, "request done");
                },
            ),
    );
    axum::serve(listener, app).await?;
    Ok(())
}
