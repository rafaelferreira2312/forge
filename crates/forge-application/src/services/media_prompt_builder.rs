use forge_core::{ExpertiseLevel, MediaOutputType};

pub struct MediaPromptBuilder;

impl MediaPromptBuilder {
    pub fn detect_media_intent(input: &str) -> Option<MediaOutputType> {
        let lower = input.to_lowercase();

        if lower.contains("imagem")
            || lower.contains("foto")
            || lower.contains("image")
            || lower.contains("ilustração")
            || lower.contains("desenho")
            || lower.contains("logo")
            || lower.contains("banner")
            || lower.contains("thumbnail")
        {
            return Some(MediaOutputType::Image);
        }

        if lower.contains("vídeo")
            || lower.contains("video")
            || lower.contains("animação")
            || lower.contains("reels")
            || lower.contains("shorts")
            || lower.contains("clipe")
        {
            return Some(MediaOutputType::Video);
        }

        if lower.contains("música")
            || lower.contains("musica")
            || lower.contains("áudio")
            || lower.contains("audio")
            || lower.contains("podcast")
            || lower.contains("narração")
            || lower.contains("som")
            || lower.contains("trilha")
        {
            return Some(MediaOutputType::Audio);
        }

        if lower.contains("design")
            || lower.contains("ui")
            || lower.contains("ux")
            || lower.contains("interface")
            || lower.contains("layout")
            || lower.contains("wireframe")
            || lower.contains("protótipo")
            || lower.contains("mockup")
        {
            return Some(MediaOutputType::Design);
        }

        None
    }

    pub fn build(
        input: &str,
        media_type: &MediaOutputType,
        expertise: &ExpertiseLevel,
        domain_context: Option<&str>,
    ) -> String {
        let base = match media_type {
            MediaOutputType::Image => Self::image_template(input, expertise),
            MediaOutputType::Video => Self::video_template(input, expertise),
            MediaOutputType::Audio => Self::audio_template(input, expertise),
            MediaOutputType::Design => Self::design_template(input, expertise),
            _ => format!("Processe o seguinte pedido:\n{}", input),
        };
        let domain = domain_context
            .map(|context| format!("\n\nContexto adicional do domínio:\n{}", context))
            .unwrap_or_default();

        format!(
            "{}{}\n\nNível do usuário: {}\n{}",
            base,
            domain,
            expertise.role_prefix(),
            expertise.prompt_modifier()
        )
    }

    fn image_template(input: &str, expertise: &ExpertiseLevel) -> String {
        match expertise {
            ExpertiseLevel::Leigo | ExpertiseLevel::Junior => format!(
                "Crie uma imagem baseada neste pedido: {}\n\nDescreva visualmente o que deve aparecer, o estilo (foto, ilustração, cartoon), as cores principais e o clima da imagem.",
                input
            ),
            ExpertiseLevel::SeniorDev | ExpertiseLevel::Pleno => format!(
                "Gere um prompt detalhado para geração de imagem baseado em: {}\n\nInclua obrigatoriamente:\n- Estilo visual (fotorrealista, ilustração, 3D render...)\n- Iluminação (natural, estúdio, hora dourada...)\n- Composição (perspectiva, enquadramento, profundidade)\n- Paleta de cores (tons dominantes)\n- Formato de saída (aspect ratio: 16:9, 1:1, 9:16)\n- Negative prompts (o que NÃO deve aparecer)\n- Nível de detalhe e qualidade",
                input
            ),
            ExpertiseLevel::Engenheiro | ExpertiseLevel::Especialista => format!(
                "Construa um prompt técnico de geração de imagem para: {}\n\nEspecifique:\n- Modelo recomendado (DALL-E 3, Midjourney v6, SDXL, Flux)\n- Parâmetros técnicos: CFG scale, steps, sampler\n- Estilo artístico com referência de artista ou escola\n- Configurações de câmera: lente (mm), abertura (f/), ISO\n- Color grading: temperatura, saturação, contraste\n- Composição técnica: regra dos terços, golden ratio\n- Negative prompts detalhados\n- Aspect ratio e resolução alvo",
                input
            ),
            _ => format!("Crie uma imagem para: {}", input),
        }
    }

    fn video_template(input: &str, expertise: &ExpertiseLevel) -> String {
        match expertise {
            ExpertiseLevel::Leigo | ExpertiseLevel::Junior => format!(
                "Crie um vídeo baseado em: {}\n\nDescreva a cena, o que acontece, o estilo visual e a duração aproximada.",
                input
            ),
            ExpertiseLevel::SeniorDev | ExpertiseLevel::Pleno => format!(
                "Gere prompt para vídeo baseado em: {}\n\nInclua:\n- Duração (segundos)\n- Movimento de câmera (pan, tilt, zoom, estático)\n- Estilo visual (cinematográfico, documental, motion graphics)\n- Iluminação e color grade\n- Trilha sonora (tom, BPM, instrumento principal)\n- Formato: horizontal (16:9), vertical (9:16), quadrado (1:1)\n- Ritmo de edição: lento, médio, acelerado",
                input
            ),
            ExpertiseLevel::Engenheiro | ExpertiseLevel::Especialista => format!(
                "Prompt técnico de produção de vídeo para: {}\n\nEspecifique:\n- Ferramenta alvo: Sora, Runway ML Gen-3, Pika, Kling\n- Frame rate: 24fps (cinemático), 30fps (digital), 60fps (esportivo)\n- Movimento: câmera lenta (120fps), time-lapse, hyperlapse\n- Direção de fotografia: lente, profundidade de campo\n- Color grade: LUT recomendado, temperatura de cor (K)\n- Tratamento de áudio: frequências, compressão, reverb\n- Sequência de cenas com timing por cena\n- Formato de exportação e codec",
                input
            ),
            _ => format!("Crie um vídeo para: {}", input),
        }
    }

    fn audio_template(input: &str, expertise: &ExpertiseLevel) -> String {
        match expertise {
            ExpertiseLevel::Leigo | ExpertiseLevel::Junior => format!(
                "Crie áudio/música para: {}\n\nDescreva o estilo (animado, calmo, épico), a duração e o uso (fundo de vídeo, música principal, narração).",
                input
            ),
            ExpertiseLevel::SeniorDev | ExpertiseLevel::Pleno => format!(
                "Gere prompt para produção de áudio baseado em: {}\n\nInclua:\n- Gênero musical e subgênero\n- BPM e compasso\n- Key e modo (maior = alegre, menor = tenso)\n- Instrumentos principais\n- Estrutura: intro / verso / refrão / bridge / outro\n- Duração total\n- Referência de artista ou estilo sonoro\n- Uso final (trilha, podcast, narração, efeito)",
                input
            ),
            ExpertiseLevel::Engenheiro | ExpertiseLevel::Especialista => format!(
                "Especificação técnica de produção de áudio para: {}\n\nDefina:\n- Ferramenta: Suno, Udio, ElevenLabs, Bark, MusicGen\n- BPM exato e tempo (4/4, 3/4, 6/8)\n- Escala e progressão de acordes (ex: I-V-vi-IV em C)\n- Timbre: síntetizadores (analógico/digital), amostras live\n- Mixagem: frequências dominantes, panning, compressão\n- Masterização: LUFS alvo (-14 streaming, -9 club)\n- Formato de saída: WAV 44.1kHz 16bit ou 24bit\n- Referências de produção específicas",
                input
            ),
            _ => format!("Produza áudio para: {}", input),
        }
    }

    fn design_template(input: &str, expertise: &ExpertiseLevel) -> String {
        match expertise {
            ExpertiseLevel::Leigo | ExpertiseLevel::Junior => format!(
                "Crie um design para: {}\n\nDescreva o visual que você imagina, as cores preferidas e onde vai ser usado.",
                input
            ),
            ExpertiseLevel::SeniorDev | ExpertiseLevel::Pleno => format!(
                "Gere especificação de design para: {}\n\nInclua:\n- Tipo: logo, UI, banner, apresentação, infográfico\n- Estilo visual: minimalista, bold, corporativo, playful\n- Paleta de cores (primária, secundária, neutros, accent)\n- Tipografia: display face + body face\n- Grid e espaçamento\n- Tamanhos de entrega necessários\n- Formato de arquivo: SVG, PNG, PDF, Figma",
                input
            ),
            ExpertiseLevel::Engenheiro | ExpertiseLevel::Especialista => format!(
                "Briefing técnico de design para: {}\n\nEspecifique:\n- Sistema de design: tokens de cor, tipografia, espaçamento\n- Tipografia técnica: família, peso, tamanho, line-height, tracking\n- Paleta completa: hex + RGB + HSL + CMYK para print\n- Grid system: colunas, gutter, margin, breakpoints\n- Componentes necessários e estados (default, hover, active, disabled)\n- Acessibilidade: contraste WCAG AA/AAA, foco visível\n- Variações: light/dark mode, responsive, print\n- Especificação de exportação por plataforma",
                input
            ),
            _ => format!("Crie um design para: {}", input),
        }
    }
}
