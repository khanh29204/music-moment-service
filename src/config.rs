use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub mongodb_uri: String,
    pub mongodb_db: String,
    pub redis_url: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .map_err(|_| "PORT must be a number")?,
            mongodb_uri: required("MONGODB_URI")?,
            mongodb_db: required("MONGODB_DB")?,
            redis_url: required("REDIS_URL")?,
            api_key: env::var("API_KEY").ok().filter(|v| !v.trim().is_empty()),
        })
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("{name} is required"))
}
