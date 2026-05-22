use sqlx::{FromRow, query, query_as, query_scalar};
use sqlx_postgres::PgPool;
use telegrab_model::entity::doc::Doc;
use telegrab_model::entity::tag::{AlbumTag, Tag, TagWithAlbumCount};
use time::OffsetDateTime;

#[derive(FromRow)]
pub struct AlbumTagRow {
    pub album_id: i32,
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(FromRow)]
pub struct TagAlbumRow {
    pub tag_id: i32,
    pub id: i32,
    pub cbz_id: Option<i32>,
    pub status: i16,
    pub url: String,
    pub page_title: Option<String>,
    pub page_date: Option<OffsetDateTime>,
    pub title: Option<String>,
    pub series: Option<String>,
    pub number: Option<String>,
    pub count: Option<String>,
    pub volume: Option<String>,
    pub summary: Option<String>,
    pub notes: Option<String>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub inker: Option<String>,
    pub colorist: Option<String>,
    pub letterer: Option<String>,
    pub cover_artist: Option<String>,
    pub editor: Option<String>,
    pub publisher: Option<String>,
    pub imprint: Option<String>,
    pub genre: Option<String>,
    pub tags: Option<String>,
    pub web: Option<String>,
    pub page_count: Option<i16>,
    pub language: Option<String>,
    pub format: Option<String>,
    pub black_and_white: Option<bool>,
    pub characters: Option<String>,
    pub teams: Option<String>,
    pub locations: Option<String>,
    pub scan_information: Option<String>,
    pub story_arc: Option<String>,
    pub series_group: Option<String>,
    pub age_rating: Option<String>,
    pub community_rating: Option<String>,
    pub critical_rating: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<AlbumTagRow> for Tag {
    fn from(row: AlbumTagRow) -> Self {
        Tag {
            id: row.id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<TagAlbumRow> for Doc {
    fn from(row: TagAlbumRow) -> Self {
        Doc {
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
        }
    }
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    let sql = "INSERT INTO tag (name, description) VALUES ($1, $2) RETURNING *";
    query_as(sql)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
}

pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Tag, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Tag>, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE name = $1";
    query_as(sql).bind(name).fetch_optional(pool).await
}

pub async fn find_all_with_count(pool: &PgPool) -> Result<Vec<TagWithAlbumCount>, sqlx::Error> {
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

pub async fn search_by_name(
    pool: &PgPool,
    pattern: &str,
    limit: i32,
) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT * FROM tag
        WHERE name ILIKE $1
        ORDER BY name ASC
        LIMIT $2
    "#;
    query_as(sql)
        .bind(pattern)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn search_excluding_album(
    pool: &PgPool,
    pattern: &str,
    album_id: i32,
    limit: i32,
) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT * FROM tag
        WHERE name ILIKE $1
          AND id NOT IN (SELECT tag_id FROM album_tag WHERE album_id = $2)
        ORDER BY name ASC
        LIMIT $3
    "#;
    query_as(sql)
        .bind(pattern)
        .bind(album_id)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn find_recent(pool: &PgPool, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT DISTINCT t.*
        FROM tag t
        JOIN album_tag at ON t.id = at.tag_id
        ORDER BY at.created_at DESC
        LIMIT $1
    "#;
    query_as(sql).bind(limit).fetch_all(pool).await
}

pub async fn find_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = "SELECT * FROM tag WHERE id = ANY($1)";
    query_as(sql).bind(ids).fetch_all(pool).await
}

pub async fn find_for_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT t.*
        FROM tag t
        JOIN album_tag at ON t.id = at.tag_id
        WHERE at.album_id = $1
        ORDER BY t.name ASC
    "#;
    query_as(sql).bind(album_id).fetch_all(pool).await
}

pub async fn find_rows_for_albums(
    pool: &PgPool,
    album_ids: &[i32],
) -> Result<Vec<AlbumTagRow>, sqlx::Error> {
    let sql = r#"
        SELECT at.album_id, t.id, t.name, t.description, t.created_at, t.updated_at
        FROM album_tag at
        JOIN tag t ON t.id = at.tag_id
        WHERE at.album_id = ANY($1)
        ORDER BY t.name ASC
    "#;
    query_as::<_, AlbumTagRow>(sql)
        .bind(album_ids)
        .fetch_all(pool)
        .await
}

pub async fn find_excluding_album(pool: &PgPool, album_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let sql = r#"
        SELECT t.*
        FROM tag t
        WHERE id NOT IN (SELECT tag_id FROM album_tag WHERE album_id = $1)
        ORDER BY t.name ASC
    "#;
    query_as(sql).bind(album_id).fetch_all(pool).await
}

pub async fn find_album_ids_for_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    let sql = "SELECT album_id FROM album_tag WHERE tag_id = $1";
    query_scalar(sql).bind(tag_id).fetch_all(pool).await
}

pub async fn find_album_rows_for_tags(
    pool: &PgPool,
    tag_ids: &[i32],
) -> Result<Vec<TagAlbumRow>, sqlx::Error> {
    let sql = r#"
        SELECT at.tag_id, doc.*, cbz.id as cbz_id
        FROM album_tag at
        JOIN doc ON at.album_id = doc.id
        LEFT JOIN cbz ON doc.id = cbz.doc_id
        WHERE at.tag_id = ANY($1)
        ORDER BY doc.id ASC
    "#;
    query_as::<_, TagAlbumRow>(sql)
        .bind(tag_ids)
        .fetch_all(pool)
        .await
}

pub async fn update(
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

pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM tag WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn add_to_album(
    pool: &PgPool,
    album_id: i32,
    tag_id: i32,
) -> Result<AlbumTag, sqlx::Error> {
    let sql = "INSERT INTO album_tag (album_id, tag_id) VALUES ($1, $2) RETURNING *";
    query_as(sql)
        .bind(album_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await
}

pub async fn remove_from_album(
    pool: &PgPool,
    album_id: i32,
    tag_id: i32,
) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM album_tag WHERE album_id = $1 AND tag_id = $2";
    query(sql)
        .bind(album_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn album_tag_exists(
    pool: &PgPool,
    album_id: i32,
    tag_id: i32,
) -> Result<bool, sqlx::Error> {
    let sql = "SELECT EXISTS(SELECT 1 FROM album_tag WHERE album_id = $1 AND tag_id = $2)";
    let exists: bool = query_scalar(sql)
        .bind(album_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await?;
    Ok(exists)
}

pub async fn name_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    let sql = "SELECT EXISTS(SELECT 1 FROM tag WHERE name = $1)";
    let exists: bool = query_scalar(sql).bind(name).fetch_one(pool).await?;
    Ok(exists)
}

pub async fn name_exists_excluding(
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
