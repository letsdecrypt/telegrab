use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::serde::iso8601;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cbz {
    pub id: i32,
    pub doc_id: Option<i32>,
    pub path: String,
    #[serde(with = "iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub updated_at: OffsetDateTime,
}
