use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ExpertiseLevel {
    Leigo,
    Junior,
    Pleno,
    #[default]
    SeniorDev,
    Engenheiro,
    Consultor,
    Especialista,
}

impl ExpertiseLevel {
    pub fn prompt_modifier(&self) -> &'static str {
        match self {
            Self::Leigo => {
                "Explique tudo em linguagem simples, sem jargão técnico. \
                 Use analogias do cotidiano. Passo a passo detalhado. \
                 Nunca assuma conhecimento prévio."
            }
            Self::Junior => {
                "O usuário conhece o básico. Explique o porquê de cada \
                 decisão importante. Inclua comentários no código. \
                 Evite over-engineering."
            }
            Self::Pleno => {
                "O usuário é funcional. Foco em clareza e boas práticas. \
                 Código limpo sem over-engineering. \
                 Comentários apenas onde a lógica não é óbvia."
            }
            Self::SeniorDev => {
                "O usuário é engenheiro sênior. Sem explicações óbvias. \
                 Entregue código production-ready, sem placeholders, \
                 sem introduções. Comentários só onde a lógica é não-trivial. \
                 Assuma conhecimento de padrões e boas práticas."
            }
            Self::Engenheiro => {
                "O usuário pensa em arquitetura de sistemas. \
                 Justifique decisões de design. Apresente trade-offs. \
                 Considere escalabilidade, manutenção e observabilidade. \
                 Mencione padrões relevantes (CQRS, event sourcing, etc) \
                 quando aplicável."
            }
            Self::Consultor => {
                "Combine visão técnica com impacto de negócio. \
                 Para cada decisão técnica relevante: apresente pros, cons, \
                 custo estimado e risco. O usuário precisa comunicar \
                 decisões para stakeholders não-técnicos também."
            }
            Self::Especialista => {
                "Use terminologia avançada do domínio sem simplificar. \
                 O usuário domina o tema completamente. \
                 Referencie padrões, normas e melhores práticas do setor. \
                 Seja preciso e denso — não repita o óbvio."
            }
        }
    }

    pub fn role_prefix(&self) -> &'static str {
        match self {
            Self::Leigo => "Você é um professor paciente e didático.",
            Self::Junior => "Você é um mentor técnico experiente.",
            Self::Pleno => "Você é um desenvolvedor sênior pragmático.",
            Self::SeniorDev => "Você é um engenheiro sênior direto e preciso.",
            Self::Engenheiro => "Você é um arquiteto de sistemas experiente.",
            Self::Consultor => "Você é um consultor sênior técnico-estratégico.",
            Self::Especialista => "Você é o maior especialista disponível neste domínio.",
        }
    }

    pub fn temperature_modifier(&self) -> f32 {
        match self {
            Self::Leigo => 0.1,
            Self::Junior => 0.05,
            Self::Pleno => 0.0,
            Self::SeniorDev => -0.05,
            Self::Engenheiro => -0.1,
            Self::Consultor => 0.05,
            Self::Especialista => -0.15,
        }
    }

    pub fn max_tokens_modifier(&self) -> i32 {
        match self {
            Self::Leigo => -400,
            Self::Junior => -200,
            Self::Pleno => 0,
            Self::SeniorDev => 400,
            Self::Engenheiro => 800,
            Self::Consultor => 600,
            Self::Especialista => 1000,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "leigo" | "beginner" | "iniciante" => Self::Leigo,
            "junior" | "jr" | "basico" | "básico" | "basic" => Self::Junior,
            "pleno" | "mid" | "intermediario" | "intermediate" => Self::Pleno,
            "senior" | "senior_dev" | "sr" | "avancado" | "avançado" | "advanced" => {
                Self::SeniorDev
            }
            "engenheiro" | "engineer" | "architect" => Self::Engenheiro,
            "consultor" | "consultant" => Self::Consultor,
            "especialista" | "expert" | "specialist" => Self::Especialista,
            _ => Self::SeniorDev,
        }
    }

    pub fn as_config_value(&self) -> &'static str {
        match self {
            Self::Leigo => "Leigo",
            Self::Junior => "Junior",
            Self::Pleno => "Pleno",
            Self::SeniorDev => "SeniorDev",
            Self::Engenheiro => "Engenheiro",
            Self::Consultor => "Consultor",
            Self::Especialista => "Especialista",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Leigo => "Leigo",
            Self::Junior => "Júnior",
            Self::Pleno => "Pleno",
            Self::SeniorDev => "Sênior Dev",
            Self::Engenheiro => "Engenheiro",
            Self::Consultor => "Consultor",
            Self::Especialista => "Especialista",
        }
    }
}
