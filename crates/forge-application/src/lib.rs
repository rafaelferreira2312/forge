pub mod services;

use forge_core::{EngineerRequest, EngineerResponse, ExpertiseLevel, PromptPipeline};
use serde::Serialize;
use services::media_prompt_builder::MediaPromptBuilder;
use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryItem {
    pub id: u64,
    pub created_at_unix: u64,
    pub input: String,
    pub provider: String,
    pub intent: String,
    pub domain: String,
    pub complexity: String,
    pub technique: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineStats {
    pub total_requests: u64,
    pub history_size: usize,
    pub by_domain: BTreeMap<String, u64>,
    pub by_intent: BTreeMap<String, u64>,
    pub average_complexity_score: f32,
    pub last_request_at_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileResponse {
    pub expertise_level: String,
    pub expertise_label: String,
    pub prompt_modifier_preview: String,
    pub temperature_adjustment: f32,
    pub max_tokens_adjustment: i32,
    pub domain: String,
    pub interaction_count: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
struct ProfileState {
    expertise: ExpertiseLevel,
    domain: String,
    interaction_count: u32,
    confidence: f32,
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            expertise: ExpertiseLevel::default(),
            domain: "all".to_string(),
            interaction_count: 0,
            confidence: 0.1,
        }
    }
}

#[derive(Clone)]
pub struct ApplicationEngine {
    pipeline: PromptPipeline,
    history: Arc<Mutex<VecDeque<HistoryItem>>>,
    profile: Arc<Mutex<ProfileState>>,
    sequence: Arc<AtomicU64>,
}

impl ApplicationEngine {
    pub fn new(pipeline: PromptPipeline) -> Self {
        Self {
            pipeline,
            history: Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_HISTORY_LIMIT))),
            profile: Arc::new(Mutex::new(ProfileState::default())),
            sequence: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn engineer(&self, request: EngineerRequest) -> EngineerResponse {
        self.engineer_internal(request, true)
    }

    pub fn preview(&self, request: EngineerRequest) -> EngineerResponse {
        self.engineer_internal(request, false)
    }

    fn engineer_internal(&self, request: EngineerRequest, record: bool) -> EngineerResponse {
        let expertise = self
            .profile
            .lock()
            .expect("profile mutex poisoned")
            .expertise
            .clone();
        let media_prompt =
            MediaPromptBuilder::detect_media_intent(&request.input).map(|media_type| {
                MediaPromptBuilder::build(&request.input, &media_type, &expertise, None)
            });
        let response = self
            .pipeline
            .engineer_with_expertise(request, expertise, media_prompt);
        if record {
            self.record(&response);
        }
        response
    }

    pub fn profile(&self) -> ProfileResponse {
        let profile = self.profile.lock().expect("profile mutex poisoned");
        profile_response(&profile)
    }

    pub fn update_profile(&self, expertise_level: &str, domain: Option<String>) -> ProfileResponse {
        let mut profile = self.profile.lock().expect("profile mutex poisoned");
        profile.expertise = ExpertiseLevel::from_str(expertise_level);
        profile.domain = domain
            .filter(|domain| !domain.trim().is_empty())
            .unwrap_or_else(|| "all".to_string());
        profile_response(&profile)
    }

    pub fn history(&self, limit: usize) -> Vec<HistoryItem> {
        let limit = limit.clamp(1, DEFAULT_HISTORY_LIMIT);
        let history = self.history.lock().expect("history mutex poisoned");
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn stats(&self) -> EngineStats {
        let history = self.history.lock().expect("history mutex poisoned");
        let mut by_domain = BTreeMap::new();
        let mut by_intent = BTreeMap::new();
        let mut score_total = 0u32;
        let mut last_request_at_unix = None;

        for item in history.iter() {
            *by_domain.entry(item.domain.clone()).or_insert(0) += 1;
            *by_intent.entry(item.intent.clone()).or_insert(0) += 1;
            score_total += match item.complexity.as_str() {
                "alta" => 6,
                "media" => 4,
                _ => 2,
            };
            last_request_at_unix = Some(item.created_at_unix);
        }

        let total_requests = history.len() as u64;
        let average_complexity_score = if total_requests == 0 {
            0.0
        } else {
            score_total as f32 / total_requests as f32
        };

        EngineStats {
            total_requests,
            history_size: history.len(),
            by_domain,
            by_intent,
            average_complexity_score,
            last_request_at_unix,
        }
    }

    fn record(&self, response: &EngineerResponse) {
        let id = self.sequence.fetch_add(1, Ordering::SeqCst);
        let item = HistoryItem {
            id,
            created_at_unix: now_unix(),
            input: response.input.clone(),
            provider: response.provider.clone(),
            intent: response.intent.primary.clone(),
            domain: response.domain.domain.clone(),
            complexity: response.complexity.level.clone(),
            technique: response.technique.technique.clone(),
            prompt: response.prompt.clone(),
        };

        let mut history = self.history.lock().expect("history mutex poisoned");
        if history.len() == DEFAULT_HISTORY_LIMIT {
            history.pop_front();
        }
        history.push_back(item);

        let mut profile = self.profile.lock().expect("profile mutex poisoned");
        profile.interaction_count = profile.interaction_count.saturating_add(1);
        profile.confidence = (profile.confidence + 0.02).min(0.9);
    }
}

fn profile_response(profile: &ProfileState) -> ProfileResponse {
    let modifier = profile.expertise.prompt_modifier();
    let prompt_modifier_preview = modifier.chars().take(100).collect();

    ProfileResponse {
        expertise_level: profile.expertise.as_config_value().to_string(),
        expertise_label: profile.expertise.label().to_string(),
        prompt_modifier_preview,
        temperature_adjustment: profile.expertise.temperature_modifier(),
        max_tokens_adjustment: profile.expertise.max_tokens_modifier(),
        domain: profile.domain.clone(),
        interaction_count: profile.interaction_count,
        confidence: profile.confidence,
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::KnowledgeBase;

    #[test]
    fn records_history_and_stats() {
        let app = ApplicationEngine::new(PromptPipeline::new(KnowledgeBase::default()));
        app.engineer(EngineerRequest {
            input: "faz um site pra minha empresa de advocacia".to_string(),
            provider: None,
        });

        let stats = app.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.by_domain.get("juridico"), Some(&1));
        assert_eq!(app.history(20).len(), 1);
    }

    #[test]
    fn updates_profile_expertise() {
        let app = ApplicationEngine::new(PromptPipeline::default());
        let profile = app.update_profile("especialista", Some("all".to_string()));

        assert_eq!(profile.expertise_level, "Especialista");
        assert_eq!(profile.max_tokens_adjustment, 1000);
    }

    #[test]
    fn media_prompt_uses_current_expertise() {
        let app = ApplicationEngine::new(PromptPipeline::default());
        app.update_profile("especialista", Some("all".to_string()));
        let response = app.engineer(EngineerRequest {
            input: "faz um logo para minha empresa".to_string(),
            provider: Some("claude".to_string()),
        });

        assert!(response.prompt.contains("Briefing técnico de design"));
        assert_eq!(response.parameters.expertise_level, "Especialista");
    }
}
