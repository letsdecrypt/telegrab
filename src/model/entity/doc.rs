use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use time::serde::iso8601;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    pub id: i32,
    pub status: i16,
    pub url: String,
    pub page_title: Option<String>,
    #[serde(with = "iso8601::option")]
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
    #[serde(with = "iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "iso8601")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComicInfoXml {
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

impl From<Doc> for ComicInfoXml {
    fn from(doc: Doc) -> Self {
        ComicInfoXml {
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
