use crate::{error::AppError, state::AppState};
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn require_api_key(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Không cấu hình API_KEY → bỏ qua auth (chỉ dùng ở môi trường dev)
    let authorized = match &state.api_key {
        None => true,
        Some(key) => {
            request.headers().get("x-api-key").and_then(|v| v.to_str().ok()) == Some(key.as_str())
        }
    };
    if !authorized {
        return Err(AppError::Unauthorized);
    }
    Ok(next.run(request).await)
}
