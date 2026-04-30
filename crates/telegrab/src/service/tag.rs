use crate::model::entity::tag::{AlbumTag, Tag, TagWithAlbumCount};
use crate::repository;
use sqlx_postgres::PgPool;
use std::collections::HashMap;

pub async fn create_tag(pool: &PgPool, name: &str, description: Option<&str>) -> Result<Tag, sqlx::Error> {
    repository::tag::create(pool, name, description).await
}

pub async fn get_tag_by_id(pool: &PgPool, id: i32) -> Result<Tag, sqlx::Error> {
    repository::tag::find_by_id(pool, id).await
}

pub async fn get_tag_by_name(pool: &PgPool, name: &str) -> Result<Option<Tag>, sqlx::Error> {
    repository::tag::find_by_name(pool, name).await
}

pub async fn get_all_tags(pool: &PgPool) -> Result<Vec<TagWithAlbumCount>, sqlx::Error> {
    repository::tag::find_all_with_count(pool).await
}

pub async fn search_tags(pool: &PgPool, keyword: &str, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let pattern = format!("%{}%", keyword);
    repository::tag::search_by_name(pool, &pattern, limit).await
}

pub async fn search_tags_excluding_album(
    pool: &PgPool,
    keyword: &str,
    album_id: i32,
    limit: i32,
) -> Result<Vec<Tag>, sqlx::Error> {
    let pattern = format!("%{}%", keyword);
    repository::tag::search_excluding_album(pool, &pattern, album_id, limit).await
}

pub async fn get_recent_tags(pool: &PgPool, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    repository::tag::find_recent(pool, limit).await
}

pub async fn get_tags_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Tag>, sqlx::Error> {
    repository::tag::find_by_ids(pool, ids).await
}

pub async fn get_tags_for_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    repository::tag::find_for_album(pool, album_id).await
}

pub async fn get_tags_for_albums(pool: &PgPool, album_ids: &[i32]) -> Result<HashMap<i32, Vec<Tag>>, sqlx::Error> {
    let rows = repository::tag::find_rows_for_albums(pool, album_ids).await?;
    let mut result: HashMap<i32, Vec<Tag>> = album_ids.iter().map(|&id| (id, Vec::new())).collect();
    for row in rows {
        result.entry(row.album_id).or_default().push(row.into());
    }
    Ok(result)
}

pub async fn get_tags_excluding_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    repository::tag::find_excluding_album(pool, album_id).await
}

pub async fn get_album_ids_for_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    repository::tag::find_album_ids_for_tag(pool, tag_id).await
}

pub async fn update_tag(
    pool: &PgPool,
    id: i32,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    repository::tag::update(pool, id, name, description).await
}

pub async fn delete_tag(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    repository::tag::delete(pool, id).await
}

pub async fn add_tag_to_album(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<AlbumTag, sqlx::Error> {
    repository::tag::add_to_album(pool, album_id, tag_id).await
}

pub async fn remove_tag_from_album(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<u64, sqlx::Error> {
    repository::tag::remove_from_album(pool, album_id, tag_id).await
}

pub async fn get_albums_for_tags(pool: &PgPool, tag_ids: &[i32]) -> Result<HashMap<i32, Vec<crate::model::entity::doc::Doc>>, sqlx::Error> {
    let rows = repository::tag::find_album_rows_for_tags(pool, tag_ids).await?;
    let mut result: HashMap<i32, Vec<crate::model::entity::doc::Doc>> =
        tag_ids.iter().map(|&id| (id, Vec::new())).collect();
    for row in rows {
        result.entry(row.tag_id).or_default().push(row.into());
    }
    Ok(result)
}

pub async fn album_tag_exists(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<bool, sqlx::Error> {
    repository::tag::album_tag_exists(pool, album_id, tag_id).await
}

pub async fn tag_name_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    repository::tag::name_exists(pool, name).await
}

pub async fn tag_name_exists_excluding(
    pool: &PgPool,
    name: &str,
    exclude_id: i32,
) -> Result<bool, sqlx::Error> {
    repository::tag::name_exists_excluding(pool, name, exclude_id).await
}
