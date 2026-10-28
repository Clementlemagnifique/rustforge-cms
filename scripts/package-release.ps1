# ==============================================================================
# RustForge CMS — Script de Packaging Serveur Portable (Drop & Run)
# Paquet serveur RustForge CMS :
# Assemble un serveur complet pret a l'emploi (Binaire Rust + Studio Web + Themes + Plugins)
# ==============================================================================

param (
    [string]$Version = "1.0.0-alpha.1",
    [string]$OutputDir = "releases"
)

$ErrorActionPreference = "Stop"
Write-Host "[INFO] Preparation du package de distribution RustForge v$Version..." -ForegroundColor Cyan

$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.FullName
$StageDir = Join-Path $WorkspaceRoot "target\package-staging\rustforge-server"
$ReleaseZip = Join-Path $WorkspaceRoot "$OutputDir\rustforge-server-v$Version-windows-x64.zip"
$ReleaseZip = [IO.Path]::GetFullPath($ReleaseZip)
if (-not $ReleaseZip.StartsWith($WorkspaceRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Archive hors du projet' }
$StageDir = [IO.Path]::GetFullPath($StageDir)
$AllowedStage = [IO.Path]::GetFullPath((Join-Path $WorkspaceRoot 'target\package-staging'))
if (-not $StageDir.StartsWith($AllowedStage + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Staging hors du projet' }
if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?$') { throw 'Version invalide' }
if (Test-Path -LiteralPath $ReleaseZip) { throw 'Archive existante : choisir une nouvelle version' }

# 1. Nettoyage du staging
if (Test-Path $StageDir) {
    Remove-Item -LiteralPath $StageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $WorkspaceRoot $OutputDir) | Out-Null

# 2. Compilation du Studio Web d'administration
Write-Host "[BUILD] Compilation du Studio Web (admin/dist)..." -ForegroundColor Yellow
Push-Location (Join-Path $WorkspaceRoot "admin")
try {
    npm run build
    if ($LASTEXITCODE -ne 0) { throw 'Compilation du Studio échouée' }
} finally { Pop-Location }

# 3. Compilation du binaire RustForge en mode Release
Write-Host "[BUILD] Compilation du binaire RustForge (cargo build --release)..." -ForegroundColor Yellow
$CargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $CargoPath)) {
    $CargoPath = "cargo"
}
Push-Location $WorkspaceRoot
try {
    & $CargoPath test --locked --workspace
    if ($LASTEXITCODE -ne 0) { throw 'Tests Rust échoués' }
    $previousTargetFlags=$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS
    try {
        $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS='-C target-feature=+crt-static'
        & $CargoPath build --locked --release --target x86_64-pc-windows-msvc --bin rustforge
        if ($LASTEXITCODE -ne 0) { throw 'Compilation Rust échouée' }
    } finally { $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=$previousTargetFlags }
} finally { Pop-Location }

# 4. Copie des composants vitaux
Write-Host "[STAGE] Assemblage de l'arborescence portable..." -ForegroundColor Yellow

# Binaire
Copy-Item (Join-Path $WorkspaceRoot "target\x86_64-pc-windows-msvc\release\rustforge.exe") -Destination (Join-Path $StageDir "rustforge.exe")

# Configuration de base
Copy-Item (Join-Path $WorkspaceRoot "config.toml.example") -Destination (Join-Path $StageDir "config.toml.example")

# Studio Web (admin/dist)
$AdminDistDest = Join-Path $StageDir "admin\dist"
New-Item -ItemType Directory -Force -Path (Split-Path $AdminDistDest) | Out-Null
Copy-Item -Recurse (Join-Path $WorkspaceRoot "admin\dist") -Destination $AdminDistDest

# Theme par defaut
$ThemeDest = Join-Path $StageDir "themes"
New-Item -ItemType Directory -Force -Path $ThemeDest | Out-Null
if (Test-Path (Join-Path $WorkspaceRoot "themes\default")) {
    Copy-Item -Recurse (Join-Path $WorkspaceRoot "themes\default") -Destination (Join-Path $ThemeDest "default")
}

# Dossiers d'extension et de medias
New-Item -ItemType Directory -Force -Path (Join-Path $StageDir "plugins") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $StageDir "uploads") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $StageDir "protected_downloads") | Out-Null
Copy-Item -LiteralPath (Join-Path $WorkspaceRoot 'LICENSE-MIT') -Destination $StageDir
Copy-Item -LiteralPath (Join-Path $WorkspaceRoot 'docs\DEMARRAGE_ET_REPRISE.md') -Destination $StageDir
Copy-Item -LiteralPath (Join-Path $WorkspaceRoot 'docs\V1_TESTABLE.md') -Destination $StageDir
foreach ($launcher in @('start.cmd','start.ps1')) { Copy-Item -LiteralPath (Join-Path $WorkspaceRoot "scripts\$launcher") -Destination $StageDir }
New-Item -ItemType Directory -Force -Path (Join-Path $StageDir 'exemples') | Out-Null
Copy-Item -LiteralPath (Join-Path $WorkspaceRoot 'tests\fixtures\wordpress-editorial.xml') -Destination (Join-Path $StageDir 'exemples')

# README de demarrage rapide serveur
$ReadmeContent = @"
# RustForge CMS Server v$Version
L'alter ego haute performance de WordPress, compile en Rust souverain.

## Demarrage en 1 minute :
1. Créez une base PostgreSQL vide (PostgreSQL est un prérequis externe).
2. Lancez le serveur :
   .\start.cmd
3. Ouvrez votre navigateur sur :
   http://localhost:8080/admin
4. L'Installateur Web demarre automatiquement :
   - Renseignez l’adresse de la base dans le lanceur. Les secrets et les tables sont préparés automatiquement.
   - Ouvrez le lien d’installation affiché dans le terminal puis créez votre administrateur. Voir V1_TESTABLE.md.
   - Capsules natives v2 : restauration CLI sur base vierge, voir DEMARRAGE_ET_REPRISE.md.
"@
Set-Content -Path (Join-Path $StageDir "README.txt") -Value $ReadmeContent -Encoding UTF8

# 5. Creation de l'archive ZIP
Write-Host "[ZIP] Compression dans $ReleaseZip..." -ForegroundColor Yellow
if (Test-Path $ReleaseZip) {
    Remove-Item -Force $ReleaseZip
}
Compress-Archive -Path "$StageDir\*" -DestinationPath $ReleaseZip -CompressionLevel Optimal
(Get-FileHash -LiteralPath $ReleaseZip -Algorithm SHA256).Hash | Set-Content -LiteralPath "$ReleaseZip.sha256"

$ZipSizeMb = ((Get-Item $ReleaseZip).Length / 1MB).ToString("F2")
Write-Host "[SUCCESS] Package serveur cree avec succes ! ($ZipSizeMb Mo)" -ForegroundColor Green
Write-Host "          Archive : $ReleaseZip" -ForegroundColor Green
