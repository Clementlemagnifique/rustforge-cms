# ==============================================================================
# RustForge CMS — Script de Packaging Serveur Portable (Drop & Run)
# Alter ego de l'archive telechargeable wordpress.zip :
# Assemble un serveur complet pret a l'emploi (Binaire Rust + Studio Web + Themes + Plugins)
# ==============================================================================

param (
    [string]$Version = "0.1.0",
    [string]$OutputDir = "releases"
)

$ErrorActionPreference = "Stop"
Write-Host "[INFO] Preparation du package de distribution RustForge v$Version..." -ForegroundColor Cyan

$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.FullName
$StageDir = Join-Path $WorkspaceRoot "target\package-staging\rustforge-server"
$ReleaseZip = Join-Path $WorkspaceRoot "$OutputDir\rustforge-server-v$Version-windows-x64.zip"

# 1. Nettoyage du staging
if (Test-Path $StageDir) {
    Remove-Item -Recurse -Force $StageDir
}
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $WorkspaceRoot $OutputDir) | Out-Null

# 2. Compilation du Studio Web d'administration
Write-Host "[BUILD] Compilation du Studio Web (admin/dist)..." -ForegroundColor Yellow
Push-Location (Join-Path $WorkspaceRoot "admin")
npm run build
Pop-Location

# 3. Compilation du binaire RustForge en mode Release
Write-Host "[BUILD] Compilation du binaire RustForge (cargo build --release)..." -ForegroundColor Yellow
$CargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $CargoPath)) {
    $CargoPath = "cargo"
}
& $CargoPath build --release --bin rustforge

# 4. Copie des composants vitaux
Write-Host "[STAGE] Assemblage de l'arborescence portable..." -ForegroundColor Yellow

# Binaire
Copy-Item (Join-Path $WorkspaceRoot "target\release\rustforge.exe") -Destination (Join-Path $StageDir "rustforge.exe")

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

# README de demarrage rapide serveur
$ReadmeContent = @"
# RustForge CMS Server v$Version
L'alter ego haute performance de WordPress, compile en Rust souverain.

## Demarrage en 1 minute :
1. Copiez 'config.toml.example' vers 'config.toml' et renseignez votre connexion PostgreSQL.
2. Lancez le serveur :
   .\rustforge.exe serve --port 8080
3. Ouvrez votre navigateur sur :
   http://localhost:8080/admin
4. L'Installateur Web demarre automatiquement :
   - Creez votre compte administrateur en 1 clic
   - OU restaurez une capsule de site (.rfcapsule) existante !
"@
Set-Content -Path (Join-Path $StageDir "README.txt") -Value $ReadmeContent -Encoding UTF8

# 5. Creation de l'archive ZIP
Write-Host "[ZIP] Compression dans $ReleaseZip..." -ForegroundColor Yellow
if (Test-Path $ReleaseZip) {
    Remove-Item -Force $ReleaseZip
}
Compress-Archive -Path "$StageDir\*" -DestinationPath $ReleaseZip -CompressionLevel Optimal

$ZipSizeMb = ((Get-Item $ReleaseZip).Length / 1MB).ToString("F2")
Write-Host "[SUCCESS] Package serveur cree avec succes ! ($ZipSizeMb Mo)" -ForegroundColor Green
Write-Host "          Archive : $ReleaseZip" -ForegroundColor Green
