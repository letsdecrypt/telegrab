use crate::model::dto::doc::{CreateDocReq, UpdateDocReq};
use crate::schema::album_query::Album;
use crate::schema::{from_global_id, ArcPgPool};
use crate::service;
use async_graphql::{Context, InputObject, Object, SimpleObject};
use time::OffsetDateTime;

#[derive(InputObject, Debug, Clone)]
pub struct CreateAlbumInput {
    #[graphql(validator(url))]
    pub url: String,
    pub client_mutation_id: Option<String>,
}

impl From<CreateAlbumInput> for CreateDocReq {
    fn from(input: CreateAlbumInput) -> Self {
        Self { url: input.url }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct CreateAlbumPayload {
    pub album: Album,
    pub client_mutation_id: Option<String>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateAlbumInput {
    pub id: String,
    pub client_mutation_id: Option<String>,
    pub page_title: Option<String>,
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
impl From<UpdateAlbumInput> for UpdateDocReq {
    fn from(input: UpdateAlbumInput) -> Self {
        Self {
            page_title: input.page_title,
            page_date: input.page_date,
            title: input.title,
            series: input.series,
            number: input.number,
            count: input.count,
            volume: input.volume,
            summary: input.summary,
            notes: input.notes,
            year: input.year,
            month: input.month,
            day: input.day,
            writer: input.writer,
            penciller: input.penciller,
            inker: input.inker,
            colorist: input.colorist,
            letterer: input.letterer,
            cover_artist: input.cover_artist,
            editor: input.editor,
            publisher: input.publisher,
            imprint: input.imprint,
            genre: input.genre,
            tags: input.tags,
            web: input.web,
            page_count: input.page_count,
            language: input.language,
            format: input.format,
            black_and_white: input.black_and_white,
            characters: input.characters,
            teams: input.teams,
            locations: input.locations,
            scan_information: input.scan_information,
            story_arc: input.story_arc,
            series_group: input.series_group,
            age_rating: input.age_rating,
            community_rating: input.community_rating,
            critical_rating: input.critical_rating,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UpdateAlbumPayload {
    pub album: Album,
    pub client_mutation_id: Option<String>,
}

#[derive(InputObject, Debug, Clone)]
pub struct DeleteAlbumInput {
    pub id: String,
    pub client_mutation_id: Option<String>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DeleteAlbumPayload {
    pub deleted_id: String,
    pub client_mutation_id: Option<String>,
}

#[derive(Default)]
pub struct AlbumMutation;

#[Object]
impl AlbumMutation {
    async fn add_album(
        &self,
        ctx: &Context<'_>,
        input: CreateAlbumInput,
    ) -> async_graphql::Result<CreateAlbumPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let new_doc: CreateDocReq = input.into();
        let doc = service::doc::create_doc(pool, new_doc).await?;
        Ok(CreateAlbumPayload {
            album: doc.into(),
            client_mutation_id,
        })
    }
    async fn update_album(
        &self,
        ctx: &Context<'_>,
        input: UpdateAlbumInput,
    ) -> async_graphql::Result<UpdateAlbumPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let (_, id) = from_global_id(input.id.as_str())?;
        let client_mutation_id = input.client_mutation_id.clone();
        let new_doc: UpdateDocReq = input.into();
        let doc = service::doc::update_doc(pool, id as i32, new_doc).await?;
        Ok(UpdateAlbumPayload {
            album: doc.into(),
            client_mutation_id,
        })
    }
    async fn delete_album(
        &self,
        ctx: &Context<'_>,
        input: DeleteAlbumInput,
    ) -> async_graphql::Result<DeleteAlbumPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let input_id = input.id.clone();
        let (_, id) = from_global_id(input_id.as_str())?;
        let client_mutation_id = input.client_mutation_id.clone();
        let count = service::doc::delete_doc_by_id(pool, id as i32).await?;
        if count == 0 {
            return Err(async_graphql::Error::new("No Album found"));
        }
        Ok(DeleteAlbumPayload {
            deleted_id: input_id,
            client_mutation_id,
        })
    }
}
