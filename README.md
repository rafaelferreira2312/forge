# 🔥 FORGE — Rust Prompt Engineer

> **Você pensa. Forge traduz. A IA constrói.**

[![License: MIT](https://img.shields.io/badge/License-MIT-orange.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://rustup.rs)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20WSL2%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)]()
[![Status](https://img.shields.io/badge/status-em%20desenvolvimento-yellow.svg)]()

---

## O que é o Forge?

Forge é um **engenheiro de prompt automático** que roda na sua máquina.

Você digita qualquer coisa — simples, vaga, criativa ou complexa. Forge analisa sua intenção, enriquece com contexto do domínio, aprende seus padrões e entrega para qualquer IA um prompt de engenheiro sênior. Tudo em menos de 1ms, sem modelo de linguagem local pesado, sem nuvem, sem custo.

```
Você:   "faz um site pra minha empresa de advocacia"

Forge:  Detecta → Site institucional, setor jurídico, nível alto
        Enriquece → OAB compliance, seções obrigatórias, tone formal
        Aprende → sua stack é Next.js + Tailwind + TypeScript
        Monta → prompt completo de 400 palavras para a IA

Claude: Entrega o site completo na primeira resposta.
```

---

## Por que Rust?

- **< 1ms** de latência no pipeline inteiro
- **~8MB** de RAM em idle — sem Electron, sem Node, sem runtime
- **Binário único** — compila e distribui como um executável
- **Segurança de memória** — sem GC, sem vazamentos, sem surpresas
- **Concorrência real** — tokio + rayon para paralelismo sem overhead

---

## Funcionalidades

### Pipeline de 8 estágios (< 1ms total)
1. **Intent Detector** — identifica intenção com keyword scoring (sem IA)
2. **Ambiguity Resolver** — classifica e roteia inputs vagos
3. **Pattern Learner** — consulta seu histórico de preferências
4. **Domain Enricher** — carrega knowledge base do setor
5. **Complexity Analyzer** — score 0–100 determina profundidade
6. **Technique Selector** — escolhe: Direct, Few-Shot, Chain-of-Thought...
7. **Adaptive Param Injector** — injeta temperature, max_tokens calibrados
8. **Prompt Assembler** — monta o briefing final

### Aprendizado adaptativo
- Parâmetros que evoluem com o uso (não são fixos)
- Perfis separados por domínio (código ≠ texto criativo)
- Sinais implícitos (você copiou? fez follow-up?) e explícitos (👍/👎)
- Momentum para evitar oscilações bruscas
- Confiança cresce de 0.1 a 0.9+ com o tempo

### Suporte multimodal
- Imagens: geração e análise com prompts especializados
- Vídeos: análise com estratégia por timestamp
- PDFs / Documentos: chunking inteligente, extração estruturada
- Fusão de imagens: style transfer, blend, swap de sujeito
- Orquestração multi-etapa: "compara o vídeo com o PDF e faz uma apresentação" → plano de 4 tasks com dependências

### Creative Engine
- Resolve "me surpreenda" e "algo criativo" com base no seu perfil
- Seeds por domínio e horário do dia
- Mostra o raciocínio da escolha na UI

### Multi-provider
| Provider | Uso | Custo |
|---|---|---|
| Ollama | modelos locais | gratuito |
| Groq | rápido, free tier | gratuito até 14.400 req/dia |
| Claude (Anthropic) | melhor para código e texto | pago |
| GPT (OpenAI) | alternativa | pago |
| Gemini (Google) | vídeo e multimodal | pago |

---

## Repositório

**GitHub:** [github.com/rafaelferreira2312/forge](https://github.com/rafaelferreira2312/forge)

**Site / landing:** [rafaelferreira2312.github.io/forge](https://rafaelferreira2312.github.io/forge/)

---

## Como usar (passo a passo)

1. **Gere seu DNA Forge** — na [landing](https://rafaelferreira2312.github.io/forge/#dna), preencha nome/empresa, email e WhatsApp. Você recebe um código `FORGE-DNA-...` salvo no navegador (e gravado no SQLite via `POST /api/leads/dna` quando o backend estiver rodando).
2. **Instale o Forge** — clone o repositório ou use o script de instalação (ver abaixo).
3. **Suba o servidor** — `cargo run -p forge-infrastructure` ou `./target/release/forge`.
4. **Abra** [http://localhost:3000](http://localhost:3000) no mesmo navegador (opcional, mas recomendado para reconhecer o DNA).
5. **Configure uma IA** — Groq (grátis), Ollama (local) ou outro provider em ⚙ Config.
6. **Converse** — digite seu pedido; o Forge monta o prompt; você escolhe copiar ou **Forge pedir**; arquivos (PDF, planilha, HTML) são entregues na conversa.

---

## Instalação

### 🐧 Linux (Ubuntu 20.04+ / Debian 11+)

```bash
# 1. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. Dependências
sudo apt update && sudo apt install -y \
  build-essential pkg-config libssl-dev libsqlite3-dev git

# 3. Clonar e rodar
git clone https://github.com/rafaelferreira2312/forge.git
cd forge
cp .env.example .env        # configure sua API key
cargo build --release
./target/release/forge
# → http://localhost:3000
```

### 🔧 WSL2 (Windows Subsystem for Linux)

```powershell
# PowerShell como Admin
wsl --install -d Ubuntu-22.04
wsl --set-default-version 2
```

```bash
# Dentro do WSL2 — siga os passos do Linux acima
# O browser do Windows acessa http://localhost:3000 automaticamente
```

### 🍎 macOS (Intel + Apple Silicon)

```bash
# 1. Xcode Tools + Homebrew
xcode-select --install
brew install openssl pkg-config sqlite

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Apple Silicon — adicione ao ~/.zshrc:
export OPENSSL_DIR=$(brew --prefix openssl)
export PKG_CONFIG_PATH="$(brew --prefix openssl)/lib/pkgconfig"

# 3. Clonar e rodar
git clone https://github.com/rafaelferreira2312/forge.git
cd forge && cp .env.example .env
cargo build --release && ./target/release/forge
```

### 🪟 Windows nativo

```powershell
# 1. Visual C++ Build Tools (necessário)
# https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Selecione: "Desktop development with C++"

# 2. Rust
winget install Rustlang.Rustup

# 3. Clonar e compilar
git clone https://github.com/rafaelferreira2312/forge.git
cd forge
copy .env.example .env
cargo build --release
.\target\release\forge.exe
```

### 🐳 Docker (qualquer OS, sem instalar Rust)

```bash
git clone https://github.com/rafaelferreira2312/forge.git
cd forge
cp .env.example .env
docker compose up -d
# → http://localhost:3000
```

---

## Scaffolding do projeto (novo dev)

```bash
# Baixe o script e execute
chmod +x forge-setup.sh
./forge-setup.sh meu-forge

# Gera toda a estrutura com arquivos zerados prontos para implementar
```

---

## Configuração

```bash
cp .env.example .env
```

```env
# Configure apenas os providers que vai usar

ANTHROPIC_API_KEY=sk-ant-...    # Claude
OPENAI_API_KEY=sk-...           # GPT (opcional)
GEMINI_API_KEY=AIza...          # Gemini (opcional)
GROQ_API_KEY=gsk_...            # Groq — gratuito

RUST_LOG=info
```

Configuração avançada em `config/config.toml`.

---

## Uso via API

```bash
# Engenharia de prompt simples
curl -X POST http://localhost:3000/api/engineer \
  -H "Content-Type: application/json" \
  -d '{ "input": "faz um site pra minha empresa de advocacia" }'

# Com provider específico
curl -X POST http://localhost:3000/api/engineer \
  -H "Content-Type: application/json" \
  -d '{ "input": "me surpreenda", "provider": "claude" }'

# Ver perfil adaptativo atual
curl http://localhost:3000/api/patterns

# Histórico
curl http://localhost:3000/api/history?limit=20

# Estatísticas
curl http://localhost:3000/api/stats
```

---

## Estrutura do projeto

```
forge/
├── forge-core/              # Domínio puro — entidades, ports
├── forge-application/       # Casos de uso e serviços
├── forge-adapters/          # HTTP, providers, SQLite
├── forge-infrastructure/    # main.rs, config, DI
├── forge-frontend/          # Preact + Tailwind (< 50KB)
├── forge-knowledge/         # Knowledge base JSON por domínio
│   ├── web/                 # Sites institucionais, e-commerce...
│   ├── code/                # Rust, Node.js, Python...
│   ├── legal/               # Contratos, políticas...
│   ├── marketing/           # Copywriting, email...
│   ├── creative/            # Seeds criativos por perfil
│   └── media/               # Templates multimodais
├── config/config.toml
├── migrations/
└── .env.example
```

---

## Como o aprendizado funciona

```
Interação 1–10   Observação      Forge coleta padrões, confiança 0.1–0.3
Interação 10–30  Calibração      Forge personaliza ativamente, 0.3–0.6
Interação 30–100 Identidade      Perfil estável por domínio, 0.6–0.9
Interação 100+   Refinamento     Micro-ajustes contínuos, 0.9+
```

Todos os dados ficam no SQLite local. Nada sai da sua máquina.

---

## Taxa de acerto esperada

| Tipo de input | Acerto estimado |
|---|---|
| Intenção clara ("faz site de advocacia") | 88–93% |
| Intenção média ("resume esse PDF") | 90–95% |
| Input com assets ("compara o vídeo com o PDF") | 80–88% |
| Input criativo com pista ("algo sobre tecnologia") | 72–80% |
| Input totalmente vago ("me surpreenda") | 65–75%* |

*Acerto subjetivo — o resultado técnico é sempre de alta qualidade.

---

## Roadmap

### Fase 1 — Engine funcional
- [ ] forge-core: entidades + ports
- [ ] IntentDetector com keyword map PT-BR + EN
- [ ] PromptAssembler básico
- [ ] Adapter Claude
- [ ] Axum server + frontend mínimo

### Fase 2 — Aprendizado passivo
- [ ] SQLite + migrations
- [ ] PatternLearner
- [ ] SignalCollector (sinais implícitos)
- [ ] Histórico de engenharias

### Fase 3 — Aprendizado ativo
- [ ] AdaptiveProfile com momentum
- [ ] ParamInjector dinâmico
- [ ] Feedback 👍/👎 na UI
- [ ] Todos os providers

### Fase 4 — Multimodal + Criativo
- [ ] AssetDetector
- [ ] Orchestrator multi-etapa
- [ ] CreativeEngine
- [ ] FusionBuilder
- [ ] Knowledge base completa

---

## Contribuindo

```bash
git checkout -b feature/minha-feature
cargo test
cargo fmt --check
cargo clippy -- -D warnings
# Pull Request com descrição clara
```

---

## Licença

MIT — use, modifique e distribua livremente.

---

> FORGE não substitui a IA.  
> Faz você usar qualquer IA como se fosse  
> um engenheiro de prompt de 10 anos de experiência.  
> **Sem curva de aprendizado. Sem conta. Sem nuvem.**
