use crate::model::entity::doc::Doc;
use crate::schema::image_query::Image;
use crate::schema::image_query::{ImagesConnectionName, ImagesEdgeName};
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

pub struct AlbumLoader {
    pool: ArcPgPool,
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

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Album {
    pub doc_id: i32,
    pub id: String,
    pub title: Option<String>,
    pub page_title: Option<String>,
    pub page_date: Option<OffsetDateTime>,
    pub status: i16,
    pub count: usize,
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
    async fn images(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
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
struct AlbumsConnectionName;
impl ConnectionNameType for AlbumsConnectionName {
    fn type_name<T: OutputType>() -> String {
        "AlbumsConnection".to_string()
    }
}
struct AlbumsEdgeName;
impl EdgeNameType for AlbumsEdgeName {
    fn type_name<T: OutputType>() -> String {
        "AlbumsEdge".to_string()
    }
}

#[derive(Default)]
pub struct AlbumQuery;

#[Object]
impl AlbumQuery {
    async fn album(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Album> {
        let pool = ctx.data::<ArcPgPool>()?;
        let (_, id) = from_global_id(id.0.as_str())?;
        let _loader = ctx.data::<DataLoader<AlbumLoader, LruCache>>(); // todo: data loader
        let doc = service::doc::get_doc_by_id(pool, id as i32).await?;
        let album = doc.into();
        Ok(album)
    }

    async fn albums(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
        title: Option<String>,
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
                let paged_docs = service::doc::get_cursor_based_pagination_docs(pool, pagination, title)
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
}
