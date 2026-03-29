use crate::schema::ArcPgPool;
use crate::schema::album_mutation::AlbumMutation;
use crate::schema::album_query::{AlbumLoader, AlbumQuery};
use crate::schema::helper::ArcStates;
use crate::schema::image_query::{ImageLoader, ImageQuery};
use crate::schema::node_query::NodeQuery;
use crate::schema::task_mutation::TaskMutation;
use crate::schema::task_query::TaskQuery;
use crate::schema::task_subscription::TaskSubscription;
use async_graphql::dataloader::{DataLoader, LruCache};
use async_graphql::runtime::{TokioSpawner, TokioTimer};
use async_graphql::{MergedObject, MergedSubscription, Schema};
use std::sync::Arc;

pub type GallerySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
#[derive(MergedObject, Default)]
pub struct QueryRoot(AlbumQuery, ImageQuery, TaskQuery, NodeQuery);
#[derive(MergedObject, Default)]
pub struct MutationRoot(AlbumMutation, TaskMutation);
#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(TaskSubscription);

pub fn create_schema(pool: ArcPgPool, states: ArcStates) -> GallerySchema {
    // Create DataLoaders with LRU cache (max 1000 items per loader)
    // This provides cross-request caching with automatic eviction
    let album_loader = DataLoader::with_cache(
        AlbumLoader {
            pool: Arc::clone(&pool),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    let image_loader = DataLoader::with_cache(
        ImageLoader {
            pool: Arc::clone(&pool),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(pool)
    .data(states)
    .data(album_loader)
    .data(image_loader)
    .finish()
}
