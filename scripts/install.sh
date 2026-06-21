#!/usr/bin/env bash
# FORGE - Instalador Universal
# curl -fsSL https://raw.githubusercontent.com/rafaelferreira2312/forge/main/scripts/install.sh | bash

set -e

RED='\033[0;31m'
ORANGE='\033[0;33m'
BOLD='\033[1m'
RESET='\033[0m'

REPO_RAW_URL="${FORGE_REPO_RAW_URL:-https://raw.githubusercontent.com/rafaelferreira2312/forge/main}"

echo ""
echo -e "${ORANGE}  FORGE - Instalador${RESET}"
echo -e "${ORANGE}  Voce pensa. Forge traduz. A IA constroi.${RESET}"
echo ""

OS=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if grep -qi microsoft /proc/version 2>/dev/null; then
    OS="wsl"
  else
    OS="linux"
  fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  OS="mac"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  OS="windows"
fi

echo -e "Sistema detectado: ${BOLD}${OS:-desconhecido}${RESET}"
echo ""

case "$OS" in
  linux) bash <(curl -fsSL "$REPO_RAW_URL/scripts/install-linux.sh") ;;
  wsl) bash <(curl -fsSL "$REPO_RAW_URL/scripts/install-wsl.sh") ;;
  mac) bash <(curl -fsSL "$REPO_RAW_URL/scripts/install-mac.sh") ;;
  *)
    echo -e "${RED}Nao foi possivel instalar automaticamente neste shell.${RESET}"
    echo "Para Windows, execute no PowerShell como Administrador:"
    echo "irm $REPO_RAW_URL/scripts/install-windows.ps1 | iex"
    ;;
esac
