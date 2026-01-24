use std::sync::Arc;
use crate::schema::album_query::Album;
use crate::schema::image_query::Image;
use async_graphql::{Interface, SimpleObject};
use base64::engine::general_purpose::STANDARD as base64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx_postgres::PgPool;
use crate::model::{Direction, PaginationArgs};
use crate::state::QueueState;

pub type ArcPgPool = Arc<PgPool>;
pub type ArcStates = Arc<QueueState>;

#[derive(Interface)]
#[graphql(
    name = "Node",
    field(name = "id", ty = "String", desc = "The id of the object")
)]
pub enum RelayNode {
    Album(Album),
    Image(Image),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RelayTy {
    Album,
    Image,
    Cbz,
    Offset,
}
pub fn to_global_id(ty: RelayTy, id: usize) -> String {
    let combined = format!("{}:{}", serde_json::to_string(&ty).unwrap(), id);
    base64.encode(combined)
}

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
pub fn offset_to_cursor(offset: usize) -> String {
    to_global_id(RelayTy::Offset, offset)
}
pub fn cursor_to_offset(cursor: &str) -> async_graphql::Result<usize> {
    let (_, offset) = from_global_id(cursor)?;
    Ok(offset)
}

#[derive(SimpleObject)]
pub struct ConnectionFields {
    pub total_count: usize,
}

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