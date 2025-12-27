# Apartment Manager - Itch.io Publish Script
# Creates distributable packages for Windows and WebGL

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false,
    [string]$OutputName = "apartment_manager"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$DistDir = Join-Path $ProjectRoot "dist"

Write-Host "=== Apartment Manager Publisher ===" -ForegroundColor Cyan
Write-Host ""

# Calculate steps based on what we're building
$buildWindows = -not $WebGLOnly
$buildWebGL = -not $WindowsOnly
$totalSteps = 2  # Clean + Summary
if ($buildWindows) { $totalSteps += 2 }  # Build + Package
if ($buildWebGL) { $totalSteps += 2 }     # Build + Package
$currentStep = 0

# Step: Clean dist folder
$currentStep++
Write-Host "[$currentStep/$totalSteps] Preparing dist folder..." -ForegroundColor Yellow
if (Test-Path $DistDir) {
    Remove-Item $DistDir -Recurse -Force
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

# === WINDOWS BUILD ===
if ($buildWindows) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building Windows release..." -ForegroundColor Yellow
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Windows build failed!"
            exit 1
        }
        Write-Host "Windows build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping Windows build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging Windows build..." -ForegroundColor Yellow
    $WindowsPackageDir = Join-Path $DistDir "windows"
    New-Item -ItemType Directory -Path $WindowsPackageDir -Force | Out-Null

    # Copy executable
    $ExePath = Join-Path $ProjectRoot "target\release\apartment.exe"
    if (-not (Test-Path $ExePath)) {
        Write-Error "Executable not found at: $ExePath"
        exit 1
    }
    Copy-Item $ExePath $WindowsPackageDir

    # Copy assets folder
    $AssetsPath = Join-Path $ProjectRoot "assets"
    Copy-Item $AssetsPath -Destination $WindowsPackageDir -Recurse

    # Create Windows zip
    $WindowsZipPath = Join-Path $DistDir "${OutputName}_windows.zip"
    Compress-Archive -Path "$WindowsPackageDir\*" -DestinationPath $WindowsZipPath -CompressionLevel Optimal
    Write-Host "Windows package created!" -ForegroundColor Green
}

# === WEBGL BUILD ===
if ($buildWebGL) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building WebGL release..." -ForegroundColor Yellow
        
        # Check if wasm32 target is installed
        $targets = rustup target list --installed
        if ($targets -notcontains "wasm32-unknown-unknown") {
            Write-Host "Installing wasm32-unknown-unknown target..." -ForegroundColor Yellow
            rustup target add wasm32-unknown-unknown
        }

        cargo build --release --target wasm32-unknown-unknown
        if ($LASTEXITCODE -ne 0) {
            Write-Error "WebGL build failed!"
            exit 1
        }
        Write-Host "WebGL build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping WebGL build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging WebGL build..." -ForegroundColor Yellow
    $WebGLPackageDir = Join-Path $DistDir "webgl"
    New-Item -ItemType Directory -Path $WebGLPackageDir -Force | Out-Null

    # Copy WASM file
    $WasmPath = Join-Path $ProjectRoot "target\wasm32-unknown-unknown\release\apartment.wasm"
    if (-not (Test-Path $WasmPath)) {
        Write-Error "WASM file not found at: $WasmPath"
        exit 1
    }
    Copy-Item $WasmPath $WebGLPackageDir

    # Copy assets folder
    $AssetsPath = Join-Path $ProjectRoot "assets"
    Copy-Item $AssetsPath -Destination $WebGLPackageDir -Recurse

    # Create HTML wrapper
    $HtmlContent = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Apartment Manager</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background: #1a1a2e;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }
        canvas {
            display: block;
            max-width: 100%;
            max-height: 100vh;
        }
    </style>
</head>
<body>
    <canvas id="glcanvas" tabindex="1"></canvas>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>load("apartment.wasm");</script>
</body>
</html>
"@
    $HtmlPath = Join-Path $WebGLPackageDir "index.html"
    Set-Content -Path $HtmlPath -Value $HtmlContent

    # Create WebGL zip
    $WebGLZipPath = Join-Path $DistDir "${OutputName}_webgl.zip"
    Compress-Archive -Path "$WebGLPackageDir\*" -DestinationPath $WebGLZipPath -CompressionLevel Optimal
    Write-Host "WebGL package created!" -ForegroundColor Green
}

# Summary
$currentStep++
Write-Host ""
Write-Host "=== Package Complete ===" -ForegroundColor Cyan

if ($buildWindows) {
    $WindowsZipPath = Join-Path $DistDir "${OutputName}_windows.zip"
    $WinSize = [math]::Round((Get-Item $WindowsZipPath).Length / 1MB, 2)
    Write-Host "Windows: $WindowsZipPath (${WinSize} MB)" -ForegroundColor Green
}

if ($buildWebGL) {
    $WebGLZipPath = Join-Path $DistDir "${OutputName}_webgl.zip"
    $WebSize = [math]::Round((Get-Item $WebGLZipPath).Length / 1MB, 2)
    Write-Host "WebGL:   $WebGLZipPath (${WebSize} MB)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Next steps for itch.io:" -ForegroundColor Yellow
Write-Host "  1. Go to https://itch.io/dashboard" -ForegroundColor White
Write-Host "  2. Create/edit your project" -ForegroundColor White
if ($buildWindows) {
    Write-Host "  3. Upload Windows zip, mark as 'Windows'" -ForegroundColor White
}
if ($buildWebGL) {
    Write-Host "  4. Upload WebGL zip, mark as 'Play in browser'" -ForegroundColor White
}
Write-Host ""

# Open dist folder
explorer $DistDir
