use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use forge_adapters::{local_knowledge_base, mock_adaptive_patterns};
use forge_application::ApplicationEngine;
use forge_core::{EngineerRequest, PromptPipeline};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "forge=info,tower_http=info".into()),
        )
        .init();

    let pipeline = PromptPipeline::new(local_knowledge_base());
    let app = ApplicationEngine::new(pipeline);

    let router = Router::new()
        .route("/", get(index))
        .route("/api/engineer", post(engineer))
        .route("/api/patterns", get(patterns))
        .route("/api/history", get(history))
        .route("/api/stats", get(stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!(%addr, "starting Forge HTTP server");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>FORGE - Rust Prompt Engineer</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 760px; margin: 48px auto; line-height: 1.6; padding: 0 20px; }
    code { background: #f3f4f6; padding: 2px 6px; border-radius: 4px; }
  </style>
</head>
<body>
  <h1>FORGE</h1>
  <p><strong>Rust Prompt Engineer</strong> transforma pedidos ambiguos em prompts estruturados usando um pipeline local.</p>
  <p>Use <code>POST /api/engineer</code> com <code>{ "input": "...", "provider": "claude" }</code>. Nenhuma API key e exigida.</p>
  <p>Tambem ha <code>GET /api/patterns</code>, <code>GET /api/history?limit=20</code> e <code>GET /api/stats</code>.</p>
</body>
</html>"#,
    )
}

async fn engineer(
    State(app): State<ApplicationEngine>,
    Json(request): Json<EngineerRequest>,
) -> Result<Json<forge_core::EngineerResponse>, impl IntoResponse> {
    if request.input.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "input must not be empty"));
    }

    Ok(Json(app.engineer(request)))
}

async fn patterns() -> Json<forge_adapters::AdaptivePatternProfile> {
    Json(mock_adaptive_patterns())
}

async fn history(
    State(app): State<ApplicationEngine>,
    Query(query): Query<HistoryQuery>,
) -> Json<Vec<forge_application::HistoryItem>> {
    Json(app.history(query.limit.unwrap_or(20)))
}

async fn stats(State(app): State<ApplicationEngine>) -> Json<forge_application::EngineStats> {
    Json(app.stats())
}
