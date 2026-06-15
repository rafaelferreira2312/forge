# FORGE - Instalador Windows
# Executar no PowerShell como Administrador:
# irm https://raw.githubusercontent.com/rafaelferreira2312/forge/main/scripts/install-windows.ps1 | iex

$ErrorActionPreference = "Stop"
$ForgeRepoUrl = if ($env:FORGE_REPO_URL) { $env:FORGE_REPO_URL } else { "https://github.com/rafaelferreira2312/forge.git" }

Write-Host ""
Write-Host "  FORGE - Instalador Windows" -ForegroundColor Yellow
Write-Host "  Voce pensa. Forge traduz. A IA constroi." -ForegroundColor Yellow
Write-Host ""

if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "Execute como Administrador!" -ForegroundColor Red
    exit 1
}

if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Instalando Chocolatey..." -ForegroundColor Cyan
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
}

Write-Host "Instalando dependencias..." -ForegroundColor Cyan
choco install -y git rust visualstudio2022buildtools

Write-Host "Instalando Ollama..." -ForegroundColor Cyan
choco install -y ollama

$RAM_GB = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB)
if ($RAM_GB -ge 16) { $Model = "llama3.1:8b" }
elseif ($RAM_GB -ge 8) { $Model = "llama3.2:3b" }
else { $Model = "phi3:mini" }

Write-Host "Baixando modelo Ollama ($Model) - $RAM_GB GB RAM..." -ForegroundColor Cyan
Start-Process ollama -ArgumentList "serve" -WindowStyle Hidden
Start-Sleep 3
ollama pull $Model

$InstallDir = "$env:USERPROFILE\.forge"
if (Test-Path "$InstallDir\.git") {
    Set-Location $InstallDir
    git pull
} else {
    if (Test-Path $InstallDir) { Remove-Item $InstallDir -Recurse -Force }
    git clone $ForgeRepoUrl $InstallDir
    Set-Location $InstallDir
}

Copy-Item .env.example .env -ErrorAction SilentlyContinue
Write-Host "Compilando Forge (3-5 min)..." -ForegroundColor Cyan
cargo build --release

$BinPath = "$env:USERPROFILE\.cargo\bin"
New-Item -ItemType Directory -Force -Path $BinPath | Out-Null
Copy-Item "$InstallDir\target\release\forge.exe" "$BinPath\forge.exe" -Force

Start-Process forge
Start-Sleep 2
Start-Process "http://localhost:3000"

Write-Host ""
Write-Host "Forge pronto em http://localhost:3000" -ForegroundColor Green
