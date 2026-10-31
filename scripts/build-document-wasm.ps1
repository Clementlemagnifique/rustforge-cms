param([switch]$Release)
$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
Push-Location $project
try {
    $cargo = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
    $bindgen = Join-Path $project '.tools/bin/wasm-bindgen.exe'
    if (!(Test-Path -LiteralPath $bindgen)) { throw 'Installer wasm-bindgen-cli compatible dans .tools avant ce build.' }
    $buildArgs = @('build','--locked','-p','rustforge-studio','--target','wasm32-unknown-unknown','--lib')
    $profile = 'debug'
    if ($Release) { $buildArgs += '--release'; $profile = 'release' }
    & $cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Compilation WASM échouée' }
    $targetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $project 'target' }
    $wasm = Join-Path $targetRoot "wasm32-unknown-unknown/$profile/rustforge_studio.wasm"
    & $bindgen $wasm --target web --out-dir (Join-Path $project 'admin/public/wasm') --out-name rustforge_studio
    if ($LASTEXITCODE -ne 0) { throw 'Génération du pont navigateur échouée' }
} finally { Pop-Location }
