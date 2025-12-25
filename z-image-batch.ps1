<#
.SYNOPSIS
    Batch Image Generation using Python Diffusers (Z-Image-Turbo)
    
.DESCRIPTION
    A wrapper script that calls the z_image_batch.py python script.
    Designed to replace comfyui-batch.ps1.

.EXAMPLE
    .\z-image-batch.ps1 -Prompt "A beautiful sunset"
    .\z-image-batch.ps1 -PromptJsonFile "assets/graphics_batch.json"
#>

param(
    [Parameter(Mandatory=$false)]
    [string]$Prompt,

    [Parameter(Mandatory=$false)]
    [string]$PromptJsonFile,

    [Parameter(Mandatory=$false)]
    [int]$Width = 1024,

    [Parameter(Mandatory=$false)]
    [int]$Height = 1024,

    [Parameter(Mandatory=$false)]
    [string]$NegativePrompt = "bad hands, blurry, low quality",

    [Parameter(Mandatory=$false)]
    [string]$OutputPath = "generated_image.png",

    [Parameter(Mandatory=$false)]
    [int]$Steps = 9,

    [Parameter(Mandatory=$false)]
    [double]$CFG = 0.0,

    [Parameter(Mandatory=$false)]
    [int]$Seed = -1,

    [Parameter(Mandatory=$false)]
    [string]$Model = "Tongyi-MAI/Z-Image-Turbo",

    [Parameter(Mandatory=$false)]
    [int]$DelaySeconds = 0
)

# Build the python command arguments
$pyArgs = @()

if ($Prompt) { $pyArgs += "--Prompt", "`"$Prompt`"" }
if ($PromptJsonFile) { $pyArgs += "--PromptJsonFile", "`"$PromptJsonFile`"" }
$pyArgs += "--Width", $Width
$pyArgs += "--Height", $Height
$pyArgs += "--NegativePrompt", "`"$NegativePrompt`""
$pyArgs += "--OutputPath", "`"$OutputPath`""
$pyArgs += "--Steps", $Steps
$pyArgs += "--CFG", $CFG
$pyArgs += "--Seed", $Seed
$pyArgs += "--Model", "`"$Model`""
$pyArgs += "--DelaySeconds", $DelaySeconds

Write-Host "Starting Python Generation Script..." -ForegroundColor Cyan
Write-Host "Model: $Model" -ForegroundColor Blue

# Check for virtual environment
$venvPath = Join-Path $PSScriptRoot ".venv"
$pythonExe = "python"

if (Test-Path $venvPath) {
    if ($env:OS -eq "Windows_NT") {
        $pythonExe = Join-Path $venvPath "Scripts\python.exe"
    } else {
        $pythonExe = Join-Path $venvPath "bin/python"
    }
    if (Test-Path $pythonExe) {
        Write-Host "Using virtual environment: $venvPath" -ForegroundColor Green
    } else {
        Write-Warning "Virtual environment found but python executable missing at $pythonExe. Falling back to global python."
        $pythonExe = "python"
    }
}

# Check if python is available
if ($pythonExe -eq "python" -and -not (Get-Command "python" -ErrorAction SilentlyContinue)) {
    Write-Error "Python is not found in PATH. Please install Python."
    exit 1
}

# Run the python script
$scriptPath = Join-Path $PSScriptRoot "z_image_batch.py"
& $pythonExe $scriptPath @pyArgs

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBatch generation complete!" -ForegroundColor Green
} else {
    Write-Error "Python script failed with exit code $LASTEXITCODE"
}
