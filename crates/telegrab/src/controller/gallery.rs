use crate::schema::{create_schema, GallerySchema};
use crate::state::AppState;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    response::{self, IntoResponse},
    routing::{get, post_service},
    Router,
};
use axum::extract::State;
use axum::http::header;

pub fn routers(state: &AppState) -> Router<AppState> {
    let schema = create_schema(state.db_pool.clone(), state.queue_state.clone());
    let graphql_service = GraphQL::new(schema.clone());
    let subscription_service = GraphQLSubscription::new(schema.clone());
    Router::new()
        .route("/", get(graphiql))
        .route("/", post_service(graphql_service))
        .route_service("/ws", subscription_service)
        .route("/schema", get(export_schema))
        .with_state(schema)
}
async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql/ws")
            .finish(),
    )
}
async fn export_schema(State(schema):State<GallerySchema>)-> impl IntoResponse{
    let sdl_content = schema.sdl();
    let response_headers = [
        (header::CONTENT_TYPE, "application/graphql; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"schema.graphql\"",
        ),
        (header::CONTENT_ENCODING, "utf-8"),
    ];

    // 组合「响应头 + SDL内容」返回
    (response_headers, sdl_content)
}