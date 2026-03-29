use crate::model::entity::pic::Pic;
use crate::schema::{ArcPgPool, RelayTy, from_global_id, to_global_id};
use crate::service;
use async_graphql::connection::{ConnectionNameType, EdgeNameType};
use async_graphql::dataloader::{DataLoader, Loader, LruCache};
use async_graphql::{Context, Object, OutputType, Result, SimpleObject};
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct ImageLoader {
    pub pool: ArcPgPool,
}
impl Loader<i32> for ImageLoader {
    type Value = Image;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[i32],
    ) -> std::result::Result<HashMap<i32, Self::Value>, Self::Error> {
        let pics_result = service::pic::get_pics_by_ids(&self.pool, keys).await;
        match pics_result {
            Ok(pics) => {
                let images: Vec<Image> = pics.into_iter().map(|doc| doc.into()).collect();
                let images_map: HashMap<i32, Image> = images
                    .into_iter()
                    .map(|image| (image.pic_id, image))
                    .collect();
                Ok(images_map)
            }
            Err(e) => Err(Arc::new(e)),
        }
    }
}

pub struct ImagesConnectionName;
impl ConnectionNameType for ImagesConnectionName {
    fn type_name<T: OutputType>() -> String {
        "ImagesConnection".to_string()
    }
}
pub struct ImagesEdgeName;
impl EdgeNameType for ImagesEdgeName {
    fn type_name<T: OutputType>() -> String {
        "ImagesEdge".to_string()
    }
}

#[derive(Default)]
pub struct ImageQuery;

#[Object]
impl ImageQuery {
    async fn image(&self, ctx: &Context<'_>, id: String) -> Result<Image> {
        let (_, id) = from_global_id(id.as_str())?;
        let loader = ctx.data::<DataLoader<ImageLoader, LruCache>>()?;
        let image = loader
            .load_one(id as i32)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{}", e)))?
            .ok_or_else(|| async_graphql::Error::new("Image not found"))?;
        Ok(image)
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Image {
    pub pic_id: i32,
    pub id: String,
    pub doc_id: String,
    pub url: String,
    pub seq: i32,
    pub status: i16,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Pic> for Image {
    fn from(pic: Pic) -> Self {
        Image {
            pic_id: pic.id,
            id: to_global_id(RelayTy::Image, pic.id as usize),
            doc_id: to_global_id(RelayTy::Album, pic.doc_id as usize),
            url: pic.url,
            seq: pic.seq,
            status: pic.status,
            created_at: pic.created_at,
            updated_at: pic.updated_at,
        }
    }
}
