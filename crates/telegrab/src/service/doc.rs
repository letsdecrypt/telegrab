use crate::model::dto::doc::{CreateDocReq, UpdateDocReq};
use crate::model::dto::pagination::{CursorBasedPaginationResponse, PaginationResponse};
use crate::model::dto::pagination::{PaginationQuery, RefineSortOrder};
use crate::model::entity::doc::{Doc, TelegraphPost};
use crate::model::{Direction, PaginationArgs, SortOrder};
use crate::repository;
use crate::service::helper::build_cursor_pagination;
use sqlx_postgres::PgPool;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy)]
enum DocSortColumn {
    Id,
    Url,
    PageTitle,
    Title,
    Status,
    PageCount,
    CreatedAt,
    UpdatedAt,
}

impl DocSortColumn {
    fn column_name(&self) -> &'static str {
        match self {
            Self::Id => "doc.id",
            Self::Url => "doc.url",
            Self::PageTitle => "doc.page_title",
            Self::Title => "doc.title",
            Self::Status => "doc.status",
            Self::PageCount => "doc.page_count",
            Self::CreatedAt => "doc.created_at",
            Self::UpdatedAt => "doc.updated_at",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('_', "") {
            s if s == "id" => Some(Self::Id),
            s if s == "url" => Some(Self::Url),
            s if s == "pagetitle" => Some(Self::PageTitle),
            s if s == "title" => Some(Self::Title),
            s if s == "status" => Some(Self::Status),
            s if s == "pagecount" => Some(Self::PageCount),
            s if s == "createdat" => Some(Self::CreatedAt),
            s if s == "updatedat" => Some(Self::UpdatedAt),
            _ => None,
        }
    }
}

pub async fn create_doc(pool: &PgPool, req: CreateDocReq) -> Result<Doc, sqlx::Error> {
    repository::doc::create(pool, &req.url).await
}

pub async fn get_doc_by_id(pool: &PgPool, id: i32) -> Result<Doc, sqlx::Error> {
    repository::doc::find_by_id(pool, id).await
}

pub async fn get_random_doc(pool: &PgPool) -> Result<Doc, sqlx::Error> {
    repository::doc::find_random(pool).await
}

pub async fn get_docs_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Doc>, sqlx::Error> {
    repository::doc::find_by_ids(pool, ids).await
}

pub async fn get_docs(
    pool: &PgPool,
    query: &PaginationQuery,
) -> Result<PaginationResponse<Doc>, sqlx::Error> {
    let sort_clause = if let Some(sort) = &query.sort {
        let order = match &query.order {
            Some(RefineSortOrder::Asc) => "ASC",
            Some(RefineSortOrder::Desc) => "DESC",
            None => "DESC",
        };
        match DocSortColumn::from_str(sort) {
            Some(col) => format!(" ORDER BY {} {}", col.column_name(), order),
            None => {
                tracing::warn!("Invalid sort parameter ignored: {}", sort);
                " ORDER BY doc.id DESC".to_string()
            }
        }
    } else {
        " ORDER BY doc.id DESC".to_string()
    };

    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    let total = repository::doc::count(pool).await?;
    let docs = repository::doc::find_page(pool, &sort_clause, &pagination_clause).await?;

    Ok(PaginationResponse {
        data: docs,
        total: total as u64,
    })
}

pub async fn get_parsed_docs(pool: &PgPool) -> Result<Vec<crate::model::entity::doc::ShimDoc>, sqlx::Error> {
    repository::doc::find_parsed(pool).await
}

pub async fn get_unparsed_docs(pool: &PgPool) -> Result<Vec<Doc>, sqlx::Error> {
    repository::doc::find_unparsed(pool).await
}

pub async fn delete_doc_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    repository::doc::delete_by_id(pool, id).await
}

pub async fn update_doc(pool: &PgPool, id: i32, req: UpdateDocReq) -> Result<Doc, sqlx::Error> {
    repository::doc::update(pool, id, req).await
}

pub async fn update_parsed_doc(
    pool: &PgPool,
    id: i32,
    p: TelegraphPost,
) -> Result<Doc, sqlx::Error> {
    let parsed_date = p.date.as_deref().and_then(|date_str| {
        OffsetDateTime::parse(
            date_str,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .ok()
    });
    let urls: Vec<&str> = p.image_urls.iter().map(|s| s.as_str()).collect();
    let seqs: Vec<i32> = (0..p.image_urls.len() as i32).collect();

    repository::doc::update_parsed(
        pool,
        id,
        p.title,
        parsed_date,
        p.image_urls.len() as i16,
        p.url,
        &urls,
        &seqs,
    )
    .await
}

pub async fn update_doc_status(pool: &PgPool, id: i32, status: i16) -> Result<u64, sqlx::Error> {
    repository::doc::update_status(pool, id, status).await
}

pub async fn get_cursor_based_pagination_docs(
    pool: &PgPool,
    pagination_args: PaginationArgs,
    sort_order: SortOrder,
    _title: Option<String>,
) -> Result<CursorBasedPaginationResponse<Doc>, sqlx::Error> {
    let total = repository::doc::count(pool).await?;
    let PaginationArgs {
        limit,
        cursor,
        direction,
    } = pagination_args;

    let order_by = match (sort_order, direction) {
        (SortOrder::Asc, Direction::Forward) => "ORDER BY doc.id ASC",
        (SortOrder::Asc, Direction::Backward) => "ORDER BY doc.id DESC",
        (SortOrder::Desc, Direction::Forward) => "ORDER BY doc.id DESC",
        (SortOrder::Desc, Direction::Backward) => "ORDER BY doc.id ASC",
    };

    let docs = if let Some(cursor) = cursor {
        let where_op = match (sort_order, direction) {
            (SortOrder::Asc, Direction::Forward) => " > ",
            (SortOrder::Asc, Direction::Backward) => " < ",
            (SortOrder::Desc, Direction::Forward) => " < ",
            (SortOrder::Desc, Direction::Backward) => " > ",
        };
        repository::doc::find_cursor_with_cursor(pool, where_op, cursor, limit as i64 + 1, order_by).await?
    } else {
        repository::doc::find_cursor_no_cursor(pool, limit as i64 + 1, order_by).await?
    };

    let paged = build_cursor_pagination(docs, total as u64, limit, direction, cursor.is_some());
    Ok(paged)
}

pub async fn search_docs_by_keyword(
    pool: &PgPool,
    keyword: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Doc>, sqlx::Error> {
    let pattern = format!("%{}%", keyword);
    repository::doc::search_by_keyword(pool, &pattern, limit, offset).await
}

pub async fn count_docs_by_keyword(pool: &PgPool, keyword: &str) -> Result<i64, sqlx::Error> {
    let pattern = format!("%{}%", keyword);
    repository::doc::count_by_keyword(pool, &pattern).await
}
