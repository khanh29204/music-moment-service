pub mod moments;
pub mod stream;

use crate::{middleware::auth::require_api_key, state::AppState};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route("/songs/moment", post(moments::save))
        .route("/songs/moment/:moment_id", get(moments::get))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));
    Router::new()
        .route("/songs/stream", get(stream::stream))
        .merge(protected)
        .with_state(state)
}
