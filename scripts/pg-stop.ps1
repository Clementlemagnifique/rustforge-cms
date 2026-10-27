# Arrête proprement l'instance PostgreSQL portable (.pgsql/).
$root    = Split-Path -Parent $PSScriptRoot
$binDir  = Join-Path $root ".pgsql\pgsql\bin"
$dataDir = Join-Path $root ".pgsql\data"

& "$binDir\pg_ctl.exe" -D $dataDir -m fast stop | Out-Host
