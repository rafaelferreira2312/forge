#!/usr/bin/env bash
set -e

REPO_RAW_URL="${FORGE_REPO_RAW_URL:-https://raw.githubusercontent.com/rafaelferreira2312/forge/main}"

echo "Detectado: WSL2"
echo "Instalando via fluxo Linux..."

bash <(curl -fsSL "$REPO_RAW_URL/scripts/install-linux.sh")

echo ""
echo "No browser do Windows acesse: http://localhost:3000"
echo "O WSL2 expoe portas automaticamente para o Windows."
