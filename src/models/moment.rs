use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentClip {
    pub moment_id: String,
    pub url: String,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMomentRequest {
    pub moment_id: String,
    pub url: String,
}
