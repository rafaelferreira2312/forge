use forge_core::{DomainRule, KnowledgeBase};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AdaptivePatternProfile {
    pub profile: String,
    pub signals: Vec<String>,
    pub preferred_techniques: Vec<String>,
    pub notes: String,
}

pub fn local_knowledge_base() -> KnowledgeBase {
    let mut knowledge = KnowledgeBase::default();
    knowledge.domains.push(DomainRule {
        name: "educacao",
        triggers: &["curso", "aula", "treinamento", "professor"],
        keywords: &["aprendizagem", "modulos", "exercicios", "avaliacao"],
        audience: "estudantes ou equipes em capacitacao",
        constraints: &["organizar por nivel", "incluir pratica guiada"],
    });
    knowledge
}

pub fn mock_adaptive_patterns() -> AdaptivePatternProfile {
    AdaptivePatternProfile {
        profile: "local_mock_v1".to_string(),
        signals: vec![
            "pedidos curtos recebem perguntas de clarificacao".to_string(),
            "dominios regulados recebem restricoes explicitas".to_string(),
            "tarefas criativas usam temperatura moderada".to_string(),
        ],
        preferred_techniques: vec![
            "role_context_constraints".to_string(),
            "brief_to_deliverables".to_string(),
            "structured_decomposition".to_string(),
        ],
        notes: "Perfil adaptativo em memoria, sem dependencia externa ou API key.".to_string(),
    }
}
