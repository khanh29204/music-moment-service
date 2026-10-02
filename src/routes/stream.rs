use crate::{error::AppError, state::AppState, utils::audio_range::calculate_audio_byte_range};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderValue, StatusCode},
    response::Response,
};
use futures_util::TryStreamExt;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct StreamQuery {
    pub url: Option<String>,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

pub async fn stream(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StreamQuery>,
) -> Result<Response, AppError> {
    let url = query
        .url
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| AppError::BadRequest("Thiếu tham số url để stream".into()))?;
    let start = query.start.filter(|v| v.is_finite()).unwrap_or(0.0);
    let end = query.end.filter(|v| v.is_finite()).ok_or_else(|| {
        AppError::BadRequest("start và end phải là ms hợp lệ (end > start)".into())
    })?;
    if end <= start {
        return Err(AppError::BadRequest(
            "start và end phải là ms hợp lệ (end > start)".into(),
        ));
    }
    let range = calculate_audio_byte_range(start, end);
    let upstream = state
        .http_client
        .get(url)
        .header(
            "Range",
            format!("bytes={}-{}", range.start_byte, range.end_byte),
        )
        .send()
        .await
        .map_err(|_| AppError::BadGateway)?;
    if !matches!(
        upstream.status(),
        reqwest::StatusCode::OK | reqwest::StatusCode::PARTIAL_CONTENT
    ) {
        return Err(AppError::BadGateway);
    }
    let body = Body::from_stream(
        upstream
            .bytes_stream()
            .map_err(|e| std::io::Error::other(e)),
    );
    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert("Content-Type", HeaderValue::from_static("audio/mpeg"));
    headers.insert(
        "Content-Length",
        HeaderValue::from_str(&range.content_length.to_string())
            .map_err(|_| AppError::Internal("invalid content length".into()))?,
    );
    headers.insert("Accept-Ranges", HeaderValue::from_static("bytes"));
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    Ok(response)
}
