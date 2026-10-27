<#
.SYNOPSIS
  Installe une instance PostgreSQL portable dans .pgsql/ (sans droits admin).

.DESCRIPTION
  - Télécharge les binaires officiels EDB (Windows x64)
  - Initialise un cluster dans .pgsql/data (utilisateur postgres / mot de passe postgres)
  - Démarre le serveur et crée la base "trustpress"
  Idempotent : chaque étape est sautée si elle a déjà été faite.
#>
param(
    [string]$Version = "18.4",
    [int]$Port = 5432,
    [string]$Password = "postgres",
    [string]$Database = "trustpress"
)

$ErrorActionPreference = "Stop"
$root    = Split-Path -Parent $PSScriptRoot
$pgRoot  = Join-Path $root ".pgsql"
$binDir  = Join-Path $pgRoot "pgsql\bin"
$dataDir = Join-Path $pgRoot "data"
$zip     = Join-Path $pgRoot "postgresql-$Version.zip"

New-Item -ItemType Directory -Force $pgRoot | Out-Null

# 1. Binaires
if (-not (Test-Path (Join-Path $binDir "pg_ctl.exe"))) {
    if (-not (Test-Path $zip)) {
        $url = "https://get.enterprisedb.com/postgresql/postgresql-$Version-1-windows-x64-binaries.zip"
        Write-Host "⬇️  Téléchargement de PostgreSQL $Version (~320 Mo)..."
        $ProgressPreference = "SilentlyContinue"
        Invoke-WebRequest $url -OutFile $zip -UseBasicParsing
    }
    Write-Host "📦 Extraction..."
    Expand-Archive $zip -DestinationPath $pgRoot -Force
    Remove-Item $zip
}

# 2. Cluster
if (-not (Test-Path (Join-Path $dataDir "PG_VERSION"))) {
    Write-Host "🗄️  Initialisation du cluster..."
    $pwFile = Join-Path $pgRoot "pwfile.tmp"
    Set-Content $pwFile $Password -NoNewline -Encoding ascii
    & "$binDir\initdb.exe" -D $dataDir -U postgres --pwfile=$pwFile -E UTF8 --locale=C -A scram-sha-256 | Out-Host
    Remove-Item $pwFile
    if ($LASTEXITCODE -ne 0) { throw "initdb a échoué" }
}

# 3. Démarrage
& "$PSScriptRoot\pg-start.ps1" -Port $Port

# 4. Base de données
$env:PGPASSWORD = $Password
$exists = & "$binDir\psql.exe" -h localhost -p $Port -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$Database'"
if ($exists -ne "1") {
    Write-Host "✨ Création de la base « $Database »..."
    & "$binDir\createdb.exe" -h localhost -p $Port -U postgres $Database
}
Write-Host "✅ PostgreSQL prêt : postgres://postgres:$Password@localhost:$Port/$Database"
