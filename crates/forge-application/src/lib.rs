use forge_core::{EngineerRequest, EngineerResponse, PromptPipeline};
use serde::Serialize;
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

#[derive(Clone)]
pub struct ApplicationEngine {
    pipeline: PromptPipeline,
    history: Arc<Mutex<VecDeque<HistoryItem>>>,
    sequence: Arc<AtomicU64>,
}

impl ApplicationEngine {
    pub fn new(pipeline: PromptPipeline) -> Self {
        Self {
            pipeline,
            history: Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_HISTORY_LIMIT))),
            sequence: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn engineer(&self, request: EngineerRequest) -> EngineerResponse {
        let response = self.pipeline.engineer(request);
        self.record(&response);
        response
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
}
