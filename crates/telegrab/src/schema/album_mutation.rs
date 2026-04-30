use crate::model::dto::doc::{CreateDocReq, UpdateDocReq};
use crate::schema::album_query::Album;
use crate::schema::{ArcPgPool, from_global_id};
use crate::service;
use async_graphql::{Context, InputObject, Object, SimpleObject};
use time::OffsetDateTime;

/// Input for creating a new album
#[derive(InputObject, Debug, Clone)]
pub struct CreateAlbumInput {
    /// URL of the album to parse and create
    pub url: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

impl From<CreateAlbumInput> for CreateDocReq {
    fn from(input: CreateAlbumInput) -> Self {
        Self { url: input.url }
    }
}

/// Payload returned after creating an album
#[derive(SimpleObject, Debug, Clone)]
pub struct CreateAlbumPayload {
    /// The newly created album
    pub album: Album,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for updating an existing album
#[derive(InputObject, Debug, Clone)]
pub struct UpdateAlbumInput {
    /// Global ID of the album to update
    pub id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
    /// Page title
    pub page_title: Option<String>,
    /// Publication date
    pub page_date: Option<OffsetDateTime>,
    /// Album title
    pub title: Option<String>,
    /// Series name
    pub series: Option<String>,
    /// Issue number
    pub number: Option<String>,
    /// Issue count
    pub count: Option<String>,
    /// Volume number
    pub volume: Option<String>,
    /// Summary description
    pub summary: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Publication year
    pub year: Option<i32>,
    /// Publication month
    pub month: Option<i32>,
    /// Publication day
    pub day: Option<i32>,
    /// Writer credit
    pub writer: Option<String>,
    /// Penciller credit
    pub penciller: Option<String>,
    /// Inker credit
    pub inker: Option<String>,
    /// Colorist credit
    pub colorist: Option<String>,
    /// Letterer credit
    pub letterer: Option<String>,
    /// Cover artist credit
    pub cover_artist: Option<String>,
    /// Editor credit
    pub editor: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Imprint name
    pub imprint: Option<String>,
    /// Genre classification
    pub genre: Option<String>,
    /// Tags string
    pub tags: Option<String>,
    /// Web URL
    pub web: Option<String>,
    /// Page count
    pub page_count: Option<String>,
    /// Language code
    pub language: Option<String>,
    /// Format (e.g., Comic, Manga)
    pub format: Option<String>,
    /// Whether the album is black and white
    pub black_and_white: Option<bool>,
    /// Characters featured
    pub characters: Option<String>,
    /// Teams featured
    pub teams: Option<String>,
    /// Locations featured
    pub locations: Option<String>,
    /// Scan information
    pub scan_information: Option<String>,
    /// Story arc name
    pub story_arc: Option<String>,
    /// Series group name
    pub series_group: Option<String>,
    /// Age rating
    pub age_rating: Option<String>,
    /// Community rating
    pub community_rating: Option<String>,
    /// Critical rating
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

/// Payload returned after updating an album
#[derive(SimpleObject, Debug, Clone)]
pub struct UpdateAlbumPayload {
    /// The updated album
    pub album: Album,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for deleting an album
#[derive(InputObject, Debug, Clone)]
pub struct DeleteAlbumInput {
    /// Global ID of the album to delete
    pub id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after deleting an album
#[derive(SimpleObject, Debug, Clone)]
pub struct DeleteAlbumPayload {
    /// Global ID of the deleted album
    pub deleted_id: String,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Mutations for album CRUD operations
#[derive(Default)]
pub struct AlbumMutation;

#[Object]
impl AlbumMutation {
    /// Create a new album from a URL
    async fn add_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for creating a new album")] input: CreateAlbumInput,
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

    /// Update an existing album's metadata
    async fn update_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for updating an album")] input: UpdateAlbumInput,
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

    /// Delete an album by its global ID
    async fn delete_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for deleting an album")] input: DeleteAlbumInput,
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
