param([switch]$Release, [switch]$PublishPreview)
$ErrorActionPreference='Stop'
$project=Split-Path $PSScriptRoot -Parent
$trunk=Join-Path $project '.tools/bin/trunk.exe'
if (!(Test-Path -LiteralPath $trunk)) { throw 'Trunk local absent : installer dans .tools avant le build.' }
$previousPath=$env:PATH
$env:PATH=(Join-Path $project '.tools/bin')+';'+(Join-Path $env:USERPROFILE '.cargo/bin')+';'+$env:PATH
Push-Location (Join-Path $project 'crates/rustforge-studio')
try {
    $argsList=@('build','--offline')
    if ($Release) { $argsList+='--release' }
    & $trunk @argsList
    if ($LASTEXITCODE -ne 0) { throw 'Build Trunk échoué' }
    # Trunk emits an inline bootstrap, which the CMS deliberately rejects.
    # Keep that bootstrap as a same-origin module instead of allowing inline JS.
    $indexPath=Join-Path $PWD 'dist/index.html'
    $html=[IO.File]::ReadAllText($indexPath)
    $pattern='(?s)<script type="module">(.*?)</script>'
    $bootstrapMatches=[regex]::Matches($html,$pattern)
    if ($bootstrapMatches.Count -ne 1) { throw 'Bootstrap Trunk inattendu : publication refusée.' }
    $bootstrap=$bootstrapMatches[0].Groups[1].Value
    $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($bootstrap))).ToLowerInvariant().Substring(0,16)
    $bootstrapName="bootstrap-$hash.js"
    [IO.File]::WriteAllText((Join-Path $PWD "dist/$bootstrapName"),$bootstrap)
    $html=[regex]::Replace($html,$pattern,"<script type=`"module`" src=`"/admin/rust/$bootstrapName`"></script>")
    [IO.File]::WriteAllText($indexPath,$html)
    if ($PublishPreview) {
        $destination=Join-Path $project 'admin/public/rust'
        New-Item -ItemType Directory -Force -Path $destination | Out-Null
        Copy-Item -Path './dist/*' -Destination $destination -Recurse -Force
    }
} finally { Pop-Location; $env:PATH=$previousPath }
