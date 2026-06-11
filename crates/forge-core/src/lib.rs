use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EngineerRequest {
    pub input: String,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineerResponse {
    pub input: String,
    pub provider: String,
    pub intent: IntentResult,
    pub ambiguity: AmbiguityResolution,
    pub domain: DomainEnrichment,
    pub complexity: ComplexityAnalysis,
    pub technique: TechniqueSelection,
    pub parameters: AdaptiveParameters,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentResult {
    pub primary: String,
    pub confidence: f32,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmbiguityResolution {
    pub is_ambiguous: bool,
    pub assumptions: Vec<String>,
    pub clarifying_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainEnrichment {
    pub domain: String,
    pub keywords: Vec<String>,
    pub audience: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexityAnalysis {
    pub level: String,
    pub score: u8,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechniqueSelection {
    pub technique: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveParameters {
    pub temperature: f32,
    pub max_tokens: u32,
    pub style: String,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub domains: Vec<DomainRule>,
}

#[derive(Debug, Clone)]
pub struct DomainRule {
    pub name: &'static str,
    pub triggers: &'static [&'static str],
    pub keywords: &'static [&'static str],
    pub audience: &'static str,
    pub constraints: &'static [&'static str],
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self {
            domains: vec![
                DomainRule {
                    name: "juridico",
                    triggers: &[
                        "advocacia",
                        "advogado",
                        "juridico",
                        "lei",
                        "contrato",
                        "processo",
                    ],
                    keywords: &[
                        "credibilidade",
                        "areas de atuacao",
                        "captacao etica",
                        "oab",
                        "consulta",
                    ],
                    audience: "clientes que precisam de orientacao juridica confiavel",
                    constraints: &[
                        "evitar promessa de resultado",
                        "usar linguagem clara e profissional",
                        "respeitar comunicacao sobria para servicos juridicos",
                    ],
                },
                DomainRule {
                    name: "tecnologia",
                    triggers: &["software", "app", "api", "sistema", "saas", "codigo"],
                    keywords: &["arquitetura", "seguranca", "escalabilidade", "ux"],
                    audience: "usuarios e decisores tecnicos",
                    constraints: &["explicitar requisitos", "priorizar manutencao"],
                },
                DomainRule {
                    name: "marketing",
                    triggers: &["campanha", "anuncio", "vendas", "marca", "landing"],
                    keywords: &["conversao", "persona", "proposta de valor", "cta"],
                    audience: "leads e clientes em potencial",
                    constraints: &["evitar afirmacoes sem prova", "destacar diferencial"],
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptPipeline {
    knowledge: KnowledgeBase,
}

impl Default for PromptPipeline {
    fn default() -> Self {
        Self::new(KnowledgeBase::default())
    }
}

impl PromptPipeline {
    pub fn new(knowledge: KnowledgeBase) -> Self {
        Self { knowledge }
    }

    pub fn engineer(&self, request: EngineerRequest) -> EngineerResponse {
        let input = request.input.trim().to_string();
        let normalized = normalize(&input);
        let provider = request.provider.unwrap_or_else(|| "local".to_string());
        let intent = detect_intent(&normalized);
        let ambiguity = resolve_ambiguity(&input, &normalized, &intent);
        let domain = enrich_domain(&self.knowledge, &normalized);
        let complexity = analyze_complexity(&input, &normalized, &ambiguity, &domain);
        let technique = select_technique(&intent, &complexity, &domain);
        let parameters = inject_parameters(&provider, &intent, &complexity, &technique);
        let prompt = assemble_prompt(
            &input,
            &provider,
            &intent,
            &ambiguity,
            &domain,
            &complexity,
            &technique,
            &parameters,
        );

        EngineerResponse {
            input,
            provider,
            intent,
            ambiguity,
            domain,
            complexity,
            technique,
            parameters,
            prompt,
        }
    }
}

fn normalize(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'ã' | 'â' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'õ' | 'ô' => 'o',
            'ú' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

fn detect_intent(normalized: &str) -> IntentResult {
    let mut signals = Vec::new();
    let (primary, confidence) = if contains_any(normalized, &["site", "pagina", "landing", "web"]) {
        signals.push("web_presence".to_string());
        ("create_website", 0.88)
    } else if contains_any(normalized, &["app", "sistema", "api", "software"]) {
        signals.push("software_build".to_string());
        ("build_software", 0.84)
    } else if contains_any(normalized, &["texto", "email", "post", "copy"]) {
        signals.push("content_generation".to_string());
        ("write_content", 0.8)
    } else if contains_any(normalized, &["analise", "comparar", "avaliar"]) {
        signals.push("analysis".to_string());
        ("analyze", 0.78)
    } else {
        signals.push("general_request".to_string());
        ("general_assistance", 0.58)
    };

    IntentResult {
        primary: primary.to_string(),
        confidence,
        signals,
    }
}

fn resolve_ambiguity(input: &str, normalized: &str, intent: &IntentResult) -> AmbiguityResolution {
    let mut assumptions = Vec::new();
    let mut clarifying_questions = Vec::new();

    if input.split_whitespace().count() < 9 {
        assumptions.push(
            "O pedido ainda nao define publico, escopo visual ou criterios de sucesso.".to_string(),
        );
        clarifying_questions.push("Qual e o publico-alvo principal?".to_string());
    }

    if intent.primary == "create_website"
        && !contains_any(normalized, &["home", "sobre", "contato", "servico", "blog"])
    {
        assumptions.push(
            "Um site institucional padrao deve incluir home, servicos, sobre e contato."
                .to_string(),
        );
        clarifying_questions
            .push("Quais paginas e chamadas para acao sao obrigatorias?".to_string());
    }

    if !contains_any(
        normalized,
        &["prazo", "orcamento", "wordpress", "react", "html", "seo"],
    ) {
        clarifying_questions
            .push("Existe alguma restricao de tecnologia, prazo, orcamento ou SEO?".to_string());
    }

    AmbiguityResolution {
        is_ambiguous: !clarifying_questions.is_empty(),
        assumptions,
        clarifying_questions,
    }
}

fn enrich_domain(knowledge: &KnowledgeBase, normalized: &str) -> DomainEnrichment {
    if let Some(rule) = knowledge
        .domains
        .iter()
        .find(|rule| contains_any(normalized, rule.triggers))
    {
        return DomainEnrichment {
            domain: rule.name.to_string(),
            keywords: rule.keywords.iter().map(|item| item.to_string()).collect(),
            audience: rule.audience.to_string(),
            constraints: rule
                .constraints
                .iter()
                .map(|item| item.to_string())
                .collect(),
        };
    }

    DomainEnrichment {
        domain: "geral".to_string(),
        keywords: vec![
            "clareza".to_string(),
            "objetivo".to_string(),
            "contexto".to_string(),
        ],
        audience: "publico definido pelo solicitante".to_string(),
        constraints: vec!["validar requisitos antes da execucao".to_string()],
    }
}

fn analyze_complexity(
    input: &str,
    normalized: &str,
    ambiguity: &AmbiguityResolution,
    domain: &DomainEnrichment,
) -> ComplexityAnalysis {
    let mut score = 1u8;
    let mut factors = Vec::new();

    if input.split_whitespace().count() > 18 {
        score += 1;
        factors.push("pedido com mais contexto textual".to_string());
    }

    if ambiguity.is_ambiguous {
        score += 2;
        factors.push("requisitos implicitos ou incompletos".to_string());
    }

    if contains_any(
        normalized,
        &["integracao", "pagamento", "login", "dashboard", "automacao"],
    ) {
        score += 2;
        factors.push("possivel requisito tecnico adicional".to_string());
    }

    if domain.domain != "geral" {
        score += 1;
        factors.push(format!("dominio especializado: {}", domain.domain));
    }

    let level = match score {
        0..=2 => "baixa",
        3..=5 => "media",
        _ => "alta",
    };

    ComplexityAnalysis {
        level: level.to_string(),
        score,
        factors,
    }
}

fn select_technique(
    intent: &IntentResult,
    complexity: &ComplexityAnalysis,
    domain: &DomainEnrichment,
) -> TechniqueSelection {
    let (technique, rationale) = if complexity.level == "alta" {
        (
            "structured_decomposition",
            "divide o problema em objetivo, contexto, restricoes, entregaveis e criterios de aceite",
        )
    } else if domain.domain != "geral" {
        (
            "role_context_constraints",
            "combina papel especializado com regras de dominio para reduzir respostas genericas",
        )
    } else if intent.primary == "create_website" {
        (
            "brief_to_deliverables",
            "transforma um pedido curto em briefing acionavel para criacao de site",
        )
    } else {
        (
            "direct_instruction",
            "mantem a resposta objetiva para uma tarefa simples",
        )
    };

    TechniqueSelection {
        technique: technique.to_string(),
        rationale: rationale.to_string(),
    }
}

fn inject_parameters(
    provider: &str,
    intent: &IntentResult,
    complexity: &ComplexityAnalysis,
    technique: &TechniqueSelection,
) -> AdaptiveParameters {
    let creative_intent = matches!(intent.primary.as_str(), "create_website" | "write_content");
    let temperature = if creative_intent { 0.72 } else { 0.35 };
    let max_tokens = match complexity.level.as_str() {
        "alta" => 2400,
        "media" => 1600,
        _ => 900,
    };
    let style = if technique.technique == "structured_decomposition" {
        "estruturado e consultivo"
    } else {
        "claro e pratico"
    };

    AdaptiveParameters {
        temperature,
        max_tokens,
        style: style.to_string(),
        provider: provider.to_string(),
    }
}

fn assemble_prompt(
    input: &str,
    provider: &str,
    intent: &IntentResult,
    ambiguity: &AmbiguityResolution,
    domain: &DomainEnrichment,
    complexity: &ComplexityAnalysis,
    technique: &TechniqueSelection,
    parameters: &AdaptiveParameters,
) -> String {
    format!(
        "Voce e um especialista em engenharia de prompts.\n\nPedido original: {input}\n\nIntencao detectada: {intent}.\nDominio: {domain} para {audience}.\nTecnica recomendada: {technique} ({rationale}).\nComplexidade: {level} (score {score}).\n\nRegras de dominio:\n- {constraints}\n\nAssumicoes para seguir sem bloquear:\n- {assumptions}\n\nPerguntas uteis, se houver tempo para refinamento:\n- {questions}\n\nEscreva um prompt final em portugues para o provedor {provider}, com estilo {style}, temperatura sugerida {temperature} e limite aproximado de {max_tokens} tokens. O prompt deve pedir uma resposta executavel, com secoes, criterios de qualidade e proximos passos.",
        intent = intent.primary,
        domain = domain.domain,
        audience = domain.audience,
        technique = technique.technique,
        rationale = technique.rationale,
        level = complexity.level,
        score = complexity.score,
        constraints = join_or_default(&domain.constraints, "sem restricoes adicionais"),
        assumptions = join_or_default(&ambiguity.assumptions, "nenhuma assuncao critica"),
        questions = join_or_default(&ambiguity.clarifying_questions, "nenhuma pergunta obrigatoria"),
        style = parameters.style,
        temperature = parameters.temperature,
        max_tokens = parameters.max_tokens,
    )
}

fn contains_any(input: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| input.contains(candidate))
}

fn join_or_default(items: &[String], default: &str) -> String {
    if items.is_empty() {
        default.to_string()
    } else {
        items.join("\n- ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_legal_website_request() {
        let pipeline = PromptPipeline::default();
        let response = pipeline.engineer(EngineerRequest {
            input: "faz um site pra minha empresa de advocacia".to_string(),
            provider: Some("claude".to_string()),
        });

        assert_eq!(response.intent.primary, "create_website");
        assert_eq!(response.domain.domain, "juridico");
        assert_eq!(response.provider, "claude");
        assert!(response.prompt.contains("advocacia"));
    }

    #[test]
    fn flags_short_requests_as_ambiguous() {
        let ambiguity =
            resolve_ambiguity("faz um site", "faz um site", &detect_intent("faz um site"));
        assert!(ambiguity.is_ambiguous);
        assert!(!ambiguity.clarifying_questions.is_empty());
    }
}
