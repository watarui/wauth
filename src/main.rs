use crate::schema::{build_schema, AppSchema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod application;
mod domain;
mod infrastructure;
mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // トレーシングの初期化
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // TOTPApplicationのインスタンスを作成
    let app = application::TOTPApplication::new().await?;

    // GraphQLスキーマを構築
    let schema = build_schema(app);

    // CORSの設定
    let cors = CorsLayer::permissive();

    // Axumルーターを設定
    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/graphql", post(graphql_handler)) // /graphql エンドポイントを明示的に追加
        .route("/health", get(health_check))
        .layer(Extension(schema))
        .layer(cors);

    // サーバーを起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("GraphQL server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(include_str!("graphql.html"))
}

async fn health_check() -> impl IntoResponse {
    "OK"
}
