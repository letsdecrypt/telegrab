use crate::model::dto::pagination::{PaginationQuery, PaginationResponse, RefineSortOrder};
use crate::model::entity::cbz::Cbz;
use convert_case::{Case, Casing};
use sqlx::PgPool;

pub async fn create_cbz(db_pool: &PgPool, path: String) -> Result<Cbz, sqlx::Error> {
    let query = "INSERT INTO cbz (path) VALUES ($1) RETURNING *";
    sqlx::query_as::<_, Cbz>(query)
        .bind(path)
        .fetch_one(db_pool)
        .await
}

pub async fn create_cbz_with_doc_id(
    db_pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<Cbz, sqlx::Error> {
    let query = "INSERT INTO cbz (doc_id, path) VALUES ($1, $2) RETURNING *";
    sqlx::query_as::<_, Cbz>(query)
        .bind(doc_id)
        .bind(path)
        .fetch_one(db_pool)
        .await
}

pub async fn get_cbz_by_id(db_pool: &PgPool, id: i32) -> Result<Cbz, sqlx::Error> {
    let query = "SELECT * FROM cbz WHERE id = $1";
    sqlx::query_as::<_, Cbz>(query)
        .bind(id)
        .fetch_one(db_pool)
        .await
}

pub async fn get_cbz_by_doc_id(db_pool: &PgPool, doc_id: i32) -> Result<Option<Cbz>, sqlx::Error> {
    let query = "SELECT * FROM cbz WHERE doc_id = $1";
    sqlx::query_as::<_, Cbz>(query)
        .bind(doc_id)
        .fetch_optional(db_pool)
        .await
}

pub async fn get_cbz_by_path(db_pool: &PgPool, path: String) -> Result<Option<Cbz>, sqlx::Error> {
    let query = "SELECT * FROM cbz WHERE path = $1";
    sqlx::query_as::<_, Cbz>(query)
        .bind(path)
        .fetch_optional(db_pool)
        .await
}

pub async fn get_cbz_page(
    pool: &PgPool,
    query: &PaginationQuery,
) -> Result<PaginationResponse<Cbz>, sqlx::Error> {
    let sort_clause = if let Some(sort) = &query.sort {
        let mut clauses = Vec::new();
        let snake_sort = sort.to_case(Case::Snake);
        if let Some(order) = &query.order {
            let order = match &order {
                RefineSortOrder::Asc => "ASC",
                RefineSortOrder::Desc => "DESC",
            };
            clauses.push(format!("{}.{} {}", "cbz", snake_sort, order));
        }
        if !clauses.is_empty() {
            format!(" ORDER BY {}", clauses.join(", "))
        } else {
            // 默认按id降序排序
            " ORDER BY cbz.id DESC".to_string()
        }
    } else {
        // 默认按id降序排序
        " ORDER BY cbz.id DESC".to_string()
    };

    // 构建分页子句
    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    // 执行查询获取总数
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cbz")
        .fetch_one(pool)
        .await?;

    // 执行查询获取数据
    let cbz_v = sqlx::query_as::<_, Cbz>(&format!(
        "SELECT * FROM cbz{}{}",
        sort_clause, pagination_clause
    ))
    .fetch_all(pool)
    .await?;

    // 构建并返回分页响应
    Ok(PaginationResponse {
        data: cbz_v,
        total: total.0 as u64,
    })
}

pub async fn update_cbz(
    db_pool: &PgPool,
    id: i32,
    doc_id: Option<i32>,
) -> Result<Cbz, sqlx::Error> {
    let query = "UPDATE cbz SET doc_id = $1 WHERE id = $2 RETURNING *";
    sqlx::query_as::<_, Cbz>(query)
        .bind(doc_id)
        .bind(id)
        .fetch_one(db_pool)
        .await
}

pub async fn update_cbz_doc_id_with_path(
    db_pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<u64, sqlx::Error> {
    let query = "UPDATE cbz SET doc_id = $1 WHERE path = $2";
    sqlx::query(query)
        .bind(doc_id)
        .bind(path)
        .execute(db_pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn remove_cbz_by_id(db_pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let query = "DELETE FROM cbz WHERE id = $1";
    sqlx::query(query)
        .bind(id)
        .execute(db_pool)
        .await
        .map(|r| r.rows_affected())
}
