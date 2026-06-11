# FORGE - Rust Prompt Engineer

FORGE e uma versao inicial funcional de um engenheiro de prompts local escrito em Rust. Ele recebe uma solicitacao curta ou ambigua, detecta intencao e dominio, estima complexidade, escolhe uma tecnica de prompt e monta um prompt final estruturado sem chamar provedores externos.

## Workspace

- `crates/forge-core`: modelos e pipeline de engenharia de prompt.
- `crates/forge-application`: servico de aplicacao, historico e estatisticas em memoria.
- `crates/forge-adapters`: conhecimento local e perfil adaptativo mock.
- `crates/forge-infrastructure`: servidor HTTP Axum e binario `forge`.
- `forge-knowledge/`: base de conhecimento inicial em TOML.
- `config/config.toml`: configuracao inicial.
- `migrations/`: SQL reservado para persistencia futura.

## Pipeline

1. Intent Detector
2. Ambiguity Resolver
3. Domain Enricher
4. Complexity Analyzer
5. Technique Selector
6. Adaptive Param Injector
7. Prompt Assembler

## Rodando

```bash
cargo run -p forge-infrastructure
```

O servidor sobe em `http://localhost:3000`.

## Endpoints

- `GET /`: pagina HTML simples explicando o projeto.
- `POST /api/engineer`: recebe `{ "input": "...", "provider": "claude" }` e retorna intencao, dominio, complexidade, tecnica, parametros e prompt final.
- `GET /api/patterns`: perfil adaptativo local/mock.
- `GET /api/history?limit=20`: historico em memoria.
- `GET /api/stats`: estatisticas em memoria.

Exemplo:

```bash
curl -s http://localhost:3000/api/stats
curl -s -X POST http://localhost:3000/api/engineer \
  -H "content-type: application/json" \
  -d '{"input":"faz um site pra minha empresa de advocacia","provider":"claude"}'
```

Nenhuma API key e exigida para `/api/engineer`; a engenharia de prompt e local.