use deadpool_redis::Pool;
use mongodb::Database;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis_pool: Pool,
    pub http_client: Client,
    pub api_key: Option<String>,
}
