use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use forge_adapters::{local_knowledge_base, mock_adaptive_patterns};
use forge_application::ApplicationEngine;
use forge_core::{EngineerRequest, PromptPipeline};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    engine: ApplicationEngine,
    db: SqlitePool,
    http: Client,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct EngineerQuery {
    record: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UpdateProfileRequest {
    expertise_level: String,
    domain: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProvidersResponse {
    providers: Vec<ProviderStatus>,
    first_run: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderStatus {
    id: String,
    name: String,
    available: bool,
    configured: bool,
    is_local: bool,
    models: Vec<String>,
    free: bool,
}

#[derive(Debug, Deserialize)]
struct SaveKeyRequest {
    provider: String,
    key: String,
}

#[derive(Debug, Serialize)]
struct SaveKeyResponse {
    provider: String,
    saved: bool,
    valid: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    prompt: String,
    provider: String,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
    provider_used: String,
    model_used: String,
    tokens_used: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FeedbackRequest {
    input: String,
    rating: i8,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
    providers: ProvidersResponse,
}

#[derive(Debug, Serialize)]
struct UsageResponse {
    total_requests: u32,
    total_tokens: u32,
    estimated_cost_usd: f32,
    estimated_cost_brl: f32,
    by_provider: Vec<ProviderUsage>,
}

#[derive(Debug, Serialize)]
struct ProviderUsage {
    provider: String,
    requests: u32,
    tokens: u32,
    estimated_cost_usd: f32,
    note: String,
}

#[derive(Debug, Deserialize)]
struct InstallModelRequest {
    model: String,
}

#[derive(Debug, Serialize)]
struct InstallModelResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct LeadDnaRequest {
    name: String,
    email: String,
    whatsapp: String,
    dna: String,
}

#[derive(Debug, Serialize)]
struct LeadDnaResponse {
    saved: bool,
    dna: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GenerateFileRequest {
    filename: Option<String>,
    file_type: String,
    title: Option<String>,
    content: String,
}

#[derive(Debug, Serialize)]
struct GenerateFileResponse {
    success: bool,
    filename: String,
    url: String,
    mime_type: String,
    message: String,
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
    let engine = ApplicationEngine::new(pipeline);
    let db = open_database().await?;
    run_schema(&db).await?;
    let http = Client::builder().timeout(Duration::from_secs(300)).build()?;
    let state = AppState { engine, db, http };

    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let router = Router::new()
        .route("/api/engineer", post(engineer))
        .route("/api/profile", get(get_profile).post(update_profile))
        .route("/api/providers", get(providers))
        .route("/api/providers/key", post(save_provider_key))
        .route("/api/chat", post(chat))
        .route("/api/feedback", post(feedback))
        .route("/api/files/generate", post(generate_file))
        .route("/api/leads/dna", post(save_lead_dna))
        .route("/api/install/ollama-model", post(install_ollama_model))
        .route("/api/health", get(health))
        .route("/api/patterns", get(patterns))
        .route("/api/history", get(history))
        .route("/api/stats", get(stats))
        .route("/api/usage", get(usage))
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!(%addr, "starting Forge HTTP server");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn open_database() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        std::env::var("FORGE_DATABASE_URL").unwrap_or_else(|_| "sqlite://forge.sqlite".to_string());
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

async fn run_schema(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_keys (
            provider   TEXT PRIMARY KEY,
            api_key    TEXT NOT NULL,
            validated  BOOLEAN DEFAULT FALSE,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_history (
            id           TEXT PRIMARY KEY,
            input        TEXT NOT NULL,
            prompt_used  TEXT NOT NULL,
            response     TEXT NOT NULL,
            provider     TEXT NOT NULL,
            model        TEXT NOT NULL,
            tokens_used  INTEGER,
            rating       INTEGER,
            created_at   DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS learning_signals (
            id         TEXT PRIMARY KEY,
            input      TEXT NOT NULL,
            rating     INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS lead_dna (
            id         TEXT PRIMARY KEY,
            dna        TEXT NOT NULL UNIQUE,
            name       TEXT NOT NULL,
            email      TEXT NOT NULL,
            whatsapp   TEXT NOT NULL,
            source     TEXT NOT NULL DEFAULT 'landing',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn engineer(
    State(state): State<AppState>,
    Query(query): Query<EngineerQuery>,
    Json(request): Json<EngineerRequest>,
) -> Result<Json<forge_core::EngineerResponse>, impl IntoResponse> {
    if request.input.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "input must not be empty"));
    }

    let response = if query.record.unwrap_or(true) {
        state.engine.engineer(request)
    } else {
        state.engine.preview(request)
    };
    Ok(Json(response))
}

async fn get_profile(State(state): State<AppState>) -> Json<forge_application::ProfileResponse> {
    Json(state.engine.profile())
}

async fn update_profile(
    State(state): State<AppState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<forge_application::ProfileResponse>, impl IntoResponse> {
    if request.expertise_level.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "expertise_level must not be empty"));
    }

    Ok(Json(
        state
            .engine
            .update_profile(&request.expertise_level, request.domain),
    ))
}

async fn providers(State(state): State<AppState>) -> Json<ProvidersResponse> {
    Json(load_providers(&state).await)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let providers = load_providers(&state).await;
    Json(HealthResponse {
        ok: true,
        providers,
    })
}

async fn save_lead_dna(
    State(state): State<AppState>,
    Json(request): Json<LeadDnaRequest>,
) -> Result<Json<LeadDnaResponse>, impl IntoResponse> {
    let name = request.name.trim();
    let email = request.email.trim().to_lowercase();
    let whatsapp = request.whatsapp.trim();
    let dna = request.dna.trim().to_uppercase();

    if name.len() < 3
        || !email.contains('@')
        || whatsapp.chars().filter(|character| character.is_ascii_digit()).count() < 10
        || !dna.starts_with("FORGE-DNA-")
    {
        return Err((StatusCode::BAD_REQUEST, "lead DNA invalido"));
    }

    sqlx::query(
        r#"
        INSERT INTO lead_dna (id, dna, name, email, whatsapp, source)
        VALUES (?1, ?2, ?3, ?4, ?5, 'landing')
        ON CONFLICT(dna) DO UPDATE SET
            name = excluded.name,
            email = excluded.email,
            whatsapp = excluded.whatsapp
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&dna)
    .bind(name)
    .bind(&email)
    .bind(whatsapp)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save lead DNA"))?;

    Ok(Json(LeadDnaResponse {
        saved: true,
        dna,
        message: "DNA Forge gravado com sucesso.".to_string(),
    }))
}

async fn install_ollama_model(Json(request): Json<InstallModelRequest>) -> Json<InstallModelResponse> {
    let model = request.model.trim();
    let allowed_models = [
        "phi3:mini",
        "llama3.2:3b",
        "llama3.1:8b",
        "llama3.2:1b",
        "mistral:7b",
        "deepseek-r1:7b",
        "qwen2.5-coder:7b",
        "codellama:7b",
        "gemma2:9b",
    ];

    if !allowed_models.contains(&model) {
        return Json(InstallModelResponse {
            success: false,
            message: format!("Modelo '{model}' nao permitido."),
        });
    }

    match Command::new("ollama").args(["pull", model]).spawn() {
        Ok(_) => Json(InstallModelResponse {
            success: true,
            message: format!("Download do modelo {model} iniciado."),
        }),
        Err(error) => Json(InstallModelResponse {
            success: false,
            message: format!("Ollama nao encontrado: {error}. Instale primeiro."),
        }),
    }
}

async fn save_provider_key(
    State(state): State<AppState>,
    Json(request): Json<SaveKeyRequest>,
) -> Result<Json<SaveKeyResponse>, impl IntoResponse> {
    let provider = request.provider.trim().to_lowercase();
    let key = request.key.trim().to_string();

    if key.is_empty()
        || !matches!(
            provider.as_str(),
            "claude"
                | "groq"
                | "openai"
                | "gemini"
                | "openrouter"
                | "mistral"
                | "deepseek"
                | "together"
                | "cerebras"
                | "huggingface"
        )
    {
        return Err((StatusCode::BAD_REQUEST, "invalid provider or empty key"));
    }

    let valid = validate_provider_key(&state.http, &provider, &key).await;
    let saved = sqlx::query(
        r#"
        INSERT INTO provider_keys (provider, api_key, validated, updated_at)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
        ON CONFLICT(provider) DO UPDATE SET
            api_key = excluded.api_key,
            validated = excluded.validated,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&provider)
    .bind(&key)
    .bind(valid)
    .execute(&state.db)
    .await
    .is_ok();

    let message = if saved && valid {
        "API key salva e validada.".to_string()
    } else if saved {
        "API key salva, mas a validação remota falhou.".to_string()
    } else {
        "Não foi possível salvar a API key.".to_string()
    };

    Ok(Json(SaveKeyResponse {
        provider,
        saved,
        valid,
        message,
    }))
}

async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    if request.prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "prompt must not be empty".to_string()));
    }

    let provider = request.provider.trim().to_lowercase();
    let result: Result<ChatResponse, String> = match provider.as_str() {
        "ollama" => call_ollama(&state.http, &request.prompt, request.model.as_deref())
            .await
            .map_err(ollama_error_message),
        "claude" | "groq" | "openai" | "gemini" | "openrouter" | "mistral" | "deepseek"
        | "together" | "cerebras" | "huggingface" => {
            let Some(key) = provider_key(&state.db, &provider).await else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("{provider} ainda não está configurado. Abra Configurações, cole a API key oficial e escolha um modelo."),
                ));
            };
            call_remote_provider(
                &state.http,
                &provider,
                &key,
                &request.prompt,
                request.model.as_deref(),
            )
            .await
            .map_err(|error| format!("Falha ao chamar {provider}: {error}"))
        }
        _ => return Err((StatusCode::BAD_REQUEST, "unsupported provider".to_string())),
    };

    let response = result.map_err(|error| (StatusCode::BAD_GATEWAY, error))?;
    let _ = sqlx::query(
        r#"
        INSERT INTO chat_history (id, input, prompt_used, response, provider, model, tokens_used)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("")
    .bind(&request.prompt)
    .bind(&response.response)
    .bind(&response.provider_used)
    .bind(&response.model_used)
    .bind(response.tokens_used.map(|value| value as i64))
    .execute(&state.db)
    .await;

    Ok(Json(response))
}

async fn feedback(
    State(state): State<AppState>,
    Json(request): Json<FeedbackRequest>,
) -> Result<StatusCode, impl IntoResponse> {
    if !matches!(request.rating, -1 | 1) {
        return Err((StatusCode::BAD_REQUEST, "rating must be 1 or -1"));
    }

    sqlx::query(
        r#"
        INSERT INTO learning_signals (id, input, rating)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(request.input)
    .bind(request.rating as i64)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save feedback"))?;

    Ok(StatusCode::OK)
}

async fn generate_file(
    Json(request): Json<GenerateFileRequest>,
) -> Result<Json<GenerateFileResponse>, (StatusCode, String)> {
    if request.content.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "content must not be empty".to_string()));
    }

    let Some((extension, mime_type)) = file_type_info(&request.file_type) else {
        return Err((
            StatusCode::BAD_REQUEST,
            "tipo de arquivo nao suportado".to_string(),
        ));
    };

    let base_name = request
        .filename
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| request.title.as_deref().unwrap_or("forge-arquivo"));
    let safe_name = sanitize_filename(base_name);
    let filename = format!("{safe_name}-{id}.{extension}", id = Uuid::new_v4());
    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .join("generated");
    fs::create_dir_all(&generated_dir).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("falha ao criar pasta de arquivos: {error}"),
        )
    })?;

    let title = request.title.as_deref().unwrap_or("Arquivo gerado pelo Forge");
    let content = render_file_content(&request.file_type, title, &request.content);
    let path = generated_dir.join(&filename);
    fs::write(&path, content).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("falha ao gravar arquivo: {error}"),
        )
    })?;

    Ok(Json(GenerateFileResponse {
        success: true,
        filename: filename.clone(),
        url: format!("/generated/{filename}"),
        mime_type: mime_type.to_string(),
        message: "Arquivo gerado com sucesso.".to_string(),
    }))
}

fn file_type_info(file_type: &str) -> Option<(&'static str, &'static str)> {
    match file_type.trim().to_lowercase().as_str() {
        "txt" => Some(("txt", "text/plain; charset=utf-8")),
        "md" | "markdown" => Some(("md", "text/markdown; charset=utf-8")),
        "html" => Some(("html", "text/html; charset=utf-8")),
        "json" => Some(("json", "application/json; charset=utf-8")),
        "csv" => Some(("csv", "text/csv; charset=utf-8")),
        "svg" => Some(("svg", "image/svg+xml; charset=utf-8")),
        "doc" | "word" => Some(("doc", "application/msword; charset=utf-8")),
        "xls" | "excel" => Some(("xls", "application/vnd.ms-excel; charset=utf-8")),
        "pdf" | "pdf-html" => Some(("html", "text/html; charset=utf-8")),
        _ => None,
    }
}

fn render_file_content(file_type: &str, title: &str, content: &str) -> String {
    let clean_content = clean_artifact_markdown(content);
    match file_type.trim().to_lowercase().as_str() {
        "pdf" | "pdf-html" | "html" | "doc" | "word" => {
            designed_document_html(title, &clean_content)
        }
        "xls" | "excel" => styled_spreadsheet_html(title, &clean_content),
        "json" => json!({
            "title": title,
            "generated_by": "FORGE",
            "content": clean_content,
        })
        .to_string(),
        "csv" => content_to_csv(&clean_content),
        "svg" => content_to_svg(title, &clean_content),
        _ => clean_content,
    }
}

fn clean_artifact_markdown(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn designed_document_html(title: &str, content: &str) -> String {
    let body = content_lines_to_html(content);
    format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    :root {{
      --ember: #FF4500;
      --ink: #151515;
      --muted: #5C5C5C;
      --paper: #FFFFFF;
      --soft: #FFF7F2;
      --line: #E8E2DC;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      background: #F3F0EB;
      color: var(--ink);
      font-family: "Segoe UI", Inter, Arial, sans-serif;
      line-height: 1.65;
    }}
    .page {{
      max-width: 820px;
      margin: 32px auto;
      background: var(--paper);
      border: 1px solid var(--line);
      border-radius: 18px;
      overflow: hidden;
      box-shadow: 0 18px 50px rgba(0,0,0,0.08);
    }}
    .cover {{
      padding: 42px 48px 34px;
      background: linear-gradient(135deg, #111 0%, #1E1E1E 55%, #2A1208 100%);
      color: #FFF;
    }}
    .cover-badge {{
      display: inline-block;
      font: 600 11px/1 "Segoe UI", sans-serif;
      letter-spacing: 0.14em;
      text-transform: uppercase;
      color: var(--ember);
      border: 1px solid rgba(255,69,0,0.35);
      background: rgba(255,69,0,0.12);
      padding: 6px 12px;
      border-radius: 999px;
      margin-bottom: 18px;
    }}
    .cover h1 {{
      margin: 0 0 10px;
      font-size: clamp(28px, 4vw, 40px);
      line-height: 1.15;
      font-weight: 800;
      letter-spacing: -0.02em;
    }}
    .cover p {{
      margin: 0;
      color: rgba(255,255,255,0.72);
      font-size: 15px;
      max-width: 620px;
    }}
    .content {{
      padding: 36px 48px 44px;
    }}
    .section-title {{
      margin: 28px 0 12px;
      font-size: 20px;
      line-height: 1.25;
      color: var(--ember);
      font-weight: 800;
      letter-spacing: -0.01em;
      padding-bottom: 8px;
      border-bottom: 2px solid rgba(255,69,0,0.18);
    }}
    .section-title:first-child {{ margin-top: 0; }}
    .doc-paragraph {{
      margin: 0 0 14px;
      font-size: 15px;
      color: #2A2A2A;
    }}
    .doc-list, .doc-ol {{
      margin: 0 0 16px;
      padding-left: 22px;
    }}
    .doc-list li, .doc-ol li {{
      margin: 0 0 8px;
      font-size: 15px;
    }}
    .callout {{
      margin: 18px 0;
      padding: 16px 18px;
      border-radius: 12px;
      background: var(--soft);
      border: 1px solid rgba(255,69,0,0.16);
    }}
    .callout strong {{ color: var(--ember); }}
    .footer {{
      padding: 16px 48px 22px;
      border-top: 1px solid var(--line);
      color: var(--muted);
      font-size: 12px;
      display: flex;
      justify-content: space-between;
      gap: 12px;
      flex-wrap: wrap;
    }}
    .footer strong {{ color: var(--ember); }}
    @media print {{
      body {{ background: #FFF; }}
      .page {{ margin: 0; border: 0; border-radius: 0; box-shadow: none; }}
      .cover {{ break-after: avoid; }}
    }}
  </style>
</head>
<body>
  <article class="page">
    <header class="cover">
      <div class="cover-badge">Documento Forge</div>
      <h1>{title}</h1>
      <p>Entrega final pronta para apresentar, compartilhar ou imprimir como PDF.</p>
    </header>
    <main class="content">
      {body}
    </main>
    <footer class="footer">
      <span>Gerado por <strong>FORGE</strong> · Rust Prompt Engineer</span>
      <span>Use Imprimir → Salvar como PDF no navegador</span>
    </footer>
  </article>
</body>
</html>"#,
        title = escape_html(title),
        body = body
    )
}

fn content_lines_to_html(content: &str) -> String {
    let mut html = String::new();
    let mut in_ul = false;
    let mut in_ol = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            continue;
        }

        if let Some(text) = trimmed.strip_prefix("### ") {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            html.push_str(&format!(
                "<h3 class=\"section-title\">{}</h3>\n",
                format_inline_html(text)
            ));
            continue;
        }
        if let Some(text) = trimmed.strip_prefix("## ") {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            html.push_str(&format!(
                "<h2 class=\"section-title\">{}</h2>\n",
                format_inline_html(text)
            ));
            continue;
        }
        if let Some(text) = trimmed.strip_prefix("# ") {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            html.push_str(&format!(
                "<h2 class=\"section-title\">{}</h2>\n",
                format_inline_html(text)
            ));
            continue;
        }
        if let Some(text) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            if !in_ul {
                close_html_lists(&mut html, &mut in_ul, &mut in_ol);
                html.push_str("<ul class=\"doc-list\">\n");
                in_ul = true;
            }
            html.push_str(&format!("<li>{}</li>\n", format_inline_html(text)));
            continue;
        }
        if let Some(text) = parse_numbered_list_item(trimmed) {
            if !in_ol {
                close_html_lists(&mut html, &mut in_ul, &mut in_ol);
                html.push_str("<ol class=\"doc-ol\">\n");
                in_ol = true;
            }
            html.push_str(&format!("<li>{}</li>\n", format_inline_html(text)));
            continue;
        }
        if is_section_header(trimmed) {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            let label = trimmed.trim_end_matches(':');
            html.push_str(&format!(
                "<h2 class=\"section-title\">{}</h2>\n",
                format_inline_html(label)
            ));
            continue;
        }
        if trimmed.starts_with("> ") {
            close_html_lists(&mut html, &mut in_ul, &mut in_ol);
            html.push_str(&format!(
                "<div class=\"callout\">{}</div>\n",
                format_inline_html(trimmed.trim_start_matches("> "))
            ));
            continue;
        }

        close_html_lists(&mut html, &mut in_ul, &mut in_ol);
        html.push_str(&format!(
            "<p class=\"doc-paragraph\">{}</p>\n",
            format_inline_html(trimmed)
        ));
    }

    close_html_lists(&mut html, &mut in_ul, &mut in_ol);
    if html.trim().is_empty() {
        html.push_str("<p class=\"doc-paragraph\">Conteúdo gerado pelo Forge.</p>\n");
    }
    html
}

fn parse_numbered_list_item(line: &str) -> Option<&str> {
    let digit_len = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_len == 0 {
        return None;
    }
    let rest = line[digit_len..].trim_start();
    rest.strip_prefix(". ")
        .or_else(|| rest.strip_prefix('.'))
        .map(str::trim)
}

fn close_html_lists(html: &mut String, in_ul: &mut bool, in_ol: &mut bool) {
    if *in_ul {
        html.push_str("</ul>\n");
        *in_ul = false;
    }
    if *in_ol {
        html.push_str("</ol>\n");
        *in_ol = false;
    }
}

fn is_section_header(line: &str) -> bool {
    let lower = line.to_lowercase();
    let keywords = [
        "ingredientes",
        "modo de preparo",
        "preparo",
        "instruções",
        "instrucoes",
        "introdução",
        "introducao",
        "conclusão",
        "conclusao",
        "resumo",
        "benefícios",
        "beneficios",
        "dicas",
        "variações",
        "variacoes",
        "informações nutricionais",
        "informacoes nutricionais",
        "tempo de preparo",
        "rendimento",
        "passo a passo",
        "materiais",
        "escopo",
        "objetivo",
        "metodologia",
        "resultados",
        "anexos",
        "cláusulas",
        "clausulas",
        "partes",
        "vigência",
        "vigencia",
    ];
    keywords.iter().any(|keyword| {
        lower == *keyword
            || lower.starts_with(&format!("{keyword}:"))
            || lower.starts_with(&format!("{keyword} -"))
    }) || (line.len() <= 60 && line.ends_with(':'))
}

fn format_inline_html(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("**") {
        output.push_str(&escape_html(&rest[..start]));
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("**") {
            output.push_str("<strong>");
            output.push_str(&escape_html(&rest[..end]));
            output.push_str("</strong>");
            rest = &rest[end + 2..];
        } else {
            output.push_str("**");
            break;
        }
    }
    output.push_str(&escape_html(rest));
    output
}

fn styled_spreadsheet_html(title: &str, content: &str) -> String {
    let rows: Vec<Vec<String>> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.trim().chars().all(|character| matches!(character, '|' | '-' | ':' | ' ')))
        .map(|line| {
            split_table_cells(line)
                .into_iter()
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect();

    let (header, body_rows) = if rows.is_empty() {
        (vec!["Coluna".to_string(), "Conteúdo".to_string()], vec![])
    } else {
        let header = rows[0].clone();
        (header, rows[1..].to_vec())
    };

    let header_html = header
        .iter()
        .map(|cell| format!("<th>{}</th>", escape_html(cell)))
        .collect::<Vec<_>>()
        .join("");
    let body_html = body_rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(|cell| format!("<td>{}</td>", escape_html(cell)))
                .collect::<Vec<_>>()
                .join("");
            format!("<tr>{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <title>{title}</title>
  <style>
    body {{ font-family: "Segoe UI", Arial, sans-serif; background:#F3F0EB; margin:0; padding:28px; }}
    .sheet {{
      max-width: 980px; margin: 0 auto; background:#FFF; border-radius:16px; overflow:hidden;
      border:1px solid #E8E2DC; box-shadow:0 18px 50px rgba(0,0,0,0.08);
    }}
    .head {{ padding:24px 28px; background:linear-gradient(135deg,#111,#2A1208); color:#FFF; }}
    .head h1 {{ margin:0; font-size:28px; color:#FF4500; }}
    table {{ width:100%; border-collapse:collapse; }}
    th, td {{ padding:12px 14px; border-bottom:1px solid #EEE; text-align:left; font-size:14px; }}
    tr:nth-child(even) td {{ background:#FFF7F2; }}
    th {{ background:#FF4500; color:#FFF; font-size:12px; letter-spacing:0.06em; text-transform:uppercase; }}
  </style>
</head>
<body>
  <div class="sheet">
    <div class="head"><h1>{title}</h1></div>
    <table><thead><tr>{header_html}</tr></thead><tbody>{body_html}</tbody></table>
  </div>
</body>
</html>"#,
        title = escape_html(title),
        header_html = header_html,
        body_html = body_html
    )
}

fn split_table_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim().trim_matches('|');
    if trimmed.contains('|') {
        trimmed.split('|').collect()
    } else if trimmed.contains(';') {
        trimmed.split(';').collect()
    } else if trimmed.contains('\t') {
        trimmed.split('\t').collect()
    } else if trimmed.contains(',') {
        trimmed.split(',').collect()
    } else {
        vec![trimmed]
    }
}

fn content_to_csv(content: &str) -> String {
    let mut csv = String::from("linha,conteudo\n");
    for (index, line) in content.lines().enumerate() {
        csv.push_str(&format!("{},\"{}\"\n", index + 1, line.replace('"', "\"\"")));
    }
    csv
}

fn content_to_svg(title: &str, content: &str) -> String {
    let lines = content
        .lines()
        .take(10)
        .enumerate()
        .map(|(index, line)| {
            format!(
                r##"<text x="32" y="{}" fill="#f0f0f0" font-family="monospace" font-size="18">{}</text>"##,
                110 + index * 28,
                escape_html(line)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"<svg width="900" height="520" viewBox="0 0 900 520" xmlns="http://www.w3.org/2000/svg">
<rect width="900" height="520" rx="24" fill="#0D0D0D"/>
<text x="32" y="58" fill="#FF4500" font-family="Arial" font-size="32" font-weight="800">{}</text>
{}
</svg>"##,
        escape_html(title),
        lines
    )
}

fn simple_pdf(title: &str, content: &str) -> String {
    let mut lines = Vec::new();
    lines.push(title.to_string());
    lines.push(String::new());
    for line in content.lines() {
        let line = line.trim_end();
        if line.chars().count() <= 95 {
            lines.push(line.to_string());
            continue;
        }
        let mut current = String::new();
        for word in line.split_whitespace() {
            if current.chars().count() + word.chars().count() + 1 > 95 {
                lines.push(current.trim().to_string());
                current.clear();
            }
            current.push_str(word);
            current.push(' ');
        }
        if !current.trim().is_empty() {
            lines.push(current.trim().to_string());
        }
    }

    let page_chunks = lines
        .chunks(42)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let page_count = page_chunks.len().max(1);
    let mut objects = Vec::new();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());

    let kids = (0..page_count)
        .map(|index| format!("{} 0 R", 4 + index * 2))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids, page_count));
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    for (page_index, chunk) in page_chunks.iter().enumerate() {
        let page_object = 4 + page_index * 2;
        let content_object = page_object + 1;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            content_object
        ));
        let text_stream = chunk
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                let y = 780 - (line_index as i32 * 17);
                format!("BT /F1 11 Tf 50 {} Td ({}) Tj ET", y, escape_pdf_text(line))
            })
            .collect::<Vec<_>>()
            .join("\n");
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            text_stream.len(),
            text_stream
        ));
    }

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
    }
    let xref_start = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1));
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        objects.len() + 1,
        xref_start
    ));
    pdf
}

fn escape_pdf_text(value: &str) -> String {
    transliterate_latin(value)
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn transliterate_latin(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'ç' => 'c',
            'Ç' => 'C',
            character if character.is_ascii() => character,
            _ => ' ',
        })
        .collect()
}

fn sanitize_filename(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "forge-arquivo".to_string()
    } else {
        sanitized.chars().take(48).collect()
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

async fn patterns() -> Json<forge_adapters::AdaptivePatternProfile> {
    Json(mock_adaptive_patterns())
}

async fn history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Json<Vec<forge_application::HistoryItem>> {
    Json(state.engine.history(query.limit.unwrap_or(20)))
}

async fn stats(State(state): State<AppState>) -> Json<forge_application::EngineStats> {
    Json(state.engine.stats())
}

async fn usage(State(state): State<AppState>) -> Json<UsageResponse> {
    let rows = sqlx::query_as::<_, (String, Option<i64>, String, String)>(
        r#"
        SELECT provider, tokens_used, prompt_used, response
        FROM chat_history
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut by_provider = std::collections::BTreeMap::<String, (u32, u32)>::new();
    for (provider, tokens_used, prompt, response) in rows {
        let estimated_tokens = tokens_used
            .map(|value| value.max(0) as u32)
            .unwrap_or_else(|| ((prompt.len() + response.len()) as f32 / 4.0).ceil() as u32);
        let entry = by_provider.entry(provider).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += estimated_tokens;
    }

    let mut total_requests = 0u32;
    let mut total_tokens = 0u32;
    let mut estimated_cost_usd = 0.0f32;
    let by_provider = by_provider
        .into_iter()
        .map(|(provider, (requests, tokens))| {
            let cost = estimated_cost(&provider, tokens);
            total_requests += requests;
            total_tokens += tokens;
            estimated_cost_usd += cost;
            ProviderUsage {
                note: usage_note(&provider).to_string(),
                provider,
                requests,
                tokens,
                estimated_cost_usd: cost,
            }
        })
        .collect::<Vec<_>>();

    Json(UsageResponse {
        total_requests,
        total_tokens,
        estimated_cost_usd,
        estimated_cost_brl: estimated_cost_usd * 5.5,
        by_provider,
    })
}

fn estimated_cost(provider: &str, tokens: u32) -> f32 {
    let per_million = match provider {
        "ollama" => 0.0,
        "groq" => 0.0,
        "openrouter" => 0.05,
        "gemini" => 0.10,
        "openai" => 0.60,
        "claude" => 0.80,
        "mistral" => 0.20,
        "deepseek" => 0.14,
        "together" => 0.20,
        "cerebras" => 0.0,
        "huggingface" => 0.0,
        _ => 0.25,
    };
    (tokens as f32 / 1_000_000.0) * per_million
}

fn usage_note(provider: &str) -> &'static str {
    match provider {
        "ollama" => "Local e gratuito; custo zero, depende da sua maquina.",
        "groq" => "Rapido e com free tier; ideal como padrao quando configurado.",
        "openrouter" => "Gateway com modelos free e pagos; custo varia por modelo.",
        "claude" => "Alta qualidade; custo estimado maior.",
        "openai" => "Modelos gerais fortes; custo estimado medio.",
        "gemini" => "Bom custo-beneficio em modelos Flash.",
        _ => "Custo estimado; confirme no painel oficial do provider.",
    }
}

async fn load_providers(state: &AppState) -> ProvidersResponse {
    let saved = saved_key_status(&state.db).await;
    let ollama_models = ollama_models(&state.http).await.unwrap_or_default();
    let ollama_available = !ollama_models.is_empty();

    let mut providers = vec![
        provider_status(
            "claude",
            "Claude (Anthropic)",
            env_or_saved_key("ANTHROPIC_API_KEY", "claude", &saved),
            false,
            vec![
                "claude-3-5-haiku-latest".to_string(),
                "claude-3-5-sonnet-latest".to_string(),
                "claude-3-opus-latest".to_string(),
            ],
            false,
        ),
        provider_status(
            "groq",
            "Groq",
            env_or_saved_key("GROQ_API_KEY", "groq", &saved),
            false,
            vec![
                "llama-3.1-8b-instant".to_string(),
                "llama-3.3-70b-versatile".to_string(),
                "mixtral-8x7b-32768".to_string(),
                "gemma2-9b-it".to_string(),
            ],
            true,
        ),
        provider_status(
            "openai",
            "OpenAI",
            env_or_saved_key("OPENAI_API_KEY", "openai", &saved),
            false,
            vec![
                "gpt-4o-mini".to_string(),
                "gpt-4o".to_string(),
                "gpt-4.1-mini".to_string(),
                "gpt-4.1".to_string(),
            ],
            false,
        ),
        provider_status(
            "gemini",
            "Gemini (Google)",
            env_or_saved_key("GEMINI_API_KEY", "gemini", &saved),
            false,
            vec![
                "gemini-1.5-flash".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-2.0-flash".to_string(),
            ],
            false,
        ),
        provider_status(
            "openrouter",
            "OpenRouter",
            env_or_saved_key("OPENROUTER_API_KEY", "openrouter", &saved),
            false,
            vec![
                "meta-llama/llama-3.1-8b-instruct:free".to_string(),
                "qwen/qwen-2.5-coder-32b-instruct".to_string(),
                "google/gemini-flash-1.5".to_string(),
                "anthropic/claude-3.5-haiku".to_string(),
            ],
            true,
        ),
        provider_status(
            "mistral",
            "Mistral AI",
            env_or_saved_key("MISTRAL_API_KEY", "mistral", &saved),
            false,
            vec![
                "mistral-small-latest".to_string(),
                "mistral-large-latest".to_string(),
                "codestral-latest".to_string(),
            ],
            false,
        ),
        provider_status(
            "deepseek",
            "DeepSeek",
            env_or_saved_key("DEEPSEEK_API_KEY", "deepseek", &saved),
            false,
            vec![
                "deepseek-chat".to_string(),
                "deepseek-reasoner".to_string(),
            ],
            false,
        ),
        provider_status(
            "together",
            "Together AI",
            env_or_saved_key("TOGETHER_API_KEY", "together", &saved),
            false,
            vec![
                "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo".to_string(),
                "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
                "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
            ],
            false,
        ),
        provider_status(
            "cerebras",
            "Cerebras",
            env_or_saved_key("CEREBRAS_API_KEY", "cerebras", &saved),
            false,
            vec!["llama3.1-8b".to_string(), "llama3.3-70b".to_string()],
            true,
        ),
        provider_status(
            "huggingface",
            "Hugging Face",
            env_or_saved_key("HF_TOKEN", "huggingface", &saved),
            false,
            vec![
                "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
                "meta-llama/Llama-3.1-8B-Instruct".to_string(),
                "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
            ],
            true,
        ),
    ];

    providers.insert(
        0,
        ProviderStatus {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            available: ollama_available,
            configured: ollama_available,
            is_local: true,
            models: ollama_models,
            free: true,
        },
    );

    let first_run = !providers.iter().any(|provider| provider.configured);
    ProvidersResponse {
        providers,
        first_run,
    }
}

fn provider_status(
    id: &str,
    name: &str,
    configured: bool,
    is_local: bool,
    models: Vec<String>,
    free: bool,
) -> ProviderStatus {
    ProviderStatus {
        id: id.to_string(),
        name: name.to_string(),
        available: configured,
        configured,
        is_local,
        models,
        free,
    }
}

fn env_or_saved_key(env_name: &str, provider: &str, saved: &[(String, bool)]) -> bool {
    std::env::var(env_name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || saved
            .iter()
            .any(|(saved_provider, validated)| saved_provider == provider && *validated)
}

async fn saved_key_status(db: &SqlitePool) -> Vec<(String, bool)> {
    sqlx::query_as::<_, (String, bool)>("SELECT provider, validated FROM provider_keys")
        .fetch_all(db)
        .await
        .unwrap_or_default()
}

async fn provider_key(db: &SqlitePool, provider: &str) -> Option<String> {
    let env_name = match provider {
        "claude" => "ANTHROPIC_API_KEY",
        "groq" => "GROQ_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "gemini" => "GEMINI_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "together" => "TOGETHER_API_KEY",
        "cerebras" => "CEREBRAS_API_KEY",
        "huggingface" => "HF_TOKEN",
        _ => return None,
    };

    if let Ok(key) = std::env::var(env_name) {
        if !key.trim().is_empty() {
            return Some(key);
        }
    }

    sqlx::query_scalar::<_, String>("SELECT api_key FROM provider_keys WHERE provider = ?1")
        .bind(provider)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

async fn ollama_models(http: &Client) -> Result<Vec<String>, reqwest::Error> {
    let response = http
        .get("http://localhost:11434/api/tags")
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    Ok(response
        .get("models")
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(|model| model.get("name").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default())
}

async fn validate_provider_key(http: &Client, provider: &str, key: &str) -> bool {
    let prompt = "Responda apenas OK.";
    call_remote_provider(http, provider, key, prompt, None)
        .await
        .is_ok()
}

async fn call_remote_provider(
    http: &Client,
    provider: &str,
    key: &str,
    prompt: &str,
    model: Option<&str>,
) -> Result<ChatResponse, reqwest::Error> {
    match provider {
        "claude" => {
            let model_used = model.unwrap_or("claude-3-5-haiku-latest");
            let response = http
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&json!({
                    "model": model_used,
                    "max_tokens": 1200,
                    "messages": [{"role": "user", "content": prompt}]
                }))
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?;
            let text = response
                .get("content")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(|item| item.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Ok(ChatResponse {
                response: text,
                provider_used: "claude".to_string(),
                model_used: model_used.to_string(),
                tokens_used: response
                    .get("usage")
                    .and_then(|usage| usage.get("output_tokens"))
                    .and_then(Value::as_u64)
                    .map(|value| value as u32),
            })
        }
        "groq" => {
            call_openai_compatible(
                http,
                "https://api.groq.com/openai/v1/chat/completions",
                key,
                model.unwrap_or("llama-3.1-8b-instant"),
                prompt,
                "groq",
            )
            .await
        }
        "openai" => {
            call_openai_compatible(
                http,
                "https://api.openai.com/v1/chat/completions",
                key,
                model.unwrap_or("gpt-4o-mini"),
                prompt,
                "openai",
            )
            .await
        }
        "openrouter" => {
            call_openai_compatible(
                http,
                "https://openrouter.ai/api/v1/chat/completions",
                key,
                model.unwrap_or("meta-llama/llama-3.1-8b-instruct:free"),
                prompt,
                "openrouter",
            )
            .await
        }
        "mistral" => {
            call_openai_compatible(
                http,
                "https://api.mistral.ai/v1/chat/completions",
                key,
                model.unwrap_or("mistral-small-latest"),
                prompt,
                "mistral",
            )
            .await
        }
        "deepseek" => {
            call_openai_compatible(
                http,
                "https://api.deepseek.com/chat/completions",
                key,
                model.unwrap_or("deepseek-chat"),
                prompt,
                "deepseek",
            )
            .await
        }
        "together" => {
            call_openai_compatible(
                http,
                "https://api.together.xyz/v1/chat/completions",
                key,
                model.unwrap_or("meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo"),
                prompt,
                "together",
            )
            .await
        }
        "cerebras" => {
            call_openai_compatible(
                http,
                "https://api.cerebras.ai/v1/chat/completions",
                key,
                model.unwrap_or("llama3.1-8b"),
                prompt,
                "cerebras",
            )
            .await
        }
        "huggingface" => {
            call_openai_compatible(
                http,
                "https://router.huggingface.co/v1/chat/completions",
                key,
                model.unwrap_or("Qwen/Qwen2.5-Coder-32B-Instruct"),
                prompt,
                "huggingface",
            )
            .await
        }
        "gemini" => {
            let model_used = model.unwrap_or("gemini-1.5-flash");
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{model_used}:generateContent?key={key}"
            );
            let response = http
                .post(url)
                .json(&json!({
                    "contents": [{"parts": [{"text": prompt}]}]
                }))
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?;
            let text = response
                .get("candidates")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(|item| item.get("content"))
                .and_then(|content| content.get("parts"))
                .and_then(Value::as_array)
                .and_then(|parts| parts.first())
                .and_then(|part| part.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Ok(ChatResponse {
                response: text,
                provider_used: "gemini".to_string(),
                model_used: model_used.to_string(),
                tokens_used: response
                    .get("usageMetadata")
                    .and_then(|usage| usage.get("totalTokenCount"))
                    .and_then(Value::as_u64)
                    .map(|value| value as u32),
            })
        }
        _ => unreachable!("unsupported remote provider"),
    }
}

async fn call_openai_compatible(
    http: &Client,
    url: &str,
    key: &str,
    model: &str,
    prompt: &str,
    provider: &str,
) -> Result<ChatResponse, reqwest::Error> {
    let response = http
        .post(url)
        .bearer_auth(key)
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let text = response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(ChatResponse {
        response: text,
        provider_used: provider.to_string(),
        model_used: model.to_string(),
        tokens_used: response
            .get("usage")
            .and_then(|usage| usage.get("total_tokens"))
            .and_then(Value::as_u64)
            .map(|value| value as u32),
    })
}

async fn call_ollama(
    http: &Client,
    prompt: &str,
    model: Option<&str>,
) -> Result<ChatResponse, reqwest::Error> {
    let models = ollama_models(http).await.unwrap_or_default();
    let model_used = model
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .or_else(|| preferred_ollama_model(&models))
        .unwrap_or_else(|| "llama3.2:3b".to_string());
    let response = http
        .post("http://localhost:11434/api/generate")
        .timeout(Duration::from_secs(300))
        .json(&json!({
            "model": model_used,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": 512
            }
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    Ok(ChatResponse {
        response: response
            .get("response")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        provider_used: "ollama".to_string(),
        model_used,
        tokens_used: None,
    })
}

fn preferred_ollama_model(models: &[String]) -> Option<String> {
    ["qwen", "llama3.2", "llama3.1", "llama", "mistral"]
        .iter()
        .find_map(|needle| models.iter().find(|model| model.contains(needle)).cloned())
        .or_else(|| models.iter().find(|model| !model.contains("deepseek")).cloned())
        .or_else(|| models.first().cloned())
}

fn ollama_error_message(error: reqwest::Error) -> String {
    let details = error.to_string();
    if details.contains("11434") || details.contains("Connection refused") || error.is_connect() {
        return "Ollama não está rodando ou não respondeu em http://localhost:11434. Abra o Ollama, rode `ollama serve` ou escolha/configure Claude, Groq, OpenAI ou Gemini em Configurações.".to_string();
    }

    if error.is_timeout() {
        return "Ollama demorou demais para responder. Tente um modelo menor/mais rápido, como qwen2.5-coder:7b ou llama3.1:8b, ou escolha uma IA via API.".to_string();
    }

    format!("Ollama retornou erro: {details}")
}
