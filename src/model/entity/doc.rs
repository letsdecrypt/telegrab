use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::serde::rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    pub id: i32,
    pub cbz_id: Option<i32>,
    pub status: i16,
    pub url: String,
    pub page_title: Option<String>,
    #[serde(with = "rfc3339::option")]
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
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShimDoc {
    pub id: i32,
    pub url: String,
    pub page_title: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "Page")]
pub struct PageInfo {
    #[serde(rename = "@Image")]
    pub image: u32,
    #[serde(rename = "@Type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

impl PageInfo {
    pub fn with_count(count: u32) -> Vec<Self> {
        (0..count)
            .map(|idx| {
                let type_str = match idx {
                    0 => "FrontCover".into(),
                    _ if idx == count - 1 => "BackCover".into(),
                    _ => "Story".into(),
                };
                Self {
                    image: idx,
                    type_: Some(type_str),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pages{
    #[serde(rename = "Page")]
    pub page: Vec<PageInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComicInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penciller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colorist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letterer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<String>,
    pub pages: Pages,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub black_and_white: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_arc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub community_rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_rating: Option<String>,
}

impl ComicInfo {
    pub fn from_doc(doc: Doc) -> Self {
        let page_info = PageInfo::with_count(doc.page_count.clone().unwrap().parse().unwrap_or(0));
        ComicInfo {
            title: doc.title,
            series: doc.series,
            number: doc.number,
            count: doc.count,
            volume: doc.volume,
            summary: doc.summary,
            notes: doc.notes,
            year: doc.year,
            month: doc.month,
            day: doc.day,
            writer: doc.writer,
            penciller: doc.penciller,
            inker: doc.inker,
            colorist: doc.colorist,
            letterer: doc.letterer,
            cover_artist: doc.cover_artist,
            editor: doc.editor,
            publisher: doc.publisher,
            imprint: doc.imprint,
            genre: doc.genre,
            tags: doc.tags,
            web: doc.web,
            page_count: doc.page_count,
            pages: Pages { page: page_info },
            language: doc.language,
            format: doc.format,
            black_and_white: doc.black_and_white,
            characters: doc.characters,
            teams: doc.teams,
            locations: doc.locations,
            scan_information: doc.scan_information,
            story_arc: doc.story_arc,
            series_group: doc.series_group,
            age_rating: doc.age_rating,
            community_rating: doc.community_rating,
            critical_rating: doc.critical_rating,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegraphPost {
    pub url: String,
    pub title: String,
    pub date: Option<String>,
    pub image_urls: Vec<String>,
}
