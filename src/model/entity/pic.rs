use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::serde::iso8601;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pic {
    pub id: i32,
    pub doc_id: i32,
    pub url: String,
    pub seq: i32,
    pub status: i16,
    #[serde(with = "iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub updated_at: OffsetDateTime,
}
