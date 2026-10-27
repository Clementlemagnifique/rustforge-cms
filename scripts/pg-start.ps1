# Démarre l'instance PostgreSQL portable (.pgsql/) si elle n'est pas déjà lancée.
param([int]$Port = 5432)

$root    = Split-Path -Parent $PSScriptRoot
$binDir  = Join-Path $root ".pgsql\pgsql\bin"
$dataDir = Join-Path $root ".pgsql\data"
$logFile = Join-Path $root ".pgsql\postgres.log"

if (-not (Test-Path "$binDir\pg_ctl.exe")) {
    Write-Error "PostgreSQL portable absent. Lancez d'abord scripts\pg-setup.ps1"
    exit 1
}

& "$binDir\pg_ctl.exe" -D $dataDir status *> $null
if ($LASTEXITCODE -eq 0) {
    Write-Host "🟢 PostgreSQL est déjà démarré."
    exit 0
}

Write-Host "▶️  Démarrage de PostgreSQL sur le port $Port..."
# ShellExecute : le processus postgres n'hérite pas des flux du terminal appelant
# (sinon un pipe reste ouvert et le script ne rend jamais la main).
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "$binDir\pg_ctl.exe"
$psi.Arguments = "-D `"$dataDir`" -l `"$logFile`" -o `"-p $Port`" -w start"
$psi.UseShellExecute = $true
$psi.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
$proc = [System.Diagnostics.Process]::Start($psi)
$proc.WaitForExit()

if ($proc.ExitCode -ne 0) {
    Write-Error "Échec du démarrage (voir $logFile)"
    exit 1
}
Write-Host "🟢 PostgreSQL démarré (journal : $logFile)"
