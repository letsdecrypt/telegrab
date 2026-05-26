use crate::schema::ArcPgPool;
use crate::schema::album_mutation::AlbumMutation;
use crate::schema::album_query::{AlbumLoader, AlbumQuery, TagsForAlbumLoader};
use crate::schema::helper::ArcStates;
use crate::schema::image_query::{ImageLoader, ImageQuery};
use crate::schema::node_query::NodeQuery;
use crate::schema::tag_mutation::TagMutation;
use crate::schema::tag_query::{AlbumsForTagLoader, TagLoader, TagQuery};
use crate::schema::task_mutation::TaskMutation;
use crate::schema::task_query::TaskQuery;
use crate::schema::task_subscription::TaskSubscription;
use async_graphql::dataloader::{DataLoader, LruCache};
use async_graphql::runtime::{TokioSpawner, TokioTimer};
use async_graphql::{MergedObject, MergedSubscription, Schema};

pub type GallerySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
#[derive(MergedObject, Default)]
pub struct QueryRoot(AlbumQuery, ImageQuery, TaskQuery, NodeQuery, TagQuery);
#[derive(MergedObject, Default)]
pub struct MutationRoot(AlbumMutation, TaskMutation, TagMutation);
#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(TaskSubscription);

pub fn create_schema(pool: ArcPgPool, states: ArcStates) -> GallerySchema {
    // Create DataLoaders with LRU cache (max 1000 items per loader)
    // This provides cross-request caching with automatic eviction
    let album_loader = DataLoader::with_cache(
        AlbumLoader {
            pool: pool.clone(),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    let image_loader = DataLoader::with_cache(
        ImageLoader {
            pool: pool.clone(),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    let tag_for_album_loader = DataLoader::with_cache(
        TagsForAlbumLoader {
            pool: pool.clone(),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    let tag_loader = DataLoader::with_cache(
        TagLoader {
            pool: pool.clone(),
        },
        TokioSpawner::current(),
        TokioTimer::default(),
        LruCache::new(1000),
    );

    let albums_for_tag_loader = DataLoader::with_cache(
        AlbumsForTagLoader {
            pool: pool.clone(),
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
    .data(tag_loader)
    .data(albums_for_tag_loader)
    .data(tag_for_album_loader)
    .finish()
}
