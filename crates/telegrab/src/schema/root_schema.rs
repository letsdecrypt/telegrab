use crate::schema::album_mutation::AlbumMutation;
use crate::schema::album_query::AlbumQuery;
use crate::schema::helper::ArcStates;
use crate::schema::image_query::ImageQuery;
use crate::schema::node_query::NodeQuery;
use crate::schema::task_mutation::TaskMutation;
use crate::schema::task_query::TaskQuery;
use crate::schema::task_subscription::TaskSubscription;
use crate::schema::ArcPgPool;
use async_graphql::{MergedObject, MergedSubscription, Schema};

pub type GallerySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
#[derive(MergedObject, Default)]
pub struct QueryRoot(AlbumQuery, ImageQuery, TaskQuery, NodeQuery);
#[derive(MergedObject, Default)]
pub struct MutationRoot(AlbumMutation, TaskMutation);
#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(TaskSubscription);

pub fn create_schema(pool: ArcPgPool, states: ArcStates) -> GallerySchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(pool)
    .data(states)
    .finish()
}
