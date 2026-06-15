#!/usr/bin/env bash
set -e

ORANGE='\033[0;33m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

FORGE_REPO_URL="${FORGE_REPO_URL:-https://github.com/rafaelferreira2312/forge.git}"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.forge}"

step() { echo -e "\n${ORANGE}>${RESET} ${BOLD}$1${RESET}"; }
ok() { echo -e "  ${GREEN}OK${RESET} $1"; }

step "Verificando dependencias do sistema"
sudo apt-get update -qq
sudo apt-get install -y build-essential pkg-config libssl-dev libsqlite3-dev git curl
ok "Dependencias instaladas"

step "Instalando Rust"
if ! command -v rustc >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
  ok "Rust instalado: $(rustc --version)"
else
  ok "Rust ja instalado: $(rustc --version)"
fi

step "Instalando Ollama (IA local gratuita)"
if ! command -v ollama >/dev/null 2>&1; then
  curl -fsSL https://ollama.ai/install.sh | sh
  ok "Ollama instalado"
else
  ok "Ollama ja instalado"
fi

RAM_GB=$(free -g | awk '/^Mem:/{print $2}')
if [ "$RAM_GB" -ge 16 ]; then
  OLLAMA_MODEL="llama3.1:8b"
elif [ "$RAM_GB" -ge 8 ]; then
  OLLAMA_MODEL="llama3.2:3b"
else
  OLLAMA_MODEL="phi3:mini"
fi

step "Baixando modelo Ollama ($OLLAMA_MODEL) para sua maquina ($RAM_GB GB RAM)"
echo "  Isso pode levar alguns minutos na primeira vez..."
(ollama serve >/dev/null 2>&1 &)
sleep 3
ollama pull "$OLLAMA_MODEL"
ok "Modelo $OLLAMA_MODEL pronto"

step "Clonando e compilando o Forge"
if [ -d "$INSTALL_DIR/.git" ]; then
  echo "  Atualizando instalacao existente..."
  git -C "$INSTALL_DIR" pull
else
  rm -rf "$INSTALL_DIR"
  git clone "$FORGE_REPO_URL" "$INSTALL_DIR"
fi

cd "$INSTALL_DIR"
cp -n .env.example .env 2>/dev/null || true
echo "  Compilando (pode levar 3-5 min na primeira vez)..."
cargo build --release --quiet
ok "Forge compilado"

step "Criando comando global 'forge'"
sudo ln -sf "$INSTALL_DIR/target/release/forge" /usr/local/bin/forge
ok "Comando 'forge' disponivel globalmente"

step "Iniciando o Forge"
(forge >/dev/null 2>&1 &)
sleep 2

echo ""
echo -e "${GREEN}====================================${RESET}"
echo -e "${BOLD}  Forge instalado com sucesso!${RESET}"
echo -e "${GREEN}====================================${RESET}"
echo ""
echo -e "  Abrindo em: ${BOLD}http://localhost:3000${RESET}"
echo ""

if command -v xdg-open >/dev/null 2>&1; then
  xdg-open http://localhost:3000 >/dev/null 2>&1 &
fi
