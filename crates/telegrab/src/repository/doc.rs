use crate::model::dto::doc::UpdateDocReq;
use crate::model::entity::doc::{Doc, ShimDoc};
use sqlx::{query, query_as, query_scalar};
use sqlx_postgres::PgPool;
use time::OffsetDateTime;

pub async fn create(pool: &PgPool, url: &str) -> Result<Doc, sqlx::Error> {
    let sql = "INSERT INTO doc (url) VALUES ($1) RETURNING *, (SELECT id FROM cbz WHERE doc_id = doc.id) AS cbz_id";
    query_as(sql).bind(url).fetch_one(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Doc, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id WHERE doc.id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

pub async fn find_random(pool: &PgPool) -> Result<Doc, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id ORDER BY RANDOM() LIMIT 1";
    query_as(sql).fetch_one(pool).await
}

pub async fn find_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Doc>, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id WHERE doc.id = ANY($1)";
    query_as(sql).bind(ids).fetch_all(pool).await
}

pub async fn find_page(
    pool: &PgPool,
    sort_clause: &str,
    pagination_clause: &str,
) -> Result<Vec<Doc>, sqlx::Error> {
    let sql = format!(
        "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id{}{}",
        sort_clause, pagination_clause
    );
    query_as(&sql).fetch_all(pool).await
}

pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let sql = "SELECT COUNT(*) FROM doc";
    query_scalar(sql).fetch_one(pool).await
}

pub async fn find_parsed(pool: &PgPool) -> Result<Vec<ShimDoc>, sqlx::Error> {
    let sql = "SELECT doc.id, cbz.id as cbz_id, url, page_title, title FROM doc left join cbz on doc.id = cbz.doc_id WHERE status > 0 ORDER BY doc.id";
    query_as::<_, ShimDoc>(sql).fetch_all(pool).await
}

pub async fn find_unparsed(pool: &PgPool) -> Result<Vec<Doc>, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id WHERE status = 0";
    query_as(sql).fetch_all(pool).await
}

pub async fn delete_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM doc WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn update(pool: &PgPool, id: i32, req: UpdateDocReq) -> Result<Doc, sqlx::Error> {
    let sql = r#"UPDATE doc
    SET page_title = $1,
        page_date = $2,
        title = $3,
        series = $4,
        number = $5,
        count = $6,
        volume = $7,
        summary = $8,
        notes = $9,
        year = $10,
        month = $11,
        day = $12,
        writer = $13,
        penciller = $14,
        inker = $15,
        colorist = $16,
        letterer = $17,
        cover_artist = $18,
        editor = $19,
        publisher = $20,
        imprint = $21,
        genre = $22,
        tags = $23,
        web = $24,
        page_count = $25,
        language = $26,
        format = $27,
        black_and_white = $28,
        characters = $29,
        teams = $30,
        locations = $31,
        scan_information = $32,
        story_arc = $33,
        series_group = $34,
        age_rating = $35,
        community_rating = $36,
        critical_rating = $37,
        updated_at = now()
    WHERE id = $38
    RETURNING *, (SELECT id FROM cbz WHERE doc_id = doc.id) AS cbz_id
    "#;

    query_as(sql)
        .bind(req.page_title)
        .bind(req.page_date)
        .bind(req.title)
        .bind(req.series)
        .bind(req.number)
        .bind(req.count)
        .bind(req.volume)
        .bind(req.summary)
        .bind(req.notes)
        .bind(req.year)
        .bind(req.month)
        .bind(req.day)
        .bind(req.writer)
        .bind(req.penciller)
        .bind(req.inker)
        .bind(req.colorist)
        .bind(req.letterer)
        .bind(req.cover_artist)
        .bind(req.editor)
        .bind(req.publisher)
        .bind(req.imprint)
        .bind(req.genre)
        .bind(req.tags)
        .bind(req.web)
        .bind(req.page_count)
        .bind(req.language)
        .bind(req.format)
        .bind(req.black_and_white)
        .bind(req.characters)
        .bind(req.teams)
        .bind(req.locations)
        .bind(req.scan_information)
        .bind(req.story_arc)
        .bind(req.series_group)
        .bind(req.age_rating)
        .bind(req.community_rating)
        .bind(req.critical_rating)
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn update_parsed(
    pool: &PgPool,
    id: i32,
    title: String,
    parsed_date: Option<OffsetDateTime>,
    page_count: i16,
    web: String,
    urls: &[&str],
    seqs: &[i32],
) -> Result<Doc, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let doc_sql = r#"UPDATE doc SET page_title = $1, page_date = $2, page_count = $3, web = $4, status = 1 WHERE id = $5 RETURNING *, (SELECT id FROM cbz WHERE doc_id = $5) AS cbz_id"#;
    let doc = query_as(doc_sql)
        .bind(title)
        .bind(parsed_date)
        .bind(page_count)
        .bind(web)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    let pic_sql = r#"INSERT INTO pic (doc_id, url, seq)
        SELECT $1, t.url, t.seq FROM UNNEST($2::text[], $3::int[]) AS t(url, seq)
        ON CONFLICT (doc_id, url, seq) DO NOTHING"#;
    query(pic_sql)
        .bind(id)
        .bind(urls)
        .bind(seqs)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(doc)
}

pub async fn update_status(pool: &PgPool, id: i32, status: i16) -> Result<u64, sqlx::Error> {
    let sql = "UPDATE doc SET status = $1 WHERE id = $2";
    query(sql)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn find_cursor_with_cursor(
    pool: &PgPool,
    where_op: &str,
    cursor: i32,
    limit: i64,
    order_by: &str,
) -> Result<Vec<Doc>, sqlx::Error> {
    let main_sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id";
    let where_clause = format!("WHERE doc.id {} $1", where_op);
    let sql = format!("{} {} {} LIMIT $2", main_sql, where_clause, order_by);
    query_as(&sql)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn find_cursor_no_cursor(
    pool: &PgPool,
    limit: i64,
    order_by: &str,
) -> Result<Vec<Doc>, sqlx::Error> {
    let main_sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id";
    let sql = format!("{} {} LIMIT $1", main_sql, order_by);
    query_as(&sql).bind(limit).fetch_all(pool).await
}

pub async fn search_by_keyword(
    pool: &PgPool,
    pattern: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Doc>, sqlx::Error> {
    let sql = r#"
        SELECT doc.*, cbz.id as cbz_id
        FROM doc
        LEFT JOIN cbz ON doc.id = cbz.doc_id
        WHERE doc.page_title ILIKE $1 OR doc.title ILIKE $1
        ORDER BY doc.id DESC
        LIMIT $2 OFFSET $3
    "#;
    query_as(sql)
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn count_by_keyword(pool: &PgPool, pattern: &str) -> Result<i64, sqlx::Error> {
    let sql = r#"
        SELECT COUNT(*)
        FROM doc
        WHERE doc.page_title ILIKE $1 OR doc.title ILIKE $1
    "#;
    query_scalar(sql).bind(pattern).fetch_one(pool).await
}
