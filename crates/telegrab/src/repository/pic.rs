use crate::model::entity::pic::Pic;
use sqlx::{query, query_as, query_scalar};
use sqlx_postgres::PgPool;

pub async fn create(pool: &PgPool, url: String, doc_id: i32, seq: i32) -> Result<Pic, sqlx::Error> {
    let sql = "INSERT INTO pic (url, doc_id, seq) VALUES ($1, $2, $3) RETURNING *";
    query_as(sql)
        .bind(url)
        .bind(doc_id)
        .bind(seq)
        .fetch_one(pool)
        .await
}

pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Pic, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

pub async fn find_cover_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Pic, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE doc_id = $1 and seq = 0 ORDER BY seq LIMIT 1";
    query_as(sql).bind(doc_id).fetch_one(pool).await
}

pub async fn find_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Pic>, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE id = ANY($1)";
    query_as(sql).bind(ids).fetch_all(pool).await
}

pub async fn find_page(
    pool: &PgPool,
    filter_clause: &str,
    sort_clause: &str,
    pagination_clause: &str,
) -> Result<Vec<Pic>, sqlx::Error> {
    let sql = format!(
        "SELECT * FROM pic{}{}{}",
        filter_clause, sort_clause, pagination_clause
    );
    query_as(&sql).fetch_all(pool).await
}

pub async fn count(pool: &PgPool, filter_clause: &str) -> Result<i64, sqlx::Error> {
    let sql = format!("SELECT COUNT(*) FROM pic{}", filter_clause);
    query_scalar(&sql).fetch_one(pool).await
}

pub async fn update_by_id(
    pool: &PgPool,
    id: i32,
    url: String,
    doc_id: i32,
    seq: i32,
) -> Result<Pic, sqlx::Error> {
    let sql = "UPDATE pic SET url = $1, doc_id = $2, seq = $3 WHERE id = $4 RETURNING *";
    query_as(sql)
        .bind(url)
        .bind(doc_id)
        .bind(seq)
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn update_status_by_id(pool: &PgPool, id: i32, status: i16) -> Result<Pic, sqlx::Error> {
    let sql = "UPDATE pic SET status = $1 WHERE id = $2 RETURNING *";
    query_as(sql).bind(status).bind(id).fetch_one(pool).await
}

pub async fn delete_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM pic WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn find_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Vec<Pic>, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE doc_id = $1 ORDER BY seq";
    query_as(sql).bind(doc_id).fetch_all(pool).await
}

pub async fn exists_status_0_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<bool, sqlx::Error> {
    let sql = r#"SELECT EXISTS(SELECT 1 FROM pic WHERE doc_id = $1 AND status = 0 ORDER BY seq) AS "exists: bool""#;
    query_scalar(sql).bind(doc_id).fetch_one(pool).await
}

pub async fn find_cursor_with_cursor(
    pool: &PgPool,
    doc_id: i32,
    where_op: &str,
    cursor: i32,
    limit: i64,
    order_by: &str,
) -> Result<Vec<Pic>, sqlx::Error> {
    let main_sql = "SELECT * FROM pic WHERE doc_id = $1";
    let where_clause = format!("AND id {} $2", where_op);
    let sql = format!("{} {} {} LIMIT $3", main_sql, where_clause, order_by);
    query_as(&sql)
        .bind(doc_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn find_cursor_no_cursor(
    pool: &PgPool,
    doc_id: i32,
    limit: i64,
    order_by: &str,
) -> Result<Vec<Pic>, sqlx::Error> {
    let main_sql = "SELECT * FROM pic WHERE doc_id = $1";
    let sql = format!("{} {} LIMIT $2", main_sql, order_by);
    query_as(&sql)
        .bind(doc_id)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn count_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<i64, sqlx::Error> {
    let sql = "SELECT COUNT(*) from pic WHERE doc_id = $1";
    query_scalar(sql).bind(doc_id).fetch_one(pool).await
}
