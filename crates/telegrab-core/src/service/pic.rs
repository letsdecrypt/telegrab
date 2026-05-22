use crate::service::helper::build_cursor_pagination;
use convert_case::{Case, Casing};
use sqlx_postgres::PgPool;
use telegrab_db as repository;
use telegrab_model::dto::pagination::{CursorBasedPaginationResponse, RefineSortOrder};
use telegrab_model::dto::pagination::{PaginationQuery, PaginationResponse};
use telegrab_model::dto::pic::MutatePicReq;
use telegrab_model::dto::pic::PicQuery;
use telegrab_model::entity::pic::Pic;
use telegrab_model::{Direction, PaginationArgs};

pub async fn create_pic(pool: &PgPool, params: MutatePicReq) -> Result<Pic, sqlx::Error> {
    repository::pic::create(pool, params.url, params.doc_id, params.seq).await
}

pub async fn get_pic_by_id(pool: &PgPool, id: i32) -> Result<Pic, sqlx::Error> {
    repository::pic::find_by_id(pool, id).await
}

pub async fn get_cover_pic_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Pic, sqlx::Error> {
    repository::pic::find_cover_by_doc_id(pool, doc_id).await
}

pub async fn get_pics_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Pic>, sqlx::Error> {
    repository::pic::find_by_ids(pool, ids).await
}

pub async fn get_pics(
    pool: &PgPool,
    query: &PaginationQuery,
    pic_query: &PicQuery,
) -> Result<PaginationResponse<Pic>, sqlx::Error> {
    let sort_clause = if let Some(sort) = &query.sort {
        let mut clauses = Vec::new();
        let snake_sort = sort.to_case(Case::Snake);
        if let Some(order) = &query.order {
            let order = match &order {
                RefineSortOrder::Asc => "ASC",
                RefineSortOrder::Desc => "DESC",
            };
            clauses.push(format!("{}.{} {}", "pic", snake_sort, order));
        }
        if !clauses.is_empty() {
            format!(" ORDER BY {}", clauses.join(", "))
        } else {
            " ORDER BY pic.id DESC".to_string()
        }
    } else {
        " ORDER BY pic.id DESC".to_string()
    };

    let filter_clause = if let Some(doc_id) = &pic_query.doc_id {
        format!(" WHERE pic.doc_id = {}", doc_id)
    } else {
        "".to_string()
    };

    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    let total = repository::pic::count(pool, &filter_clause).await?;
    let pics =
        repository::pic::find_page(pool, &filter_clause, &sort_clause, &pagination_clause).await?;

    Ok(PaginationResponse {
        data: pics,
        total: total as u64,
    })
}

pub async fn update_pic_by_id(
    pool: &PgPool,
    id: i32,
    params: MutatePicReq,
) -> Result<Pic, sqlx::Error> {
    repository::pic::update_by_id(pool, id, params.url, params.doc_id, params.seq).await
}

pub async fn update_pic_status_by_id(
    pool: &PgPool,
    id: i32,
    status: i16,
) -> Result<Pic, sqlx::Error> {
    repository::pic::update_status_by_id(pool, id, status).await
}

pub async fn delete_pic_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    repository::pic::delete_by_id(pool, id).await
}

pub async fn get_pics_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Vec<Pic>, sqlx::Error> {
    repository::pic::find_by_doc_id(pool, doc_id).await
}

pub async fn has_status_0_pics_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<bool, sqlx::Error> {
    repository::pic::exists_status_0_by_doc_id(pool, doc_id).await
}

pub async fn get_cursor_based_pagination_pics(
    pool: &PgPool,
    pagination_args: PaginationArgs,
    doc_id: i32,
) -> Result<CursorBasedPaginationResponse<Pic>, sqlx::Error> {
    let total = repository::pic::count_by_doc_id(pool, doc_id).await?;
    let PaginationArgs {
        limit,
        cursor,
        direction,
    } = pagination_args;

    let order_by = match direction {
        Direction::Forward => "ORDER BY seq",
        Direction::Backward => "ORDER BY seq DESC",
    };

    let pics = if let Some(cursor) = cursor {
        let where_op = if direction == Direction::Forward {
            " > "
        } else {
            " < "
        };
        repository::pic::find_cursor_with_cursor(
            pool,
            doc_id,
            where_op,
            cursor,
            limit as i64 + 1,
            order_by,
        )
        .await?
    } else {
        repository::pic::find_cursor_no_cursor(pool, doc_id, limit as i64 + 1, order_by).await?
    };

    let paged = build_cursor_pagination(pics, total as u64, limit, direction, cursor.is_some());
    Ok(paged)
}
