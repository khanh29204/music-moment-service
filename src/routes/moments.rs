use crate::{
    error::AppError,
    models::moment::{MomentClip, SaveMomentRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use deadpool_redis::redis::AsyncCommands;
use mongodb::bson::doc;
use serde_json::json;
use std::sync::Arc;

const NULL: &str = "__NULL__";

pub async fn save(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveMomentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if body.moment_id.trim().is_empty() {
        return Err(AppError::BadRequest("momentId không được để trống".into()));
    }
    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url không được để trống".into()));
    }
    let collection = state.db.collection::<MomentClip>("moment_clips");
    let result = collection.update_one(doc! { "momentId": &body.moment_id }, doc! {
        "$set": { "url": &body.url }, "$setOnInsert": { "createdAt": mongodb::bson::DateTime::now() }
    }).upsert(true).await?;
    let clip = collection
        .find_one(doc! { "momentId": &body.moment_id })
        .await?
        .ok_or_else(|| AppError::Internal("saved moment not found".into()))?;
    let mut conn = state
        .redis_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _: () = conn
        .set_ex(format!("moment:{}", body.moment_id), &clip.url, 10)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((
        if result.upserted_id.is_some() {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(json!({"momentId": clip.moment_id, "url": clip.url, "createdAt": clip.created_at})),
    ))
}

pub async fn get(
    Path(moment_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let key = format!("moment:{moment_id}");
    let mut conn = state
        .redis_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let cached: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(value) = cached {
        if value == NULL {
            return Err(AppError::NotFound);
        }
        return Ok(Json(json!({"url": value})));
    }
    let clip = state
        .db
        .collection::<MomentClip>("moment_clips")
        .find_one(doc! {"momentId": &moment_id})
        .await?;
    match clip {
        Some(clip) => {
            let _: () = conn
                .set_ex(&key, &clip.url, 10)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(Json(json!({"url": clip.url})))
        }
        None => {
            let _: () = conn
                .set_ex(&key, NULL, 10)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Err(AppError::NotFound)
        }
    }
}
