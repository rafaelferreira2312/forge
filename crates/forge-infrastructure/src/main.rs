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
        .route("/api/leads/dna", post(save_lead_dna))
        .route("/api/install/ollama-model", post(install_ollama_model))
        .route("/api/health", get(health))
        .route("/api/patterns", get(patterns))
        .route("/api/history", get(history))
        .route("/api/stats", get(stats))
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
    Json(request): Json<EngineerRequest>,
) -> Result<Json<forge_core::EngineerResponse>, impl IntoResponse> {
    if request.input.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "input must not be empty"));
    }

    Ok(Json(state.engine.engineer(request)))
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
