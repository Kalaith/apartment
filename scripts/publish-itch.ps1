# Package and publish Apartment's WebGL build as a self-contained itch.io upload.
# The normal RustGames package expects shared assets one directory above the game;
# itch.io serves each channel from an isolated package root.

param(
    [string]$Target = "kalaith/apartment:html5",
    [string]$ButlerPath = "",
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path $PSScriptRoot -Parent
$workspaceRoot = Split-Path $projectRoot -Parent
$sourceDir = Join-Path $projectRoot "dist\webgl"
$packageDir = Join-Path $projectRoot "dist\itch-webgl"
$sharedWebDir = Join-Path $workspaceRoot "rust_management\web"
$sharedRuntimeDir = Join-Path $workspaceRoot "Release\shared-assets\runtime"

if ([string]::IsNullOrWhiteSpace($ButlerPath)) {
    $ButlerPath = Join-Path $workspaceRoot "rust_management\itch-butler\butler.exe"
}

if (-not (Test-Path $sourceDir -PathType Container)) {
    throw "WebGL package not found: $sourceDir. Run .\publish.ps1 first."
}

if (-not (Test-Path $ButlerPath -PathType Leaf)) {
    throw "Butler executable not found: $ButlerPath"
}

$requiredSourceFiles = @(
    "index.html",
    "apartment.wasm",
    "assets.zip"
)
foreach ($fileName in $requiredSourceFiles) {
    $sourcePath = Join-Path $sourceDir $fileName
    if (-not (Test-Path $sourcePath -PathType Leaf)) {
        throw "Required WebGL file not found: $sourcePath"
    }
}

$requiredSharedFiles = @(
    "shared.css"
)
foreach ($fileName in $requiredSharedFiles) {
    $sourcePath = Join-Path $sharedWebDir $fileName
    if (-not (Test-Path $sourcePath -PathType Leaf)) {
        throw "Required shared web file not found: $sourcePath"
    }
}

$runtimeFiles = @(
    "mq_js_bundle.js",
    "sapp_jsutils.js",
    "quad-net.js",
    "storage.js",
    "clipboard.js"
)
foreach ($fileName in $runtimeFiles) {
    $sourcePath = Join-Path $sharedRuntimeDir $fileName
    if (-not (Test-Path $sourcePath -PathType Leaf)) {
        throw "Required shared runtime file not found: $sourcePath"
    }
}

if (Test-Path $packageDir) {
    Remove-Item -LiteralPath $packageDir -Recurse -Force
}
New-Item -ItemType Directory -Path $packageDir -Force | Out-Null

Copy-Item -Path (Join-Path $sourceDir "*") -Destination $packageDir -Recurse -Force
Copy-Item -Path (Join-Path $sharedWebDir "shared.css") -Destination $packageDir -Force

$runtimeDestination = Join-Path $packageDir "shared-assets\runtime"
New-Item -ItemType Directory -Path $runtimeDestination -Force | Out-Null
foreach ($fileName in $runtimeFiles) {
    Copy-Item -Path (Join-Path $sharedRuntimeDir $fileName) -Destination $runtimeDestination -Force
}

$indexPath = Join-Path $packageDir "index.html"
$index = Get-Content -LiteralPath $indexPath -Raw -Encoding UTF8
$pathReplacements = @{
    'href="../shared.css"' = 'href="shared.css"'
    'src="../shared-assets/runtime/mq_js_bundle.js' = 'src="shared-assets/runtime/mq_js_bundle.js'
    'src="../shared-assets/runtime/sapp_jsutils.js' = 'src="shared-assets/runtime/sapp_jsutils.js'
    'src="../shared-assets/runtime/quad-net.js' = 'src="shared-assets/runtime/quad-net.js'
    'src="../shared-assets/runtime/storage.js' = 'src="shared-assets/runtime/storage.js'
    'src="../shared-assets/runtime/clipboard.js' = 'src="shared-assets/runtime/clipboard.js'
}
foreach ($replacement in $pathReplacements.GetEnumerator()) {
    $index = $index.Replace($replacement.Key, $replacement.Value)
}

# The catalog shell includes these support widgets for WebHatchery-hosted games.
# An itch upload is a standalone game page, so keep its presentation focused on
# the game and avoid carrying either widget into the embedded build.
$index = $index.Replace('    <link rel="stylesheet" href="../bug-report.css">', '')
$index = [regex]::Replace(
    $index,
    '(?s)\s*<!-- Player bug reporting.*?<script src="../bug-report.js"></script>',
    ''
)
$index = [regex]::Replace(
    $index,
    "(?s)\s*<!-- Ko-fi support widget.*?kofiWidgetOverlay\.draw.*?</script>",
    ''
)
Set-Content -LiteralPath $indexPath -Value $index -Encoding UTF8

Write-Host "Prepared self-contained itch package: $packageDir" -ForegroundColor Green
Get-ChildItem $packageDir -Recurse -File | Measure-Object -Property Length -Sum | ForEach-Object {
    Write-Host "Files: $($_.Count), bytes: $($_.Sum)" -ForegroundColor Gray
}

$butlerArgs = @("push", "--assume-yes", "--auto-wrap")
if ($DryRun) {
    $butlerArgs += "--dry-run"
}
$butlerArgs += @($packageDir, $Target)

& $ButlerPath @butlerArgs
if ($LASTEXITCODE -ne 0) {
    throw "Butler upload failed with exit code $LASTEXITCODE."
}
