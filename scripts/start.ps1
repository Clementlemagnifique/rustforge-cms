$ErrorActionPreference='Stop'
Set-Location -LiteralPath $PSScriptRoot
$binary=Join-Path $PSScriptRoot 'rustforge.exe'
if (-not (Test-Path -LiteralPath $binary)) { throw 'Binaire absent. Utilisez le paquet RustForge CMS complet.' }
if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'config.toml'))) {
    Write-Host 'Bienvenue dans RustForge CMS. Une base PostgreSQL vide suffit.'
    Write-Host 'Adresse : postgres://utilisateur:mot-de-passe@localhost:5432/ma_base'
    $previousDatabaseUrl=$env:DATABASE_URL
    try {
        if (-not $env:DATABASE_URL) {
            $connectionSecret=Read-Host 'Adresse de votre base (saisie masquée)' -AsSecureString
            $env:DATABASE_URL=[System.Net.NetworkCredential]::new('', $connectionSecret).Password
        }
        & $binary init
        if ($LASTEXITCODE -ne 0) { throw 'Initialisation interrompue. Vérifiez les informations indiquées.' }
    } finally { $env:DATABASE_URL=$previousDatabaseUrl }
}
& $binary serve
if ($LASTEXITCODE -ne 0) { throw 'RustForge CMS a rencontré une erreur au démarrage.' }
