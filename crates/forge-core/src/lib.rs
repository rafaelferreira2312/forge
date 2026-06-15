pub mod domain;

pub use domain::value_objects::{ExpertiseLevel, KnowledgeLevel, MediaOutputType, UserArea};

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
    pub expertise_level: String,
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
        self.engineer_with_expertise(request, ExpertiseLevel::default(), None)
    }

    pub fn engineer_with_expertise(
        &self,
        request: EngineerRequest,
        expertise: ExpertiseLevel,
        media_prompt: Option<String>,
    ) -> EngineerResponse {
        let input = request.input.trim().to_string();
        let normalized = normalize(&input);
        let provider = request.provider.unwrap_or_else(|| "local".to_string());
        let intent = detect_intent(&normalized);
        let ambiguity = resolve_ambiguity(&input, &normalized, &intent);
        let domain = enrich_domain(&self.knowledge, &input, &normalized);
        let complexity = analyze_complexity(&input, &normalized, &ambiguity, &domain);
        let technique = select_technique(&intent, &complexity, &domain);
        let parameters = inject_parameters(&provider, &intent, &complexity, &technique, &expertise);
        let prompt = if let Some(media_prompt) = media_prompt {
            assemble_media_prompt(&media_prompt, &domain, &expertise)
        } else {
            assemble_prompt(&input, &intent, &domain, &technique, &expertise)
        };

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
    let (primary, confidence) = if is_direct_answer_request(normalized) {
        signals.push("direct_answer".to_string());
        ("direct_answer", 0.9)
    } else if is_creative_request(normalized) {
        signals.push("creative_engine".to_string());
        ("open_creative", 0.95)
    } else if contains_any(normalized, &["site", "pagina", "landing", "web"]) {
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

fn enrich_domain(knowledge: &KnowledgeBase, input: &str, normalized: &str) -> DomainEnrichment {
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

    if let Some((sector, audience, constraints, keywords)) = detect_sector(input) {
        return DomainEnrichment {
            domain: sector.to_string(),
            keywords: keywords
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
            audience: audience.to_string(),
            constraints: constraints.iter().map(|item| item.to_string()).collect(),
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
    let (technique, rationale) = if intent.primary == "open_creative" {
        (
            "creative_engine",
            "transforma pedido aberto em uma entrega criativa completa sem pedir confirmacao",
        )
    } else if complexity.level == "alta" {
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
    expertise: &ExpertiseLevel,
) -> AdaptiveParameters {
    let creative_intent = matches!(
        intent.primary.as_str(),
        "create_website" | "write_content" | "open_creative"
    );
    let base_temperature = if creative_intent { 0.72 } else { 0.35 };
    let base_max_tokens = match complexity.level.as_str() {
        "alta" => 2400,
        "media" => 1600,
        _ => 900,
    };
    let temperature = if intent.primary == "open_creative" {
        0.92
    } else {
        (base_temperature + expertise.temperature_modifier()).clamp(0.0, 1.0)
    };
    let max_tokens =
        (base_max_tokens as i32 + expertise.max_tokens_modifier()).clamp(256, 6000) as u32;
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
        expertise_level: expertise.as_config_value().to_string(),
    }
}

fn assemble_prompt(
    input: &str,
    intent: &IntentResult,
    domain: &DomainEnrichment,
    technique: &TechniqueSelection,
    expertise: &ExpertiseLevel,
) -> String {
    assemble_executable_prompt(
        input,
        &intent.primary,
        &domain.domain,
        &domain.audience,
        &domain.constraints,
        &technique.technique,
        expertise.role_prefix(),
        expertise.prompt_modifier(),
    )
}

pub fn assemble_executable_prompt(
    original_input: &str,
    intent: &str,
    domain: &str,
    audience: &str,
    constraints: &[String],
    _technique: &str,
    expertise_role: &str,
    expertise_modifier: &str,
) -> String {
    let constraints_text = if constraints.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nRESTRIÇÕES DO DOMÍNIO:\n{}",
            constraints
                .iter()
                .map(|constraint| format!("- {}", constraint))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    match intent {
        "create_website" => format!(
            "{expertise_role}\n\nCrie um site completo para: {original_input}\n\nDOMÍNIO: {domain} | PÚBLICO: {audience}\n\nENTREGUE OBRIGATORIAMENTE:\n1. Estrutura de arquivos do projeto\n2. HTML semântico completo (todas as páginas)\n3. CSS moderno responsivo (mobile-first)\n4. JavaScript funcional (formulários, interações)\n5. README com instruções de deploy\n\nSEÇÕES MÍNIMAS DO SITE:\n- Hero com headline clara e CTA principal\n- Sobre / Quem somos\n- Serviços / O que oferecemos\n- Depoimentos ou casos de sucesso\n- Contato com formulário funcional\n- Footer com informações essenciais\n\nPADRÕES TÉCNICOS:\n- HTML5 semântico\n- CSS: variáveis, flexbox/grid, responsivo\n- SEO: meta tags, Open Graph, schema.org adequado ao setor\n- Performance: lazy loading, fontes otimizadas\n- Acessibilidade: WCAG 2.1 AA, foco visível, alt texts{constraints_text}\n\n{expertise_modifier}"
        ),

        "build_software" => format!(
            "{expertise_role}\n\nDesenvolva o seguinte sistema: {original_input}\n\nDOMÍNIO: {domain} | PÚBLICO: {audience}\n\nENTREGUE OBRIGATORIAMENTE:\n1. Estrutura de pastas do projeto\n2. Código funcional de todos os módulos principais\n3. Tratamento de erros em todos os fluxos críticos\n4. Validação de inputs\n5. README com instalação, configuração e uso\n6. Exemplos de uso reais{constraints_text}\n\n{expertise_modifier}"
        ),

        "open_creative" => format!(
            "{expertise_role}\n\nO usuário pediu para ser surpreendido. Escolha UMA das opções abaixo e execute completamente, sem pedir confirmação:\n\nA) Crie algo funcional e inesperado em código\n   (ferramenta CLI, visualizador, gerador, jogo no terminal)\n\nB) Escreva uma análise criativa de um tema cotidiano\n   sob uma perspectiva completamente inesperada\n\nC) Proponha e detalhe um produto/serviço fictício\n   que resolve um problema que as pessoas nem sabem que têm\n\nREGRAS:\n- Escolha a opção mais inesperada e útil\n- Execute completamente — não descreva, entregue\n- Máxima criatividade, mínimo de genericidade\n- Não explique sua escolha antes de executar\n\n{expertise_modifier}"
        ),

        "general_assistance" => format!(
            "{expertise_role}\n\nResponda sobre: {original_input}\n\nESTRUTURE ASSIM:\n1. Conceito central (direto, sem rodeios)\n2. Como funciona na prática\n3. Exemplos concretos e aplicáveis\n4. Quando usar / quando não usar\n5. Próximos passos ou aprofundamento{constraints_text}\n\n{expertise_modifier}"
        ),

        "direct_answer" => format!(
            "{expertise_role}\n\nResponda exatamente ao pedido do usuário, de forma curta e direta: {original_input}\n\nREGRAS:\n- Se o usuário pediu confirmação, responda apenas a confirmação necessária.\n- Não explique conceitos que não foram perguntados.\n- Não transforme a resposta em tutorial.\n- Use no máximo 3 frases, salvo se o usuário pedir detalhes.\n\n{expertise_modifier}"
        ),

        _ => format!(
            "{expertise_role}\n\nExecute com máxima qualidade: {original_input}{constraints_text}\n\n{expertise_modifier}"
        ),
    }
}

fn assemble_media_prompt(
    media_prompt: &str,
    domain: &DomainEnrichment,
    expertise: &ExpertiseLevel,
) -> String {
    format!(
        "{role}\n\n{media_prompt}\n\nCONTEXTO DO DOMÍNIO:\n- Domínio: {domain}\n- Público: {audience}\n\nRESTRIÇÕES DO DOMÍNIO:\n- {constraints}\n\n{modifier}",
        role = expertise.role_prefix(),
        modifier = expertise.prompt_modifier(),
        domain = domain.domain,
        audience = domain.audience,
        constraints = join_or_default(&domain.constraints, "sem restricoes adicionais"),
    )
}

pub fn detect_sector(
    input: &str,
) -> Option<(&'static str, &'static str, Vec<&'static str>, &'static str)> {
    let l = input.to_lowercase();

    let health = [
        "dentista",
        "médico",
        "medico",
        "clinica",
        "clínica",
        "hospital",
        "odontologia",
        "psicólogo",
        "psicologo",
        "fisio",
        "nutricionista",
        "farmácia",
        "farmacia",
        "veterinário",
        "veterinario",
        "terapeuta",
        "enfermeiro",
    ];

    let legal = [
        "advogado",
        "advogada",
        "advocacia",
        "jurídico",
        "juridico",
        "direito",
        "oab",
        "escritório jurídico",
        "processo",
        "contrato",
    ];

    let food = [
        "restaurante",
        "lanchonete",
        "chef",
        "cardápio",
        "cardapio",
        "delivery",
        "comida",
        "gastronomia",
        "bar",
        "cafeteria",
        "padaria",
        "confeitaria",
        "buffet",
    ];

    let beauty = [
        "salão",
        "salao",
        "cabeleireiro",
        "estética",
        "estetica",
        "manicure",
        "maquiagem",
        "barbearia",
        "spa",
        "nail",
        "lashes",
        "sobrancelha",
        "depilação",
        "depilacao",
    ];

    let education = [
        "professor",
        "escola",
        "colégio",
        "colegio",
        "curso",
        "ensino",
        "educação",
        "educacao",
        "pedagogia",
        "tutor",
        "mentoria",
        "treinamento",
    ];

    let realestate = [
        "imóveis",
        "imoveis",
        "imobiliária",
        "imobiliaria",
        "corretor",
        "construtora",
        "arquiteto",
        "engenheiro civil",
        "reforma",
        "decoração",
        "decoracao",
    ];

    let finance = [
        "contador",
        "contabilidade",
        "financeiro",
        "finanças",
        "financas",
        "investimento",
        "seguro",
        "corretora",
        "crédito",
    ];

    let tech = [
        "startup",
        "saas",
        "aplicativo",
        "plataforma",
        "api",
        "desenvolvedor",
        "programador",
        "sistema",
        "software",
    ];

    if health.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "saude",
            "pacientes buscando atendimento de saúde",
            vec![
                "linguagem acolhedora e profissional",
                "sem promessa de resultado ou cura",
                "mencionar registro profissional (CRM/CRO/CFF)",
                "LGPD para dados de pacientes",
            ],
            "acolhimento, especialidades, agendamento, convenios",
        ));
    }

    if legal.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "juridico",
            "clientes buscando orientação jurídica confiável",
            vec![
                "sem promessa de resultado judicial",
                "linguagem clara e profissional",
                "respeitar Código de Ética da OAB",
                "captação ética conforme resolução OAB",
            ],
            "credibilidade, areas de atuacao, captacao etica, OAB",
        ));
    }

    if food.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "gastronomia",
            "clientes buscando experiência gastronômica",
            vec![
                "destacar diferenciais e especialidades",
                "facilidade para pedido/reserva em destaque",
                "horário de funcionamento visível",
                "fotos dos pratos em destaque",
            ],
            "cardapio, ambiente, delivery, reserva, experiencia",
        ));
    }

    if beauty.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "beleza",
            "clientes buscando serviços de beleza e estética",
            vec![
                "portfólio visual obrigatório",
                "agendamento online em destaque",
                "depoimentos de clientes com fotos",
            ],
            "servicos, agendamento, portfolio, transformacao",
        ));
    }

    if education.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "educacao",
            "alunos e responsáveis buscando formação",
            vec![
                "credenciais e metodologia em destaque",
                "depoimentos de alunos",
                "processo de inscrição claro",
            ],
            "metodologia, resultados, credenciais, inscricao",
        ));
    }

    if realestate.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "imoveis",
            "compradores e locatários de imóveis",
            vec![
                "CRECI em destaque",
                "portfólio de imóveis com fotos",
                "calculadora de financiamento se aplicável",
            ],
            "portfolio, localizacao, CRECI, busca de imoveis",
        ));
    }

    if finance.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "financeiro",
            "clientes buscando serviços financeiros",
            vec![
                "registro em órgão regulador (CVM/SUSEP/CRC)",
                "sem promessa de rentabilidade",
                "linguagem clara sobre riscos",
            ],
            "confianca, seguranca, expertise, regulatorio",
        ));
    }

    if tech.iter().any(|keyword| l.contains(keyword)) {
        return Some((
            "tecnologia",
            "usuários e decisores técnicos",
            vec![
                "explicitar requisitos não-funcionais",
                "priorizar manutenibilidade e escalabilidade",
                "documentação clara",
            ],
            "arquitetura, escalabilidade, seguranca, UX",
        ));
    }

    None
}

pub fn is_creative_request(input: &str) -> bool {
    let l = input.to_lowercase();
    [
        "me surpreenda",
        "me surpreende",
        "me impressione",
        "me impressiona",
        "surpreenda",
        "impressione",
        "algo criativo",
        "seja criativo",
        "crie algo",
        "faça algo",
        "faz algo",
        "invente",
        "improvise",
        "me surpenda",
        "me surpeenda",
        "me surpreeda",
        "surpreenda-me",
        "impressione-me",
        "você decide",
        "voce decide",
        "escolha você",
        "escolha voce",
        "mostre algo diferente",
        "algo diferente",
        "algo único",
        "algo unico",
        "algo especial",
    ]
    .iter()
    .any(|term| l.contains(term))
}

pub fn is_direct_answer_request(input: &str) -> bool {
    let l = input.to_lowercase();
    let word_count = l.split_whitespace().count();
    let direct_terms = [
        "responda se",
        "responda si",
        "responda só",
        "responda so",
        "só responda",
        "so responda",
        "apenas responda",
        "está ok",
        "esta ok",
        "tá ok",
        "ta ok",
        "está certo",
        "esta certo",
        "sim ou não",
        "sim ou nao",
        "diga ok",
        "fale ok",
    ];

    direct_terms.iter().any(|term| l.contains(term))
        || (word_count <= 5 && contains_any(&l, &["ok", "certo", "funciona", "responda"]))
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
        assert_eq!(response.parameters.expertise_level, "SeniorDev");
        assert!(response.prompt.contains("advocacia"));
    }

    #[test]
    fn applies_expertise_to_parameters() {
        let pipeline = PromptPipeline::default();
        let response = pipeline.engineer_with_expertise(
            EngineerRequest {
                input: "faz um site pra minha empresa de advocacia".to_string(),
                provider: Some("claude".to_string()),
            },
            ExpertiseLevel::Especialista,
            None,
        );

        assert_eq!(response.parameters.expertise_level, "Especialista");
        assert!(response.prompt.contains("Use terminologia avançada"));
        assert!(!response.prompt.contains("Escreva um prompt"));
    }

    #[test]
    fn flags_short_requests_as_ambiguous() {
        let ambiguity =
            resolve_ambiguity("faz um site", "faz um site", &detect_intent("faz um site"));
        assert!(ambiguity.is_ambiguous);
        assert!(!ambiguity.clarifying_questions.is_empty());
    }

    #[test]
    fn detects_health_sector_when_domain_is_general() {
        let pipeline = PromptPipeline::default();
        let response = pipeline.engineer(EngineerRequest {
            input: "sou dentista e quero um site".to_string(),
            provider: None,
        });

        assert_eq!(response.domain.domain, "saude");
        assert!(response
            .domain
            .constraints
            .contains(&"LGPD para dados de pacientes".to_string()));
        assert!(response.prompt.contains("Crie um site completo"));
        assert!(!response.prompt.contains("Escreva um prompt"));
    }

    #[test]
    fn detects_food_sector_for_restaurant_site() {
        let pipeline = PromptPipeline::default();
        let response = pipeline.engineer(EngineerRequest {
            input: "tenho um restaurante e quero um site".to_string(),
            provider: None,
        });

        assert_eq!(response.domain.domain, "gastronomia");
        assert!(response.prompt.contains("DOMÍNIO: gastronomia"));
    }

    #[test]
    fn detects_open_creative_request() {
        let pipeline = PromptPipeline::default();
        let response = pipeline.engineer(EngineerRequest {
            input: "me surpreenda".to_string(),
            provider: None,
        });

        assert_eq!(response.intent.primary, "open_creative");
        assert_eq!(response.technique.technique, "creative_engine");
        assert_eq!(response.parameters.temperature, 0.92);
        assert!(response
            .prompt
            .contains("O usuário pediu para ser surpreendido"));
        assert!(!response.prompt.contains("Escreva um prompt"));
    }
}
