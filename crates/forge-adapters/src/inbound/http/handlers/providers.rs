use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderStatus>,
    pub first_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub configured: bool,
    pub is_local: bool,
    pub models: Vec<String>,
    pub free: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveKeyRequest {
    pub provider: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct SaveKeyResponse {
    pub provider: String,
    pub saved: bool,
    pub valid: bool,
    pub message: String,
}
