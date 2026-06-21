#!/usr/bin/env bash
set -e

FORGE_REPO_URL="${FORGE_REPO_URL:-https://github.com/rafaelferreira2312/forge.git}"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.forge}"

step() { echo -e "\n\033[0;33m>\033[0m \033[1m$1\033[0m"; }
ok() { echo -e "  \033[0;32mOK\033[0m $1"; }

step "Verificando Xcode Tools"
xcode-select --install 2>/dev/null || true

step "Verificando Homebrew"
if ! command -v brew >/dev/null 2>&1; then
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi
ok "Homebrew ok"

step "Instalando dependencias"
brew install openssl pkg-config sqlite git curl
ok "Dependencias ok"

step "Instalando Rust"
if ! command -v rustc >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi
ok "Rust: $(rustc --version)"

if [[ "$(uname -m)" == "arm64" ]]; then
  OPENSSL_PREFIX="$(brew --prefix openssl)"
  grep -q "OPENSSL_DIR=$OPENSSL_PREFIX" "$HOME/.zshrc" 2>/dev/null || {
    echo "export OPENSSL_DIR=$OPENSSL_PREFIX" >> "$HOME/.zshrc"
    echo "export PKG_CONFIG_PATH=\"$OPENSSL_PREFIX/lib/pkgconfig\"" >> "$HOME/.zshrc"
  }
  export OPENSSL_DIR="$OPENSSL_PREFIX"
  export PKG_CONFIG_PATH="$OPENSSL_PREFIX/lib/pkgconfig"
  ok "Apple Silicon configurado"
fi

step "Instalando Ollama"
if ! command -v ollama >/dev/null 2>&1; then
  brew install ollama
fi

RAM_GB=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
if [ "$RAM_GB" -ge 16 ]; then
  OLLAMA_MODEL="llama3.1:8b"
elif [ "$RAM_GB" -ge 8 ]; then
  OLLAMA_MODEL="llama3.2:3b"
else
  OLLAMA_MODEL="phi3:mini"
fi

step "Baixando modelo Ollama ($OLLAMA_MODEL) - $RAM_GB GB RAM detectados"
brew services start ollama || (ollama serve >/dev/null 2>&1 &)
sleep 3
ollama pull "$OLLAMA_MODEL"
ok "Modelo pronto"

step "Compilando Forge"
if [ -d "$INSTALL_DIR/.git" ]; then
  git -C "$INSTALL_DIR" pull
else
  rm -rf "$INSTALL_DIR"
  git clone "$FORGE_REPO_URL" "$INSTALL_DIR"
fi
cd "$INSTALL_DIR"
cp -n .env.example .env 2>/dev/null || true
cargo build --release --quiet
sudo ln -sf "$INSTALL_DIR/target/release/forge" /usr/local/bin/forge
ok "Forge compilado"

step "Iniciando"
(forge >/dev/null 2>&1 &)
sleep 2
open http://localhost:3000

echo ""
echo "Forge pronto em http://localhost:3000"
