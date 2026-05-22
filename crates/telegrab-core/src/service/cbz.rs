use convert_case::{Case, Casing};
use sqlx_postgres::PgPool;
use telegrab_db as repository;
use telegrab_model::dto::pagination::{PaginationQuery, PaginationResponse, RefineSortOrder};
use telegrab_model::entity::cbz::Cbz;

pub async fn create_cbz(db_pool: &PgPool, path: String) -> Result<Cbz, sqlx::Error> {
    repository::cbz::create(db_pool, path).await
}

pub async fn create_cbz_with_doc_id(
    db_pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<Cbz, sqlx::Error> {
    repository::cbz::create_with_doc_id(db_pool, doc_id, path).await
}

pub async fn get_cbz_by_id(db_pool: &PgPool, id: i32) -> Result<Cbz, sqlx::Error> {
    repository::cbz::find_by_id(db_pool, id).await
}

pub async fn get_cbz_by_doc_id(db_pool: &PgPool, doc_id: i32) -> Result<Option<Cbz>, sqlx::Error> {
    repository::cbz::find_by_doc_id(db_pool, doc_id).await
}

pub async fn get_cbz_by_path(db_pool: &PgPool, path: String) -> Result<Option<Cbz>, sqlx::Error> {
    repository::cbz::find_by_path(db_pool, path).await
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
            " ORDER BY cbz.id DESC".to_string()
        }
    } else {
        " ORDER BY cbz.id DESC".to_string()
    };

    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    let total = repository::cbz::count(pool).await?;
    let cbz_v = repository::cbz::find_page(pool, &sort_clause, &pagination_clause).await?;

    Ok(PaginationResponse {
        data: cbz_v,
        total: total as u64,
    })
}

pub async fn update_cbz(
    db_pool: &PgPool,
    id: i32,
    doc_id: Option<i32>,
) -> Result<Cbz, sqlx::Error> {
    repository::cbz::update(db_pool, id, doc_id).await
}

pub async fn update_cbz_doc_id_with_path(
    db_pool: &PgPool,
    doc_id: i32,
    path: String,
) -> Result<u64, sqlx::Error> {
    repository::cbz::update_doc_id_by_path(db_pool, doc_id, path).await
}

pub async fn remove_cbz_by_id(db_pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    repository::cbz::delete_by_id(db_pool, id).await
}
