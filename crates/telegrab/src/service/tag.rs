use crate::model::entity::tag::{AlbumTag, Tag, TagWithAlbumCount};
use sqlx::FromRow;
use std::collections::HashMap;
use sqlx::{query, query_as, query_scalar};
use sqlx_postgres::PgPool;
use time::OffsetDateTime;

// Create a new tag
pub async fn create_tag(pool: &PgPool, name: &str, description: Option<&str>) -> Result<Tag, sqlx::Error> {
    let sql = "INSERT INTO tag (name, description) VALUES ($1, $2) RETURNING *";
    query_as(sql)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
}

// Get tag by ID
pub async fn get_tag_by_id(pool: &PgPool, id: i32) -> Result<Tag, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

// Get tag by name
pub async fn get_tag_by_name(pool: &PgPool, name: &str) -> Result<Option<Tag>, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE name = $1";
    query_as(sql).bind(name).fetch_optional(pool).await
}

// Get all tags with album count
pub async fn get_all_tags(pool: &PgPool) -> Result<Vec<TagWithAlbumCount>, sqlx::Error> {
    let sql = r#"
        SELECT t.id, t.name, t.description, t.created_at, t.updated_at,
               COUNT(at.album_id) as album_count
        FROM tag t
        LEFT JOIN album_tag at ON t.id = at.tag_id
        GROUP BY t.id, t.name, t.description, t.created_at, t.updated_at
        ORDER BY t.name ASC
    "#;
    query_as(sql).fetch_all(pool).await
}

// Search tags by name (for autocomplete)
pub async fn search_tags(pool: &PgPool, keyword: &str, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let search_pattern = format!("%{}%", keyword);
    let sql = r#"
        SELECT * FROM tag
        WHERE name ILIKE $1
        ORDER BY name ASC
        LIMIT $2
    "#;
    query_as(sql)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(pool)
        .await
}

// Search tags excluding already associated ones (for album tag selection)
pub async fn search_tags_excluding_album(
    pool: &PgPool,
    keyword: &str,
    album_id: i32,
    limit: i32,
) -> Result<Vec<Tag>, sqlx::Error> {
    let search_pattern = format!("%{}%", keyword);
    let sql = r#"
        SELECT * FROM tag
        WHERE name ILIKE $1
          AND id NOT IN (SELECT tag_id FROM album_tag WHERE album_id = $2)
        ORDER BY name ASC
        LIMIT $3
    "#;
    query_as(sql)
        .bind(&search_pattern)
        .bind(album_id)
        .bind(limit)
        .fetch_all(pool).await
}

// Get recent tags (for album tag selection dropdown)
pub async fn get_recent_tags(pool: &PgPool, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT DISTINCT t.*
        FROM tag t
        JOIN album_tag at ON t.id = at.tag_id
        ORDER BY at.created_at DESC
        LIMIT $1
    "#;
    query_as(sql).bind(limit).fetch_all(pool).await
}

pub async fn get_tags_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE id = ANY($1)";
    query_as(sql).bind(ids).fetch_all(pool).await
}

// Get tags for a single album
pub async fn get_tags_for_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT t.*
        FROM tag t
        JOIN album_tag at ON t.id = at.tag_id
        WHERE at.album_id = $1
        ORDER BY t.name ASC
    "#;
    query_as(sql).bind(album_id).fetch_all(pool).await
}

// Batch load tags for multiple albums (returns album_id -> Vec<Tag>)
pub async fn get_tags_for_albums(pool: &PgPool, album_ids: &[i32]) -> Result<HashMap<i32, Vec<Tag>>, sqlx::Error> {
    #[derive(FromRow)]
    struct AlbumTagRow {
        album_id: i32,
        id: i32,
        name: String,
        description: Option<String>,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    }

    let sql = r#"
        SELECT at.album_id, t.id, t.name, t.description, t.created_at, t.updated_at
        FROM album_tag at
        JOIN tag t ON t.id = at.tag_id
        WHERE at.album_id = ANY($1)
        ORDER BY t.name ASC
    "#;
    let rows: Vec<AlbumTagRow> = sqlx::query_as(sql)
        .bind(album_ids)
        .fetch_all(pool)
        .await?;

    let mut result: HashMap<i32, Vec<Tag>> = album_ids.iter().map(|&id| (id, Vec::new())).collect();
    for row in rows {
        result.entry(row.album_id).or_default().push(Tag {
            id: row.id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }
    Ok(result)
}

// Get tags not associated with specific album
pub async fn get_tags_excluding_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT t.*
        FROM tag t
        WHERE id NOT IN (SELECT tag_id FROM album_tag WHERE album_id = $1)
        ORDER BY t.name ASC
    "#;
    query_as(sql).bind(album_id).fetch_all(pool).await
}

// Get album IDs for a tag (for single tag view page)
pub async fn get_album_ids_for_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    let sql = "SELECT album_id FROM album_tag WHERE tag_id = $1";
    query_scalar(sql).bind(tag_id).fetch_all(pool).await
}

// Update a tag
pub async fn update_tag(
    pool: &PgPool,
    id: i32,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    let sql = r#"
        UPDATE tag
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = now()
        WHERE id = $3
        RETURNING *
    "#;
    query_as(sql)
        .bind(name)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await
}

// Delete a tag (and cascade delete album_tag relationships)
pub async fn delete_tag(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM tag WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

// Associate a tag with an album
pub async fn add_tag_to_album(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<AlbumTag, sqlx::Error> {
    let sql = "INSERT INTO album_tag (album_id, tag_id) VALUES ($1, $2) RETURNING *";
    query_as(sql)
        .bind(album_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await
}

// Remove a tag from an album
pub async fn remove_tag_from_album(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM album_tag WHERE album_id = $1 AND tag_id = $2";
    query(sql)
        .bind(album_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

// Batch load albums for multiple tags (returns tag_id -> Vec<Doc>)
pub async fn get_albums_for_tags(pool: &PgPool, tag_ids: &[i32]) -> Result<HashMap<i32, Vec<crate::model::entity::doc::Doc>>, sqlx::Error> {
    let sql = r#"
        SELECT at.tag_id, doc.*, cbz.id as cbz_id
        FROM album_tag at
        JOIN doc ON at.album_id = doc.id
        LEFT JOIN cbz ON doc.id = cbz.doc_id
        WHERE at.tag_id = ANY($1)
        ORDER BY doc.id ASC
    "#;

    #[derive(FromRow)]
    struct TagAlbumRow {
        tag_id: i32,
        id: i32,
        cbz_id: Option<i32>,
        status: i16,
        url: String,
        page_title: Option<String>,
        page_date: Option<OffsetDateTime>,
        title: Option<String>,
        series: Option<String>,
        number: Option<String>,
        count: Option<String>,
        volume: Option<String>,
        summary: Option<String>,
        notes: Option<String>,
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        writer: Option<String>,
        penciller: Option<String>,
        inker: Option<String>,
        colorist: Option<String>,
        letterer: Option<String>,
        cover_artist: Option<String>,
        editor: Option<String>,
        publisher: Option<String>,
        imprint: Option<String>,
        genre: Option<String>,
        tags: Option<String>,
        web: Option<String>,
        page_count: Option<i16>,
        language: Option<String>,
        format: Option<String>,
        black_and_white: Option<bool>,
        characters: Option<String>,
        teams: Option<String>,
        locations: Option<String>,
        scan_information: Option<String>,
        story_arc: Option<String>,
        series_group: Option<String>,
        age_rating: Option<String>,
        community_rating: Option<String>,
        critical_rating: Option<String>,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    }

    let rows: Vec<TagAlbumRow> = sqlx::query_as(sql)
        .bind(tag_ids)
        .fetch_all(pool)
        .await?;

    let mut result: HashMap<i32, Vec<crate::model::entity::doc::Doc>> =
        tag_ids.iter().map(|&id| (id, Vec::new())).collect();

    for row in rows {
        result.entry(row.tag_id).or_default().push(crate::model::entity::doc::Doc {
            id: row.id,
            cbz_id: row.cbz_id,
            status: row.status,
            url: row.url,
            page_title: row.page_title,
            page_date: row.page_date,
            title: row.title,
            series: row.series,
            number: row.number,
            count: row.count,
            volume: row.volume,
            summary: row.summary,
            notes: row.notes,
            year: row.year,
            month: row.month,
            day: row.day,
            writer: row.writer,
            penciller: row.penciller,
            inker: row.inker,
            colorist: row.colorist,
            letterer: row.letterer,
            cover_artist: row.cover_artist,
            editor: row.editor,
            publisher: row.publisher,
            imprint: row.imprint,
            genre: row.genre,
            tags: row.tags,
            web: row.web,
            page_count: row.page_count,
            language: row.language,
            format: row.format,
            black_and_white: row.black_and_white,
            characters: row.characters,
            teams: row.teams,
            locations: row.locations,
            scan_information: row.scan_information,
            story_arc: row.story_arc,
            series_group: row.series_group,
            age_rating: row.age_rating,
            community_rating: row.community_rating,
            critical_rating: row.critical_rating,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }

    Ok(result)
}

// Check if a tag is already associated with an album
pub async fn album_tag_exists(pool: &PgPool, album_id: i32, tag_id: i32) -> Result<bool, sqlx::Error> {
    let sql = "SELECT EXISTS(SELECT 1 FROM album_tag WHERE album_id = $1 AND tag_id = $2)";
    let exists: bool = query_scalar(sql)
        .bind(album_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await?;
    Ok(exists)
}

// Check if tag name already exists
pub async fn tag_name_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    let sql = "SELECT EXISTS(SELECT 1 FROM tag WHERE name = $1)";
    let exists: bool = query_scalar(sql)
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(exists)
}

// Check if tag name exists excluding a specific tag ID (for updates)
pub async fn tag_name_exists_excluding(
    pool: &PgPool,
    name: &str,
    exclude_id: i32,
) -> Result<bool, sqlx::Error> {
    let sql = "SELECT EXISTS(SELECT 1 FROM tag WHERE name = $1 AND id != $2)";
    let exists: bool = query_scalar(sql)
        .bind(name)
        .bind(exclude_id)
        .fetch_one(pool)
        .await?;
    Ok(exists)
}
