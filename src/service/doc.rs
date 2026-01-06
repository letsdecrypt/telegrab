use crate::model::dto::doc::{CreateDocReq, UpdateDocReq};
use crate::model::dto::pagination::PaginationResponse;
use crate::model::dto::pagination::{PaginationQuery, RefineSortOrder};
use crate::model::entity::doc::{Doc, ShimDoc, TelegraphPost};
use convert_case::{Case, Casing};
use sqlx::{query, query_as};
use sqlx_postgres::PgPool;
use time::OffsetDateTime;

pub async fn create_doc(pool: &PgPool, req: CreateDocReq) -> Result<Doc, sqlx::Error> {
    let sql = "INSERT INTO doc (url) VALUES ($1) RETURNING *, (SELECT id FROM cbz WHERE doc_id = doc.id) AS cbz_id";
    query_as(sql).bind(req.url).fetch_one(pool).await
}
pub async fn get_doc_by_id(pool: &PgPool, id: i32) -> Result<Doc, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id WHERE doc.id = $1";
    query_as(sql).bind(id).fetch_one(pool).await
}

pub async fn get_docs(
    pool: &PgPool,
    query: &PaginationQuery,
) -> Result<PaginationResponse<Doc>, sqlx::Error> {
    // 构建排序子句
    let sort_clause = if let Some(sort) = &query.sort {
        let mut clauses = Vec::new();
        let snake_sort = sort.to_case(Case::Snake);
        if let Some(order) = &query.order {
            let order = match &order {
                RefineSortOrder::Asc => "ASC",
                RefineSortOrder::Desc => "DESC",
            };
            clauses.push(format!("{}.{} {}", "doc", snake_sort, order));
        }
        if !clauses.is_empty() {
            format!(" ORDER BY {}", clauses.join(", "))
        } else {
            // 默认按id降序排序
            " ORDER BY doc.id DESC".to_string()
        }
    } else {
        // 默认按id降序排序
        " ORDER BY doc.id DESC".to_string()
    };

    // 构建分页子句
    let pagination_clause = format!(" LIMIT {} OFFSET {}", query.limit(), query.offset());

    // 执行查询获取总数
    let (total,): (i64,) = query_as("SELECT COUNT(*) FROM doc").fetch_one(pool).await?;

    // 执行查询获取数据
    let docs = query_as(&format!(
        "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id{}{}",
        sort_clause, pagination_clause
    ))
    .fetch_all(pool)
    .await?;

    // 构建并返回分页响应
    Ok(PaginationResponse {
        data: docs,
        total: total as u64,
    })
}

pub async fn get_parsed_docs(pool: &PgPool) -> Result<Vec<ShimDoc>, sqlx::Error> {
    let sql = "SELECT doc.id, cbz.id as cbz_id, url, page_title, title FROM doc left join cbz on doc.id = cbz.doc_id WHERE status > 0 ORDER BY doc.id";
    query_as::<_, ShimDoc>(sql).fetch_all(pool).await
}
pub async fn get_unparsed_docs(pool: &PgPool) -> Result<Vec<Doc>, sqlx::Error> {
    let sql = "SELECT doc.*, cbz.id as cbz_id FROM doc left join cbz on doc.id = cbz.doc_id WHERE status = 0";
    query_as(sql).fetch_all(pool).await
}

pub async fn delete_doc_by_id(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM doc WHERE id = $1";
    query(sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

pub async fn update_doc(pool: &PgPool, id: i32, req: UpdateDocReq) -> Result<Doc, sqlx::Error> {
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

pub async fn update_parsed_doc(
    pool: &PgPool,
    id: i32,
    p: TelegraphPost,
) -> Result<Doc, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let parsed_date = p.date.as_deref().and_then(|date_str| {
        OffsetDateTime::parse(
            date_str,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .ok()
    });
    let doc_sql = r#"UPDATE doc SET page_title = $1, page_date = $2, page_count = $3, web = $4, status = 1 WHERE id = $5 RETURNING *, (SELECT id FROM cbz WHERE doc_id = $5) AS cbz_id"#;
    let doc = query_as(doc_sql)
        .bind(p.title)
        .bind(parsed_date)
        .bind(p.image_urls.len().to_string())
        .bind(p.url.clone())
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    let pic_sql = r#"INSERT INTO pic (doc_id, url, seq) VALUES ($1, $2, $3)"#;
    let check_sql = r#"SELECT COUNT(*) FROM pic WHERE doc_id = $1 and url = $2"#;
    for (i, url) in p.image_urls.iter().enumerate() {
        let (count,): (i64,) = query_as(check_sql)
            .bind(id)
            .bind(url)
            .fetch_one(pool)
            .await?;
        if count == 0 {
            query(pic_sql)
                .bind(id)
                .bind(url)
                .bind(i as i32)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(doc)
}

pub async fn update_doc_status(pool: &PgPool, id: i32, status: i32) -> Result<u64, sqlx::Error> {
    let sql = "UPDATE doc SET status = $1 WHERE id = $2";
    query(sql)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}
