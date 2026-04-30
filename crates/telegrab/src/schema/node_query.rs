use crate::schema::album_query::AlbumLoader;
use crate::schema::image_query::ImageLoader;
use crate::schema::{RelayNode, RelayTy, from_global_id};
use async_graphql::dataloader::{DataLoader, LruCache};
use async_graphql::{Context, Object, Result};

/// Root query for Relay node interface (global ID lookup)
#[derive(Default)]
pub struct NodeQuery;

#[Object]
impl NodeQuery {
    /// Fetch a node by its global ID (Relay spec)
    async fn node(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Global ID of the node")] id: String,
    ) -> Result<Option<RelayNode>> {
        let (ty, id) = from_global_id(id.as_str())?;
        match ty {
            RelayTy::Album => {
                let loader = ctx.data::<DataLoader<AlbumLoader, LruCache>>()?;
                let album = loader.load_one(id as i32).await?;
                Ok(album.map(|a| RelayNode::Album(a)))
            }
            RelayTy::Image => {
                let loader = ctx.data::<DataLoader<ImageLoader, LruCache>>()?;
                let image = loader.load_one(id as i32).await?;
                Ok(image.map(|i| RelayNode::Image(i)))
            }
            _ => Err(async_graphql::Error::new("Invalid node type")),
        }
    }

    /// Fetch multiple nodes by their global IDs (batched Relay spec)
    async fn nodes(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "List of global IDs to fetch")] ids: Vec<String>,
    ) -> Result<Vec<Option<RelayNode>>> {
        // Parse all IDs first
        let mut parsed_ids = Vec::with_capacity(ids.len());
        for id in &ids {
            parsed_ids.push(from_global_id(id.as_str())?);
        }

        // Group IDs by type for batch loading
        let mut album_ids: Vec<i32> = Vec::new();
        let mut image_ids: Vec<i32> = Vec::new();
        let mut id_mapping: Vec<(usize, RelayTy, i32)> = Vec::new();

        for (idx, (ty, id)) in parsed_ids.iter().enumerate() {
            match ty {
                RelayTy::Album => {
                    album_ids.push(*id as i32);
                    id_mapping.push((idx, RelayTy::Album, *id as i32));
                }
                RelayTy::Image => {
                    image_ids.push(*id as i32);
                    id_mapping.push((idx, RelayTy::Image, *id as i32));
                }
                _ => {}
            }
        }

        // Batch load albums
        let album_loader = ctx.data::<DataLoader<AlbumLoader, LruCache>>()?;
        let albums = if !album_ids.is_empty() {
            album_loader.load_many(album_ids).await?
        } else {
            std::collections::HashMap::new()
        };

        // Batch load images
        let image_loader = ctx.data::<DataLoader<ImageLoader, LruCache>>()?;
        let images = if !image_ids.is_empty() {
            image_loader.load_many(image_ids).await?
        } else {
            std::collections::HashMap::new()
        };

        // Build results in original order
        let mut results: Vec<Option<RelayNode>> = (0..ids.len()).map(|_| None).collect();
        for (idx, ty, id) in id_mapping {
            match ty {
                RelayTy::Album => {
                    if let Some(album) = albums.get(&id) {
                        results[idx] = Some(RelayNode::Album(album.clone()));
                    }
                }
                RelayTy::Image => {
                    if let Some(image) = images.get(&id) {
                        results[idx] = Some(RelayNode::Image(image.clone()));
                    }
                }
                _ => {}
            }
        }

        Ok(results)
    }
}
