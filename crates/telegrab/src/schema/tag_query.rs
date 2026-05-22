use crate::model::entity::tag::{Tag as TagEntity, TagWithAlbumCount};
use crate::schema::album_query::Album;
use crate::schema::album_query::{AlbumsConnectionName, AlbumsEdgeName};
use crate::schema::{
    from_global_id, offset_to_cursor, to_global_id, ArcPgPool, ConnectionFields, RelayTy,
};
use crate::service;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::dataloader::{DataLoader, Loader, LruCache};
use async_graphql::{ComplexObject, Context, Object, SimpleObject, ID};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// DataLoader for batch loading tags by their IDs
pub struct TagLoader {
    pub pool: ArcPgPool,
}

impl Loader<i32> for TagLoader {
    type Value = Tag;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let tags_result = service::tag::get_tags_by_ids(&self.pool, keys).await;
        match tags_result {
            Ok(tag_entities) => {
                let tags: Vec<Tag> = tag_entities.into_iter().map(|doc| doc.into()).collect();
                let tags_map: HashMap<i32, Tag> = tags
                    .into_iter()
                    .map(|album| (album.tag_id, album))
                    .collect();
                Ok(tags_map)
            }
            Err(e) => Err(Arc::new(e)),
        }
    }
}

/// DataLoader for batch loading albums associated with tags
pub struct AlbumsForTagLoader {
    pub pool: ArcPgPool,
}

impl Loader<i32> for AlbumsForTagLoader {
    type Value = Vec<Album>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let docs_map = service::tag::get_albums_for_tags(&self.pool, keys)
            .await
            .map_err(Arc::new)?;
        let results: HashMap<i32, Vec<Album>> = docs_map
            .into_iter()
            .map(|(tag_id, docs)| {
                let albums: Vec<Album> = docs.into_iter().map(|doc| doc.into()).collect();
                (tag_id, albums)
            })
            .collect();
        Ok(results)
    }
}

type TagAlbumsConnection =
    Connection<String, Album, ConnectionFields, EmptyFields, AlbumsConnectionName, AlbumsEdgeName>;

/// A tag that can be associated with albums
#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Tag {
    /// Internal database ID
    pub tag_id: i32,
    /// Global unique ID (Relay-style)
    pub id: String,
    /// Tag name
    pub name: String,
    /// Optional tag description
    pub description: Option<String>,
    /// Number of albums using this tag
    pub album_count: i64,
}

impl From<TagWithAlbumCount> for Tag {
    fn from(value: TagWithAlbumCount) -> Self {
        Self {
            tag_id: value.id,
            id: to_global_id(RelayTy::Tag, value.id as usize),
            name: value.name,
            description: value.description,
            album_count: value.album_count,
        }
    }
}

impl From<TagEntity> for Tag {
    fn from(value: TagEntity) -> Self {
        Self {
            tag_id: value.id,
            id: to_global_id(RelayTy::Tag, value.id as usize),
            name: value.name,
            description: value.description,
            album_count: 0,
        }
    }
}

#[ComplexObject]
impl Tag {
    /// Albums associated with this tag
    async fn albums(&self, ctx: &Context<'_>) -> async_graphql::Result<TagAlbumsConnection> {
        let loader = ctx.data::<DataLoader<AlbumsForTagLoader, LruCache>>()?;
        let albums = loader
            .load_one(self.tag_id)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .unwrap_or_default();

        let total = albums.len();
        let mut connection = Connection::with_additional_fields(
            false,
            false,
            ConnectionFields { total_count: total },
        );
        connection.edges.extend(albums.into_iter().map(|n| {
            async_graphql::connection::Edge::with_additional_fields(
                offset_to_cursor(n.doc_id as usize),
                n,
                EmptyFields,
            )
        }));

        Ok(connection)
    }
}

// ════════════════════════════════════════════════════════════
// Tag Suggestion 类型
// ════════════════════════════════════════════════════════════

/// Category of a tag suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
pub enum TagCategory {
    Author,
    Circle,
    Source,
    Magazine,
    Event,
    Language,
    Edition,
}

impl From<service::tag_suggestion::TagCategory> for TagCategory {
    fn from(cat: service::tag_suggestion::TagCategory) -> Self {
        match cat {
            service::tag_suggestion::TagCategory::Author => TagCategory::Author,
            service::tag_suggestion::TagCategory::Circle => TagCategory::Circle,
            service::tag_suggestion::TagCategory::Source => TagCategory::Source,
            service::tag_suggestion::TagCategory::Magazine => TagCategory::Magazine,
            service::tag_suggestion::TagCategory::Event => TagCategory::Event,
            service::tag_suggestion::TagCategory::Language => TagCategory::Language,
            service::tag_suggestion::TagCategory::Edition => TagCategory::Edition,
        }
    }
}

/// A tag candidate extracted from an album's page_title
#[derive(Debug, Clone, SimpleObject)]
pub struct TagSuggestion {
    /// Suggested tag name
    pub name: String,
    /// Category of the suggestion
    pub category: TagCategory,
    /// The existing tag if one with this name already exists
    pub existing_tag: Option<Tag>,
    /// Whether this tag is already linked to the album
    pub already_linked: bool,
}

/// Root query for tag-related operations
#[derive(Default)]
pub struct TagQuery;

#[Object]
impl TagQuery {
    /// Get a single tag by its global ID
    async fn tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Global ID of the tag")] id: ID,
    ) -> async_graphql::Result<Tag> {
        let (_, id) = from_global_id(id.0.as_str())?;
        let loader = ctx.data::<DataLoader<TagLoader, LruCache>>()?;
        let tag = loader
            .load_one(id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .ok_or_else(|| async_graphql::Error::new("Tag not found"))?;
        Ok(tag.into())
    }

    /// Get all tags
    async fn tags(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Tag>> {
        let pool = ctx.data::<ArcPgPool>()?;
        let tags = service::tag::get_all_tags(pool)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
        let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
        Ok(tag_gqls)
    }

    /// Search tags by keyword, optionally filtering by album
    async fn tag_search(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search keyword for tag name")] keyword: Option<String>,
        #[graphql(desc = "Album global ID to exclude tags already on this album")] album_id: Option<ID>,
        #[graphql(desc = "Maximum number of tags to return (default: 5)")] first: Option<i32>,
    ) -> async_graphql::Result<Vec<Tag>> {
        let pool = ctx.data::<ArcPgPool>()?;
        let limit = first.unwrap_or(5) as i32;

        match (keyword, album_id) {
            (Some(kw), Some(album_id)) => {
                if kw.is_empty() {
                    let (_, album_id) = from_global_id(album_id.0.as_str())?;
                    let tags = service::tag::get_tags_excluding_album(pool, album_id as i32)
                        .await
                        .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                    let mut result: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();

                    let recent = service::tag::get_recent_tags(pool, limit)
                        .await
                        .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                    for tag in recent {
                        if !result.iter().any(|t| t.name == tag.name) {
                            result.push(tag.into());
                        }
                        if result.len() >= limit as usize {
                            break;
                        }
                    }
                    Ok(result)
                } else {
                    let (_, album_id) = from_global_id(album_id.0.as_str())?;
                    let tags = service::tag::search_tags_excluding_album(
                        pool,
                        &kw,
                        album_id as i32,
                        limit,
                    )
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                    let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
                    Ok(tag_gqls)
                }
            }
            (Some(kw), None) => {
                let tags = service::tag::search_tags(pool, &kw, limit)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
                Ok(tag_gqls)
            }
            (None, Some(album_id)) => {
                let (_, album_id) = from_global_id(album_id.0.as_str())?;
                let tags = service::tag::get_tags_excluding_album(pool, album_id as i32)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
                Ok(tag_gqls)
            }
            (None, None) => {
                let tags = service::tag::get_recent_tags(pool, limit)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
                let tag_gqls: Vec<Tag> = tags.into_iter().map(|t| t.into()).collect();
                Ok(tag_gqls)
            }
        }
    }

    /// Get albums associated with a specific tag
    async fn albums_by_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Global ID of the tag")] id: ID,
        #[graphql(desc = "Cursor to fetch items after (forward pagination)")] _after: Option<String>,
        #[graphql(desc = "Cursor to fetch items before (backward pagination)")] _before: Option<String>,
        #[graphql(desc = "Number of items to fetch from the start (forward pagination)")] _first: Option<i32>,
        #[graphql(desc = "Number of items to fetch from the end (backward pagination)")] _last: Option<i32>,
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
        let (_, tag_id) = from_global_id(id.0.as_str())?;
        let loader = ctx.data::<DataLoader<AlbumsForTagLoader, LruCache>>()?;
        let albums = loader
            .load_one(tag_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .unwrap_or_default();

        let total = albums.len();
        let mut connection = Connection::with_additional_fields(
            false,
            false,
            ConnectionFields { total_count: total },
        );
        connection.edges.extend(albums.into_iter().map(|n| {
            Edge::with_additional_fields(offset_to_cursor(n.doc_id as usize), n, EmptyFields)
        }));

        Ok(connection)
    }

    /// Analyze an album's page_title and suggest candidate tags.
    /// Returns suggestions with categories and existing-tag lookup.
    async fn suggest_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Album global ID to analyze")] album_id: ID,
    ) -> async_graphql::Result<Vec<TagSuggestion>> {
        let pool = ctx.data::<ArcPgPool>()?;
        let (_, album_id) = from_global_id(album_id.as_str())?;

        let doc = service::doc::get_doc_by_id(pool, album_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        let page_title = doc.page_title.as_deref().unwrap_or("");
        let suggestions = service::tag_suggestion::extract(page_title);

        let album_tags = service::tag::get_tags_for_album(pool, album_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
        let linked_names: HashSet<&str> =
            album_tags.iter().map(|t| t.name.as_str()).collect();

        let mut result = Vec::new();
        for sug in suggestions {
            let existing = service::tag::get_tag_by_name(pool, &sug.name)
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
            let already_linked = linked_names.contains(sug.name.as_str());
            result.push(TagSuggestion {
                name: sug.name,
                category: sug.category.into(),
                existing_tag: existing.map(|t| t.into()),
                already_linked,
            });
        }

        Ok(result)
    }
}
