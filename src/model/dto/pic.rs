use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutatePicReq {
    pub doc_id: i32,
    pub url: String,
    pub seq: i32,
}