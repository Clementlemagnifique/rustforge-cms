#Requires -Version 7.4
param([Parameter(Mandatory)][ValidateSet('Start','Stop','Status')][string]$Action)
$ErrorActionPreference='Stop'
$project=Split-Path $PSScriptRoot -Parent
$recipe=Join-Path $project 'scratch/wasm-auth-recipe'
$stateFile=Join-Path $recipe 'runtime.json'
if (!(Test-Path -LiteralPath $stateFile)) {throw 'Fixture absente : créer une base de recette dédiée avant ce script.'}
$state=Get-Content -Raw -LiteralPath $stateFile | ConvertFrom-Json
if ($state.database -notmatch '^rf_wasm_browser_[a-f0-9]{32}$' -or $state.port -ne 55499) {throw 'Identité de fixture invalide'}
$executable=Join-Path $project 'target/debug/rustforge.exe'
$pgBin=Join-Path $project '.pgsql/pgsql/bin'
$pgData=Join-Path $project 'scratch/consolidation-db'
$server=Get-Process -Id $state.pid -ErrorAction SilentlyContinue
$owned=$false
if ($server) {
    $started=[datetime]$state.started
    $owned=([IO.Path]::GetFullPath($server.Path) -eq [IO.Path]::GetFullPath($executable) -and $server.StartTime.ToUniversalTime().Ticks -eq $started.ToUniversalTime().Ticks)
}
if ($Action -eq 'Status') { [pscustomobject]@{server_owned_running=$owned;port=55499;closed=$state.closed};return }
if ($Action -eq 'Stop') {
    if ($server -and !$owned) {throw 'Identité de processus différente : arrêt refusé'}
    if ($owned) {Stop-Process -Id $server.Id}
    if (!$state.closed) {
        & "$pgBin/pg_ctl.exe" -D $pgData -m fast stop
        if ($LASTEXITCODE -ne 0) {throw 'Arrêt du PostgreSQL de recette échoué'}
    }
    $state.closed=$true
    $state | ConvertTo-Json | Set-Content -LiteralPath $stateFile
    return
}
if ($owned -and !$state.closed) {Write-Host 'Instance de recette déjà démarrée sur 55499';return}
if ($server) {throw 'Un autre processus utilise le PID enregistré : démarrage refusé'}
& "$pgBin/pg_ctl.exe" -D $pgData -l "$recipe/postgres.log" -o '-p 55496 -h 127.0.0.1' start
if ($LASTEXITCODE -ne 0) {throw 'PostgreSQL déjà utilisé ou indisponible : aucun processus externe ne sera arrêté'}
try {
    $environment=@{
        DATABASE_URL="postgres://rustforge_test@127.0.0.1:55496/$($state.database)"
        JWT_SECRET=([guid]::NewGuid().ToString('N')+[guid]::NewGuid().ToString('N'))
        ADMIN_DIST="$project/admin/dist";THEMES_DIR="$project/themes"
        UPLOADS_DIR="$recipe/uploads";PLUGINS_DIR="$recipe/plugins";PROTECTED_DOWNLOADS_DIR="$recipe/protected_downloads"
    }
    $server=Start-Process -FilePath $executable -ArgumentList @('serve','--port','55499','--host','127.0.0.1') -WorkingDirectory $recipe -Environment $environment -WindowStyle Hidden -RedirectStandardOutput "$recipe/server.log" -RedirectStandardError "$recipe/server-error.log" -PassThru
    $state.pid=$server.Id;$state.started=$server.StartTime.ToString('o');$state.closed=$false
    $state | ConvertTo-Json | Set-Content -LiteralPath $stateFile
    Write-Host 'Instance de recette démarrée : http://127.0.0.1:55499/admin/'
} catch {
    & "$pgBin/pg_ctl.exe" -D $pgData -m fast stop | Out-Null
    throw
}
