use crate::schema::album_query::TagsForAlbumLoader;
use crate::schema::tag_query::{AlbumsForTagLoader, Tag};
use crate::schema::{ArcPgPool, from_global_id};
use crate::service;
use async_graphql::dataloader::{DataLoader, LruCache};
use async_graphql::{Context, InputObject, Object, SimpleObject, ID};

/// Input for creating a new tag
#[derive(InputObject, Debug, Clone)]
pub struct CreateTagInput {
    /// Name of the tag
    pub name: String,
    /// Optional description of the tag
    pub description: Option<String>,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after creating a tag
#[derive(SimpleObject, Debug, Clone)]
pub struct CreateTagPayload {
    /// The newly created tag
    pub tag: Tag,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for updating an existing tag
#[derive(InputObject, Debug, Clone)]
pub struct UpdateTagInput {
    /// Global ID of the tag to update
    pub id: String,
    /// New name for the tag
    pub name: Option<String>,
    /// New description for the tag
    pub description: Option<String>,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after updating a tag
#[derive(SimpleObject, Debug, Clone)]
pub struct UpdateTagPayload {
    /// The updated tag
    pub tag: Tag,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for deleting a tag
#[derive(InputObject, Debug, Clone)]
pub struct DeleteTagInput {
    /// Global ID of the tag to delete
    pub id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after deleting a tag
#[derive(SimpleObject, Debug, Clone)]
pub struct DeleteTagPayload {
    /// Global ID of the deleted tag
    pub deleted_id: String,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for adding a tag to an album
#[derive(InputObject, Debug, Clone)]
pub struct AddTagToAlbumInput {
    /// Global ID of the album
    pub album_id: String,
    /// Global ID of the tag to add
    pub tag_id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after adding a tag to an album
#[derive(SimpleObject, Debug, Clone)]
pub struct AddTagToAlbumPayload {
    /// The tag that was added
    pub tag: Tag,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for removing a tag from an album
#[derive(InputObject, Debug, Clone)]
pub struct RemoveTagFromAlbumInput {
    /// Global ID of the album
    pub album_id: String,
    /// Global ID of the tag to remove
    pub tag_id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after removing a tag from an album
#[derive(SimpleObject, Debug, Clone)]
pub struct RemoveTagFromAlbumPayload {
    /// Global ID of the removed tag
    pub removed_tag_id: String,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Mutations for tag CRUD and album-tag association operations
#[derive(Default)]
pub struct TagMutation;

#[Object]
impl TagMutation {
    /// Create a new tag
    async fn create_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for creating a new tag")] input: CreateTagInput,
    ) -> async_graphql::Result<CreateTagPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let client_mutation_id = input.client_mutation_id.clone();

        let exists = service::tag::tag_name_exists(pool, &input.name)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        if exists {
            return Err(async_graphql::Error::new("Tag name already exists"));
        }

        let tag = service::tag::create_tag(pool, &input.name, input.description.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        Ok(CreateTagPayload {
            tag: tag.into(),
            client_mutation_id,
        })
    }

    /// Update an existing tag's name or description
    async fn update_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for updating a tag")] input: UpdateTagInput,
    ) -> async_graphql::Result<UpdateTagPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let (_, id) = from_global_id(input.id.as_str())?;

        if let Some(name) = &input.name {
            let exists = service::tag::tag_name_exists_excluding(pool, name, id as i32)
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

            if exists {
                return Err(async_graphql::Error::new("Tag name already exists"));
            }
        }

        let tag = service::tag::update_tag(pool, id as i32, input.name.as_deref(), input.description.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        Ok(UpdateTagPayload {
            tag: tag.into(),
            client_mutation_id,
        })
    }

    /// Delete a tag by its global ID
    async fn delete_tag(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for deleting a tag")] input: DeleteTagInput,
    ) -> async_graphql::Result<DeleteTagPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let input_id = input.id.clone();
        let client_mutation_id = input.client_mutation_id.clone();
        let (_, id) = from_global_id(input_id.as_str())?;

        let count = service::tag::delete_tag(pool, id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        if count == 0 {
            return Err(async_graphql::Error::new("No tag found"));
        }

        Ok(DeleteTagPayload {
            deleted_id: input_id,
            client_mutation_id,
        })
    }

    /// Associate a tag with an album
    async fn add_tag_to_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for adding a tag to an album")] input: AddTagToAlbumInput,
    ) -> async_graphql::Result<AddTagToAlbumPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let (_, album_id) = from_global_id(input.album_id.as_str())?;
        let (_, tag_id) = from_global_id(input.tag_id.as_str())?;

        let tag = service::tag::get_tag_by_id(pool, tag_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        let exists = service::tag::album_tag_exists(pool, album_id as i32, tag_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        if exists {
            return Err(async_graphql::Error::new("Tag already associated with this album"));
        }

        service::tag::add_tag_to_album(pool, album_id as i32, tag_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        // Invalidate caches
        let tags_for_album_loader = ctx.data::<DataLoader<TagsForAlbumLoader, LruCache>>()?;
        tags_for_album_loader.clear_one(&(album_id as i32));

        let albums_for_tag_loader = ctx.data::<DataLoader<AlbumsForTagLoader, LruCache>>()?;
        albums_for_tag_loader.clear_one(&(tag_id as i32));

        Ok(AddTagToAlbumPayload {
            tag: tag.into(),
            client_mutation_id,
        })
    }

    /// Remove a tag association from an album
    async fn remove_tag_from_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for removing a tag from an album")] input: RemoveTagFromAlbumInput,
    ) -> async_graphql::Result<RemoveTagFromAlbumPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let input_tag_id = input.tag_id.clone();
        let client_mutation_id = input.client_mutation_id.clone();
        let (_, album_id) = from_global_id(input.album_id.as_str())?;
        let (_, tag_id) = from_global_id(input_tag_id.as_str())?;

        let count = service::tag::remove_tag_from_album(pool, album_id as i32, tag_id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

        if count == 0 {
            return Err(async_graphql::Error::new("Tag not associated with this album"));
        }

        // Invalidate caches
        let tag_for_album_loader = ctx.data::<DataLoader<TagsForAlbumLoader, LruCache>>()?;
        tag_for_album_loader.clear_one(&(album_id as i32));

        let albums_for_tag_loader = ctx.data::<DataLoader<AlbumsForTagLoader, LruCache>>()?;
        albums_for_tag_loader.clear_one(&(tag_id as i32));

        Ok(RemoveTagFromAlbumPayload {
            removed_tag_id: input_tag_id,
            client_mutation_id,
        })
    }

    /// Batch associate tags with an album.
    /// Creates any new tags (by name), then links all tags to the album.
    async fn batch_add_tags_to_album(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Global ID of the album")] album_id: ID,
        #[graphql(desc = "Global IDs of existing tags to add")] tag_ids: Vec<ID>,
        #[graphql(desc = "Names of new tags to create and add")] new_tag_names: Vec<String>,
        #[graphql(desc = "Client mutation ID for Relay support")] client_mutation_id: Option<String>,
    ) -> async_graphql::Result<BatchAddTagsPayload> {
        let pool = ctx.data::<ArcPgPool>()?;
        let (_, album_id) = from_global_id(album_id.as_str())?;
        let album_id = album_id as i32;

        let mut added_tags: Vec<Tag> = Vec::new();

        // 1. Add existing tags
        for tag_id_str in &tag_ids {
            let (_, tag_id) = from_global_id(tag_id_str.as_str())?;
            let tag_id = tag_id as i32;

            let exists = service::tag::album_tag_exists(pool, album_id, tag_id)
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

            if !exists {
                service::tag::add_tag_to_album(pool, album_id, tag_id)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
            }

            let tag = service::tag::get_tag_by_id(pool, tag_id)
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
            added_tags.push(tag.into());
        }

        // 2. Create and add new tags
        for name in &new_tag_names {
            if name.trim().is_empty() {
                continue;
            }

            let tag = match service::tag::get_tag_by_name(pool, name).await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            {
                Some(existing) => existing,
                None => service::tag::create_tag(pool, name, None)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?,
            };

            let already_linked = service::tag::album_tag_exists(pool, album_id, tag.id)
                .await
                .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;

            if !already_linked {
                service::tag::add_tag_to_album(pool, album_id, tag.id)
                    .await
                    .map_err(|e| async_graphql::Error::new(format!("{}", e)))?;
            }

            added_tags.push(tag.into());
        }

        // Invalidate caches
        let tags_for_album_loader = ctx.data::<DataLoader<TagsForAlbumLoader, LruCache>>()?;
        tags_for_album_loader.clear_one(&album_id);
        for tag in &added_tags {
            let albums_for_tag_loader = ctx.data::<DataLoader<AlbumsForTagLoader, LruCache>>()?;
            albums_for_tag_loader.clear_one(&tag.tag_id);
        }

        Ok(BatchAddTagsPayload {
            added_tags,
            client_mutation_id,
        })
    }
}

/// Input for batch adding tags to an album — flattened into mutation args.

/// Payload for batch tag addition
#[derive(SimpleObject, Debug, Clone)]
pub struct BatchAddTagsPayload {
    /// All tags now associated with the album (existing + newly created)
    pub added_tags: Vec<Tag>,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}
