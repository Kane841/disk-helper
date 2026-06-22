# Disk Helper - Windows dev environment setup
# Usage:
#   Set-ExecutionPolicy -Scope Process Bypass -Force
#   .\scripts\setup-dev-env.ps1 -UseChinaMirror
#   .\scripts\setup-dev-env.ps1 -SkipVsBuild -SkipOllama

param(
    [switch]$SkipVsBuild,
    [switch]$SkipOllama,
    [switch]$UseChinaMirror
)

$ErrorActionPreference = "Stop"

function Write-Step([string]$msg) {
    Write-Host ""
    Write-Host "==> $msg" -ForegroundColor Cyan
}

function Test-CommandExists([string]$name) {
    return $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

Write-Host "Disk Helper - Tauri dev environment setup" -ForegroundColor Green
Write-Host "Project: $PSScriptRoot\.."

Write-Step "Check Node.js"
if (-not (Test-CommandExists "node")) {
    Write-Host "Node.js not found. Install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}
Write-Host ("Node {0} / npm {1}" -f (node --version), (npm --version))

Write-Step "Check WebView2 runtime"
$webviewKey = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
if (Test-Path $webviewKey) {
    Write-Host "WebView2: installed"
} else {
    Write-Host "WebView2 may be missing. Install if tauri dev fails:" -ForegroundColor Yellow
    Write-Host "https://developer.microsoft.com/microsoft-edge/webview2/"
}

if (-not $SkipVsBuild) {
    Write-Step "Check MSVC / VS Build Tools"
    $vsWhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    $hasCpp = $false
    $installPath = $null
    if (Test-Path $vsWhere) {
        $installPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>&1 | Select-Object -First 1
        if ($installPath -and ($installPath -is [string]) -and ($installPath.Length -gt 0)) {
            $hasCpp = $true
        }
    }
    if ($hasCpp) {
        Write-Host "VS C++ Build Tools: installed at $installPath"
    } else {
        Write-Host "C++ Build Tools not found. Install VS 2022 Build Tools with C++ workload." -ForegroundColor Yellow
        Write-Host "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        if (Test-CommandExists "winget") {
            Write-Host "Trying winget install (admin recommended, large download)..."
            $override = "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
            winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override $override
        }
    }
}

Write-Step "Check Rust toolchain"
if (Test-CommandExists "rustc") {
    Write-Host (rustc --version)
    Write-Host (cargo --version)
} else {
    Write-Host "Rust not found. Installing rustup..." -ForegroundColor Yellow
    $rustupInit = Join-Path $env:TEMP "rustup-init.exe"

    if ($UseChinaMirror) {
        $env:RUSTUP_DIST_SERVER = "https://rsproxy.cn"
        $env:RUSTUP_UPDATE_ROOT = "https://rsproxy.cn/rustup"
        Write-Host "Using Rust mirror: rsproxy.cn"
        $rustupUrl = "https://rsproxy.cn/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    } else {
        $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    }

    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupInit -UseBasicParsing
    } catch {
        Write-Host ("Failed to download rustup-init: {0}" -f $_.Exception.Message) -ForegroundColor Red
        Write-Host "Manual install: https://www.rust-lang.org/tools/install"
        Write-Host "Or retry: .\scripts\setup-dev-env.ps1 -UseChinaMirror"
        exit 1
    }

    & $rustupInit -y --default-toolchain stable --profile default
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $env:Path = $cargoBin + ";" + $env:Path
        Write-Host "Added to PATH for this session: $cargoBin"
        Write-Host "Restart PowerShell before running npm run tauri dev" -ForegroundColor Yellow
    }
}

Write-Step "Install npm dependencies"
$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $projectRoot
try {
    if ($UseChinaMirror) {
        npm install --registry=https://registry.npmmirror.com
    } else {
        npm install
    }
} finally {
    Pop-Location
}

if ($UseChinaMirror) {
    Write-Step "Configure Cargo mirror (rsproxy)"
    $cargoDir = Join-Path $projectRoot ".cargo"
    if (-not (Test-Path $cargoDir)) {
        New-Item -ItemType Directory -Path $cargoDir | Out-Null
    }
    $cargoConfig = Join-Path $cargoDir "config.toml"
    if (-not (Test-Path $cargoConfig)) {
        @"
[source.crates-io]
replace-with = "rsproxy-sparse"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
"@ | Set-Content -Path $cargoConfig -Encoding UTF8
        Write-Host "Created $cargoConfig"
    } else {
        Write-Host "Cargo config already exists: $cargoConfig"
    }
}

if (-not $SkipOllama) {
    Write-Step "Check Ollama (optional until M5)"
    if (Test-CommandExists "ollama") {
        Write-Host (ollama --version)
        ollama pull deepseek-r1:1.5b
    } else {
        Write-Host "Ollama not installed: https://ollama.com/download" -ForegroundColor Yellow
        Write-Host "After install: ollama pull deepseek-r1:1.5b"
    }
}

Write-Step "Environment summary"
@(
    @{ Name = "node"; Ok = (Test-CommandExists "node") },
    @{ Name = "npm"; Ok = (Test-CommandExists "npm") },
    @{ Name = "rustc"; Ok = (Test-CommandExists "rustc") },
    @{ Name = "cargo"; Ok = (Test-CommandExists "cargo") }
) | ForEach-Object {
    $label = if ($_.Ok) { "OK" } else { "MISSING" }
    $color = if ($_.Ok) { "Green" } else { "Red" }
    Write-Host ("  [{0}] {1}" -f $label, $_.Name) -ForegroundColor $color
}

Write-Step "Next steps"
Write-Host "1. Restart terminal if Rust was just installed."
Write-Host "2. Frontend only (M1 prototype): cd disk-helper && npm run dev"
Write-Host "3. Tauri desktop: cd disk-helper && npm run tauri dev"
Write-Host "4. First tauri dev may take 10-30 minutes to compile Rust deps."
