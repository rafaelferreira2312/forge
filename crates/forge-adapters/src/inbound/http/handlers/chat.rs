use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub provider_used: String,
    pub model_used: String,
    pub tokens_used: Option<u32>,
}
