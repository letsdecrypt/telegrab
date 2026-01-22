use crate::controller::pic::PicQuery;
use crate::model::dto::pagination::RefineSortOrder;
use crate::model::dto::pagination::{PaginationQuery, PaginationResponse};
use crate::model::dto::pic::MutatePicReq;
use crate::model::entity::pic::Pic;
use convert_case::{Case, Casing};
use sqlx::{query, query_as};
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
