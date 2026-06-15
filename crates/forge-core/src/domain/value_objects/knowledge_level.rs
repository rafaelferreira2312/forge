use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum KnowledgeLevel {
    Iniciante,
    Basico,
    Intermediario,
    #[default]
    Avancado,
    Especialista,
}

impl KnowledgeLevel {
    pub fn prompt_modifier(&self, area: &UserArea) -> String {
        match (self, area) {
            (Self::Iniciante, _) => {
                "Explique de forma simples, sem jargão técnico. \
                 Use exemplos do cotidiano. Passo a passo."
                    .into()
            }
            (Self::Basico, _) => {
                "O usuário conhece o básico. Explique o raciocínio \
                 por trás das decisões. Evite termos muito técnicos \
                 sem explicação."
                    .into()
            }
            (Self::Intermediario, _) => {
                "O usuário é funcional na área. Respostas diretas, \
                 sem tutoriais básicos. Foco em clareza e boas práticas."
                    .into()
            }
            (Self::Avancado, UserArea::Tecnologia) => {
                "O usuário é desenvolvedor/engenheiro experiente. \
                 Sem explicações óbvias. Código production-ready. \
                 Assuma conhecimento de padrões e arquitetura."
                    .into()
            }
            (Self::Avancado, UserArea::Saude) => {
                "O usuário é profissional de saúde experiente. \
                 Use terminologia clínica correta. Sem simplificações \
                 desnecessárias. Referencie protocolos quando relevante."
                    .into()
            }
            (Self::Avancado, UserArea::Juridico) => {
                "O usuário é profissional do direito. Use linguagem \
                 jurídica adequada. Referencie legislação e jurisprudência \
                 quando relevante. Sem explicações básicas de conceitos legais."
                    .into()
            }
            (Self::Avancado, UserArea::Negocios) => {
                "O usuário é executivo/gestor experiente. Foco em \
                 impacto, ROI, risco e decisão. Linguagem objetiva \
                 orientada a resultado."
                    .into()
            }
            (Self::Avancado, UserArea::Ciencias) => {
                "O usuário tem formação científica avançada. Use \
                 notação formal, terminologia da área, referencie \
                 metodologia científica quando aplicável."
                    .into()
            }
            (Self::Especialista, _) => format!(
                "O usuário é especialista em {}. Use terminologia \
                 avançada sem simplificar. Seja denso e preciso. \
                 Não repita o óbvio.",
                area.label_pt()
            ),
            _ => "Responda de forma clara e direta.".into(),
        }
    }

    pub fn role_prefix(&self, area: &UserArea) -> String {
        match (self, area) {
            (Self::Iniciante, _) => "Você é um professor paciente e didático.".into(),
            (Self::Basico, _) => "Você é um mentor experiente e acessível.".into(),
            (Self::Intermediario, _) => "Você é um profissional sênior pragmático.".into(),
            (Self::Avancado, UserArea::Tecnologia) => {
                "Você é um engenheiro de software sênior direto e preciso.".into()
            }
            (Self::Avancado, UserArea::Saude) => {
                "Você é um médico/especialista clínico sênior.".into()
            }
            (Self::Avancado, UserArea::Juridico) => {
                "Você é um advogado sênior especialista na área.".into()
            }
            (Self::Avancado, UserArea::Negocios) => {
                "Você é um consultor estratégico sênior.".into()
            }
            (Self::Avancado, UserArea::Educacao) => {
                "Você é um educador especialista com vasta experiência.".into()
            }
            (Self::Avancado, UserArea::Criativo) => {
                "Você é um diretor criativo sênior com portfólio extenso.".into()
            }
            (Self::Avancado, UserArea::Ciencias) => {
                "Você é um pesquisador/cientista sênior da área.".into()
            }
            (Self::Especialista, area) => {
                format!("Você é o maior especialista disponível em {}.", area.label_pt())
            }
            _ => "Você é um especialista qualificado na área.".into(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "iniciante" | "beginner" => Self::Iniciante,
            "basico" | "básico" | "basic" => Self::Basico,
            "intermediario" | "intermediate" | "pleno" => Self::Intermediario,
            "avancado" | "avançado" | "advanced" | "senior" | "seniordev" => Self::Avancado,
            "especialista" | "specialist" | "expert" => Self::Especialista,
            _ => Self::Avancado,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum UserArea {
    Tecnologia,
    Saude,
    Juridico,
    Negocios,
    Educacao,
    Criativo,
    Ciencias,
    #[default]
    Outro,
}

impl UserArea {
    pub fn label_pt(&self) -> &'static str {
        match self {
            Self::Tecnologia => "Tecnologia",
            Self::Saude => "Saúde",
            Self::Juridico => "Jurídico",
            Self::Negocios => "Negócios",
            Self::Educacao => "Educação",
            Self::Criativo => "Criativo / Design",
            Self::Ciencias => "Ciências",
            Self::Outro => "Outro",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tecnologia" | "tech" | "ti" => Self::Tecnologia,
            "saude" | "saúde" | "health" => Self::Saude,
            "juridico" | "jurídico" | "legal" => Self::Juridico,
            "negocios" | "negócios" | "business" => Self::Negocios,
            "educacao" | "educação" | "education" => Self::Educacao,
            "criativo" | "creative" | "design" => Self::Criativo,
            "ciencias" | "ciências" | "science" => Self::Ciencias,
            _ => Self::Outro,
        }
    }
}
