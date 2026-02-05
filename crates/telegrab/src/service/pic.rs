use crate::controller::pic::PicQuery;
use crate::model::dto::pagination::{CursorBasedPaginationResponse, RefineSortOrder};
use crate::model::dto::pagination::{PaginationQuery, PaginationResponse};
use crate::model::dto::pic::MutatePicReq;
use crate::model::entity::pic::Pic;
use crate::model::{Direction, PaginationArgs};
use crate::service::helper::build_cursor_pagination;
use convert_case::{Case, Casing};
use sqlx::{query, query_as, query_scalar};
use sqlx_postgres::PgPool;

pub async fn create_pic(pool: &PgPool, params: MutatePicReq) -> Result<Pic, sqlx::Error> {
    let sql = "INSERT INTO pic (url, doc_id, seq) VALUES ($1, $2, $3) RETURNING *";
    query_as(sql)
        .bind(params.url)
        .bind(params.doc_id)
        .bind(params.seq)
        .fetch_one(pool)
        .await
}

pub async fn get_pic_by_id(pool: &PgPool, id: i32) -> Result<Pic, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}
pub async fn get_cover_pic_by_doc_id(pool: &PgPool, doc_id: i32)->Result<Pic, sqlx::Error>{
    let sql = "SELECT * FROM pic WHERE doc_id = $1 and seq = 0 ORDER BY seq LIMIT 1";
    query_as(sql).bind(doc_id).fetch_one(pool).await
}
pub async fn get_pics_by_ids(
    pool: &PgPool,
    ids: &[i32],
) -> Result<Vec<Pic>, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE id = ANY($1)";
    query_as(sql).bind(ids).fetch_all(pool).await
}

pub async fn get_pics(
    pool: &PgPool,
    query: &PaginationQuery,
    pic_query: &PicQuery,
) -> Result<PaginationResponse<Pic>, sqlx::Error> {
    // 构建排序子句
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
            // 默认按id降序排序
            " ORDER BY pic.id DESC".to_string()
        }
    } else {
        // 默认按id降序排序
        " ORDER BY pic.id DESC".to_string()
    };
    // 构建过滤子句
    let filter_clause = if let Some(doc_id) = &pic_query.doc_id {
        format!(" WHERE pic.doc_id = {}", doc_id)
    } else {
        "".to_string()
    };

    // 构建分页子句
    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    // 执行查询获取总数
    let (total,): (i64,) = query_as(&format!("SELECT COUNT(*) FROM pic{}", filter_clause))
        .fetch_one(pool)
        .await?;

    // 执行查询获取数据
    let pics = query_as(&format!(
        "SELECT * FROM pic{}{}{}",
        filter_clause, sort_clause, pagination_clause
    ))
    .fetch_all(pool)
    .await?;

    // 构建并返回分页响应
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
    let sql = "UPDATE pic SET url = $1, doc_id = $2, seq = $3 WHERE id = $4 RETURNING *";
    query_as(sql)
        .bind(params.url)
        .bind(params.doc_id)
        .bind(params.seq)
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn update_pic_status_by_id(
    pool: &PgPool,
    id: i32,
    status: i16,
) -> Result<Pic, sqlx::Error> {
    let sql = "UPDATE pic SET status = $1 WHERE id = $2 RETURNING *";
    query_as(sql)
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn delete_pic_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM pic WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn get_pics_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<Vec<Pic>, sqlx::Error> {
    let sql = "SELECT * FROM pic WHERE doc_id = $1 ORDER BY seq";
    query_as(sql).bind(doc_id).fetch_all(pool).await
}
pub async fn has_status_0_pics_by_doc_id(pool: &PgPool, doc_id: i32) -> Result<bool, sqlx::Error> {
    let sql = r#"SELECT EXISTS(SELECT 1 FROM pic WHERE doc_id = $1 AND status == 0 ORDER BY seq) AS "exists: bool""#;
    query_scalar(sql).bind(doc_id).fetch_one(pool).await
}

pub async fn get_cursor_based_pagination_pics(
    pool: &PgPool,
    pagination_args: PaginationArgs,
    doc_id: i32,
) -> Result<CursorBasedPaginationResponse<Pic>, sqlx::Error> {
    let total: i64 = query_scalar("SELECT COUNT(*) from pic WHERE doc_id = $1")
        .bind(doc_id)
        .fetch_one(pool)
        .await?;
    let PaginationArgs {
        limit,
        cursor,
        direction,
    } = pagination_args;
    let main_sql = "SELECT * FROM pic WHERE doc_id = $1";
    let order_by_clause = match direction {
        Direction::Forward => "ORDER BY seq",
        Direction::Backward => "ORDER BY seq DESC",
    };
    let pics = if let Some(cursor) = cursor {
        let where_clause = format!(
            "AND id {} $2",
            if direction == Direction::Forward {
                " > "
            } else {
                " < "
            }
        );
        let sql = format!("{} {} {} LIMIT $3", main_sql, where_clause, order_by_clause);
        query_as(&sql)
            .bind(doc_id)
            .bind(cursor)
            .bind(limit as i64 + 1)
            .fetch_all(pool)
            .await?
    } else {
        let sql = format!("{} {} LIMIT $2", main_sql, order_by_clause);
        query_as(&sql)
            .bind(doc_id)
            .bind(limit as i64 + 1)
            .fetch_all(pool)
            .await?
    };
    let paged = build_cursor_pagination(pics, total as u64, limit, direction, cursor.is_some());
    Ok(paged)
}
