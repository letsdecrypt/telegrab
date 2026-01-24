use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::serde::rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Pic {
    pub id: i32,
    pub doc_id: i32,
    pub url: String,
    pub seq: i32,
    pub status: i16,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

