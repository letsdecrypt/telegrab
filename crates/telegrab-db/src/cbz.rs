use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use sqlx_postgres::PgPool;
use telegrab_model::entity::cbz::Cbz;

pub async fn create(pool: &PgPool, path: String) -> Result<Cbz, sqlx::Error> {
    let sql = "INSERT INTO cbz (path) VALUES ($1) RETURNING *";
    query_as(sql).bind(path).fetch_one(pool).await
}

pub async fn create_with_doc_id(
    pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<Cbz, sqlx::Error> {
    let sql = "INSERT INTO cbz (doc_id, path) VALUES ($1, $2) RETURNING *";
    query_as(sql).bind(doc_id).bind(path).fetch_one(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Cbz, sqlx::Error> {
    let sql = "SELECT * FROM cbz WHERE id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

pub async fn find_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Option<Cbz>, sqlx::Error> {
    let sql = "SELECT * FROM cbz WHERE doc_id = $1";
    query_as(sql).bind(doc_id).fetch_optional(pool).await
}

pub async fn find_by_path(pool: &PgPool, path: String) -> Result<Option<Cbz>, sqlx::Error> {
    let sql = "SELECT * FROM cbz WHERE path = $1";
    query_as(sql).bind(path).fetch_optional(pool).await
}

pub async fn find_page(
    pool: &PgPool,
    sort_clause: &str,
    pagination_clause: &str,
) -> Result<Vec<Cbz>, sqlx::Error> {
    let sql = AssertSqlSafe(format!(
        "SELECT * FROM cbz{}{}",
        sort_clause, pagination_clause
    ));
    query_as(sql).fetch_all(pool).await
}

pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let sql = "SELECT COUNT(*) FROM cbz";
    query_scalar(sql).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: i32, doc_id: Option<i32>) -> Result<Cbz, sqlx::Error> {
    let sql = "UPDATE cbz SET doc_id = $1 WHERE id = $2 RETURNING *";
    query_as(sql).bind(doc_id).bind(id).fetch_one(pool).await
}

pub async fn update_doc_id_by_path(
    pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<u64, sqlx::Error> {
    let sql = "UPDATE cbz SET doc_id = $1 WHERE path = $2";
    query(sql)
        .bind(doc_id)
        .bind(path)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn delete_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM cbz WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}
