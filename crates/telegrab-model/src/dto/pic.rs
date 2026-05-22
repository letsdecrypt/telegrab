use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutatePicReq {
    pub doc_id: i32,
    pub url: String,
    pub seq: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PicQuery {
    pub doc_id: Option<i32>,
}
