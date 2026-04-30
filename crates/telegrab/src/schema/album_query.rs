use crate::model::entity::doc::Doc;
use crate::model::SortOrder;
use crate::schema::image_query::Image;
use crate::schema::image_query::{ImagesConnectionName, ImagesEdgeName};
use crate::schema::tag_query::Tag;
use crate::schema::{
    from_global_id, offset_to_cursor, process_pagination, to_global_id, ArcPgPool, ConnectionFields,
    RelayTy,
};
use crate::service;
use async_graphql::connection::{Connection, ConnectionNameType, Edge, EdgeNameType, EmptyFields};
use async_graphql::dataloader::{DataLoader, Loader, LruCache};
use async_graphql::{connection, ComplexObject, Context, Object, OutputType, SimpleObject, ID};
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;

/// DataLoader for batch loading albums by their IDs
pub struct AlbumLoader {
    pub pool: ArcPgPool,
}

impl Loader<i32> for AlbumLoader {
    type Value = Album;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let docs_result = service::doc::get_docs_by_ids(&self.pool, keys).await;
        match docs_result {
            Ok(docs) => {
                let albums: Vec<Album> = docs.into_iter().map(|doc| doc.into()).collect();
                let albums_map: HashMap<i32, Album> = albums
                    .into_iter()
                    .map(|album| (album.doc_id, album))
                    .collect();
                Ok(albums_map)
            }
            Err(e) => Err(Arc::new(e)),
        }
    }
}

/// DataLoader for batch loading tags for albums
pub struct TagsForAlbumLoader {
    pub pool: ArcPgPool,
}

impl Loader<i32> for TagsForAlbumLoader {
    type Value = Vec<Tag>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let tags_map = service::tag::get_tags_for_albums(&self.pool, keys)
            .await
            .map_err(Arc::new)?;
        let results: HashMap<i32, Vec<Tag>> = tags_map
            .into_iter()
            .map(|(album_id, tags)| {
                let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
                (album_id, tag_gqls)
            })
            .collect();
        Ok(results)
    }
}

/// Paginated search result for albums
#[derive(Debug, Clone, SimpleObject)]
pub struct AlbumSearchResult {
    /// List of albums on the current page
    pub albums: Vec<Album>,
    /// Total number of matching records
    pub total: i32,
    /// Current page number (1-based)
    pub page: i32,
    /// Number of items per page
    pub page_size: i32,
    /// Total number of pages
    pub total_pages: i32,
}

/// An album (doc) containing images
#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Album {
    /// Internal database ID
    pub doc_id: i32,
    /// Global unique ID (Relay-style)
    pub id: String,
    /// Album title
    pub title: Option<String>,
    /// Page title (often the original filename or source title)
    pub page_title: Option<String>,
    /// Publication or creation date
    pub page_date: Option<OffsetDateTime>,
    /// Album status (e.g., active, archived)
    pub status: i16,
    /// Number of images in the album
    pub count: usize,
    /// URL to the album source or cover
    pub url: String,
}

impl From<Doc> for Album {
    fn from(value: Doc) -> Self {
        Self {
            doc_id: value.id,
            id: to_global_id(RelayTy::Album, value.id as usize),
            title: value.title,
            page_title: value.page_title,
            page_date: value.page_date,
            status: value.status,
            count: value.page_count.map(|s| s as usize).unwrap_or(0),
            url: value.url,
        }
    }
}

#[ComplexObject]
impl Album {
    /// Tags associated with this album
    async fn tags(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Tag>> {
        let loader = ctx.data::<DataLoader<TagsForAlbumLoader, LruCache>>()?;
        let tags = loader
            .load_one(self.doc_id)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .unwrap_or_default();
        Ok(tags)
    }

    /// Images in this album with cursor-based pagination
    async fn images(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Cursor to fetch items after (forward pagination)")] after: Option<String>,
        #[graphql(desc = "Cursor to fetch items before (backward pagination)")] before: Option<String>,
        #[graphql(desc = "Number of items to fetch from the start (forward pagination)")] first: Option<i32>,
        #[graphql(desc = "Number of items to fetch from the end (backward pagination)")] last: Option<i32>,
    ) -> async_graphql::Result<
        Connection<
            String,
            Image,
            ConnectionFields,
            EmptyFields,
            ImagesConnectionName,
            ImagesEdgeName,
        >,
    > {
        let pool = ctx.data::<ArcPgPool>()?;
        connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let pagination = process_pagination(after, before, first, last)
                    .map_err(|e| async_graphql::Error::new(e.message.to_string()))?;
                let paged_pics =
                    service::pic::get_cursor_based_pagination_pics(pool, pagination, self.doc_id)
                        .await
                        .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let images: Vec<Image> =
                    paged_pics.data.into_iter().map(|doc| doc.into()).collect();
                let mut connection = Connection::with_additional_fields(
                    paged_pics.has_prev,
                    paged_pics.has_next,
                    ConnectionFields {
                        total_count: paged_pics.total as usize,
                    },
                );
                connection.edges.extend(images.into_iter().map(|n| {
                    Edge::with_additional_fields(
                        offset_to_cursor(n.pic_id as usize),
                        n,
                        EmptyFields,
                    )
                }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

/// Connection name type for albums (Relay pagination)
pub struct AlbumsConnectionName;

impl ConnectionNameType for AlbumsConnectionName {
    fn type_name<T: OutputType>() -> String {
        "AlbumsConnection".to_string()
    }
}

/// Edge name type for albums (Relay pagination)
pub struct AlbumsEdgeName;

impl EdgeNameType for AlbumsEdgeName {
    fn type_name<T: OutputType>() -> String {
        "AlbumsEdge".to_string()
    }
}

/// Root query for album-related operations
#[derive(Default)]
pub struct AlbumQuery;

#[Object]
impl AlbumQuery {
    /// Get a random album
    async fn random_album(&self, ctx: &Context<'_>) -> async_graphql::Result<Album> {
        let pool = ctx.data::<ArcPgPool>()?;
        let doc = service::doc::get_random_doc(pool).await?;
        let album = doc.into();
        Ok(album)
    }

    /// Get a single album by its global ID
    async fn album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Global ID of the album")] id: ID,
    ) -> async_graphql::Result<Album> {
        let (_, id) = from_global_id(id.0.as_str())?;
        let loader = ctx.data::<DataLoader<AlbumLoader, LruCache>>()?;
        let album = loader
            .load_one(id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .ok_or_else(|| async_graphql::Error::new("Album not found"))?;
        Ok(album)
    }

    /// List all albums with cursor-based pagination
    async fn albums(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Cursor to fetch items after (forward pagination)")] after: Option<String>,
        #[graphql(desc = "Cursor to fetch items before (backward pagination)")] before: Option<String>,
        #[graphql(desc = "Number of items to fetch from the start (forward pagination)")] first: Option<i32>,
        #[graphql(desc = "Number of items to fetch from the end (backward pagination)")] last: Option<i32>,
        #[graphql(desc = "Sort order (ascending or descending)")] order: Option<SortOrder>,
        #[graphql(desc = "Filter by title (partial match)")] title: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            String,
            Album,
            ConnectionFields,
            EmptyFields,
            AlbumsConnectionName,
            AlbumsEdgeName,
        >,
    > {
        let pool = ctx.data::<ArcPgPool>()?;
        connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let pagination = process_pagination(after, before, first, last)
                    .map_err(|e| async_graphql::Error::new(e.message.to_string()))?;
                let paged_docs = service::doc::get_cursor_based_pagination_docs(
                    pool,
                    pagination,
                    order.unwrap_or(SortOrder::Asc),
                    title,
                )
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let albums: Vec<Album> =
                    paged_docs.data.into_iter().map(|doc| doc.into()).collect();
                let mut connection = Connection::with_additional_fields(
                    paged_docs.has_prev,
                    paged_docs.has_next,
                    ConnectionFields {
                        total_count: paged_docs.total as usize,
                    },
                );
                connection.edges.extend(albums.into_iter().map(|n| {
                    Edge::with_additional_fields(
                        offset_to_cursor(n.doc_id as usize),
                        n,
                        EmptyFields,
                    )
                }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    /// Search albums by keyword with offset-based pagination
    async fn album_search(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search keyword (empty or null returns empty result)")] keyword: Option<String>,
        #[graphql(default = 1, desc = "Page number (1-based, default: 1)")] page: i32,
        #[graphql(default = 10, desc = "Items per page (default: 10, max: 100)")] page_size: i32,
    ) -> async_graphql::Result<AlbumSearchResult> {
        let pool = ctx.data::<ArcPgPool>()?;

        // Validate parameters
        let page = if page < 1 { 1 } else { page };
        let page_size = if page_size < 1 { 10 } else if page_size > 100 { 100 } else { page_size };

        // Return empty result if no keyword provided
        match keyword {
            None => Ok(AlbumSearchResult {
                albums: Vec::new(),
                total: 0,
                page,
                page_size,
                total_pages: 0,
            }),
            Some(kw) => {
                if kw.is_empty() {
                    return Ok(AlbumSearchResult {
                        albums: Vec::new(),
                        total: 0,
                        page,
                        page_size,
                        total_pages: 0,
                    });
                }

                let offset = (page - 1) * page_size;
                let docs = service::doc::search_docs_by_keyword(pool, &kw, page_size, offset)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let total = service::doc::count_docs_by_keyword(pool, &kw)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))? as i32;

                let albums: Vec<Album> = docs.into_iter().map(|doc| doc.into()).collect();
                let total_pages = (total + page_size - 1) / page_size;

                Ok(AlbumSearchResult {
                    albums,
                    total,
                    page,
                    page_size,
                    total_pages,
                })
            }
        }
    }
}
