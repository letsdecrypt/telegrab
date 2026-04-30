use crate::model::{Direction, PaginationArgs};
use crate::schema::album_query::Album;
use crate::schema::image_query::Image;
use crate::schema::tag_query::Tag;
use crate::state::QueueState;
use async_graphql::{Interface, SimpleObject};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as base64;
use serde::{Deserialize, Serialize};
use sqlx_postgres::PgPool;
use std::sync::Arc;

/// Shared PostgreSQL connection pool type
pub type ArcPgPool = Arc<PgPool>;

/// Shared task queue state type
pub type ArcStates = Arc<QueueState>;

/// Relay-style Node interface for polymorphic ID-based lookups
#[derive(Interface)]
#[graphql(
    name = "Node",
    field(name = "id", ty = "String", desc = "The id of the object")
)]
pub enum RelayNode {
    Album(Album),
    Image(Image),
    Tag(Tag),
}

/// Enum of Relay type discriminators used in global IDs
#[derive(Debug, Serialize, Deserialize)]
pub enum RelayTy {
    Album,
    Image,
    Cbz,
    Offset,
    Tag,
}

/// Encode a type and local ID into a base64 global ID
pub fn to_global_id(ty: RelayTy, id: usize) -> String {
    let combined = format!("{}:{}", serde_json::to_string(&ty).unwrap(), id);
    base64.encode(combined)
}

/// Decode a base64 global ID into its type and local ID
pub fn from_global_id(global_id: &str) -> async_graphql::Result<(RelayTy, usize)> {
    let decoded = base64.decode(global_id)?;
    let s = std::str::from_utf8(&decoded)?;
    if let Some((ty, id_str)) = s.split_once(':') {
        let id = id_str.parse::<usize>()?;
        let ty = serde_json::from_str(ty).map_err(|_| {
            async_graphql::Error::new(format!("Invalid format: {} is not a valid json", ty))
        })?;
        Ok((ty, id))
    } else {
        Err("Invalid format: missing colon".into())
    }
}

/// Encode a numeric offset into a cursor string
pub fn offset_to_cursor(offset: usize) -> String {
    to_global_id(RelayTy::Offset, offset)
}

/// Decode a cursor string back into a numeric offset
pub fn cursor_to_offset(cursor: &str) -> async_graphql::Result<usize> {
    let (_, offset) = from_global_id(cursor)?;
    Ok(offset)
}

/// Additional fields on Relay connections (total count)
#[derive(SimpleObject)]
pub struct ConnectionFields {
    /// Total number of items across all pages
    pub total_count: usize,
}

/// Convert Relay pagination arguments into internal pagination parameters
pub fn process_pagination(
    after: Option<String>,
    before: Option<String>,
    first: Option<usize>,
    last: Option<usize>,
) -> async_graphql::Result<PaginationArgs> {
    match (after, before, first, last) {
        (Some(after), _, Some(first), _) => {
            let id = cursor_to_offset(after.as_str())?;
            let cursor = Some(id as i32);
            let direction = Direction::Forward;
            let limit = first;
            Ok(PaginationArgs {
                cursor,
                direction,
                limit,
            })
        }
        (_, Some(before), _, Some(last)) => {
            let id = cursor_to_offset(before.as_str())?;
            let cursor = Some(id as i32);
            let direction = Direction::Backward;
            let limit = last;
            Ok(PaginationArgs {
                cursor,
                direction,
                limit,
            })
        }
        (None, _, Some(first), _) => Ok(PaginationArgs {
            cursor: None,
            direction: Direction::Forward,
            limit: first,
        }),
        (_, None, _, Some(last)) => Ok(PaginationArgs {
            cursor: None,
            direction: Direction::Backward,
            limit: last,
        }),
        (_, _, _, _) => Ok(PaginationArgs {
            cursor: None,
            direction: Direction::Forward,
            limit: 10,
        }),
    }
}
