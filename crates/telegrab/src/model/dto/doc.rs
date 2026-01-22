use serde::Deserialize;
use time::OffsetDateTime;

// 创建用户的请求体（API 入参）
#[derive(Debug, Deserialize)]
pub struct CreateDocReq {
    pub url: String,
}

// 更新文档的请求体（API 入参）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocReq {
    pub page_title: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub page_date: Option<OffsetDateTime>,
    pub title: Option<String>,
    pub series: Option<String>,
    pub number: Option<String>,
    pub count: Option<String>,
    pub volume: Option<String>,
    pub summary: Option<String>,
    pub notes: Option<String>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub inker: Option<String>,
    pub colorist: Option<String>,
    pub letterer: Option<String>,
    pub cover_artist: Option<String>,
    pub editor: Option<String>,
    pub publisher: Option<String>,
    pub imprint: Option<String>,
    pub genre: Option<String>,
    pub tags: Option<String>,
    pub web: Option<String>,
    pub page_count: Option<String>,
    pub language: Option<String>,
    pub format: Option<String>,
    pub black_and_white: Option<bool>,
    pub characters: Option<String>,
    pub teams: Option<String>,
    pub locations: Option<String>,
    pub scan_information: Option<String>,
    pub story_arc: Option<String>,
    pub series_group: Option<String>,
    pub age_rating: Option<String>,
    pub community_rating: Option<String>,
    pub critical_rating: Option<String>,
}
