# ComfyUI Batch Image Generation Script
# Usage:
#   .\comfyui-generate.ps1 -Prompt "single prompt" ...
#   .\comfyui-generate.ps1 -PromptJsonFile "prompts.json" ...

param(
    [Parameter(Mandatory=$false)]
    [string]$Prompt,

    [Parameter(Mandatory=$false)]
    [string]$PromptJsonFile,

    # Width = 3840
    [Parameter(Mandatory=$false)]
    [int]$Width = 1024,

    # Height = 2160
    [Parameter(Mandatory=$false)]
    [int]$Height = 1024,

    [Parameter(Mandatory=$false)]
    [string]$NegativePrompt = "bad hands, blurry, low quality",

    [Parameter(Mandatory=$false)]
    [string]$OutputPath = "generated_image.png",

    [Parameter(Mandatory=$false)]
    [string]$ComfyUIServer = "127.0.0.1:8188",

    [Parameter(Mandatory=$false)]
    [int]$Steps = 20,

    [Parameter(Mandatory=$false)]
    [double]$CFG = 8.0,

    [Parameter(Mandatory=$false)]
    [int]$Seed = -1,
    # models
    # plantMilkModelSuite_walnut.safetensors
    # juggernautXL_v8Rundiffusion.safetensors
    # ponyDiffusionV6XL_v6StartWithThisOne.safetensors
    [Parameter(Mandatory=$false)]
    [string]$Model = "plantMilkModelSuite_walnut.safetensors",

    [Parameter(Mandatory=$false)]
    [string]$Sampler = "euler",

    [Parameter(Mandatory=$false)]
    [string]$Scheduler = "normal",

    [Parameter(Mandatory=$false)]
    [switch]$UseScreenResolution = $false,

    [Parameter(Mandatory=$false)]
    [switch]$SetAsWallpaper = $false,

    [Parameter(Mandatory=$false)]
    [switch]$OpenImage = $false,

    [Parameter(Mandatory=$false)]
    [switch]$Test = $false,

    [Parameter(Mandatory=$false)]
    [switch]$LowRes = $false,

    [Parameter(Mandatory=$false)]
    [int]$DelaySeconds = 0
)

# ...existing code for helper functions (Get-ScreenResolution, Set-Wallpaper, Queue-Prompt, Wait-ForCompletion, Download-GeneratedImage, Test-ComfyUIConnection)...

function Invoke-ImageGeneration {
    param(
        [string]$Prompt,
        [int]$Width,
        [int]$Height,
        [string]$NegativePrompt,
        [string]$OutputPath,
        [string]$ComfyUIServer,
        [int]$Steps,
        [double]$CFG,
        [int]$Seed,
        [string]$Model,
        [string]$Sampler,
        [string]$Scheduler,
        [switch]$SetAsWallpaper,
        [switch]$OpenImage,
        [switch]$LowRes
    )

    # Generate random seed if not provided
    if ($Seed -eq -1) {
        $Seed = Get-Random -Minimum 1 -Maximum 2147483647
    }

    # Function to get screen resolution
    function Get-ScreenResolution {
        try {
            $monitor = Get-WmiObject -Class Win32_VideoController | Where-Object { $_.CurrentHorizontalResolution -ne $null } | Select-Object -First 1
            if ($monitor) {
                return @{ Width = $monitor.CurrentHorizontalResolution; Height = $monitor.CurrentVerticalResolution }
            }
            Add-Type -AssemblyName System.Windows.Forms
            $screen = [System.Windows.Forms.Screen]::PrimaryScreen
            return @{ Width = $screen.Bounds.Width; Height = $screen.Bounds.Height }
        } catch {
            Write-Warning "Could not detect screen resolution, using defaults"
            return @{ Width = 1920; Height = 1080 }
        }
    }


    # Apply screen resolution if requested
    if ($UseScreenResolution) {
        Write-Host "Detecting screen resolution..." -ForegroundColor Blue
        $screenRes = Get-ScreenResolution
        $Width = $screenRes.Width
        $Height = $screenRes.Height
        Write-Host "Using screen resolution: ${Width}x${Height}" -ForegroundColor Green
    }

    # Generate unique client ID
    $ClientId = [System.Guid]::NewGuid().ToString()

    Write-Host "ComfyUI Image Generator" -ForegroundColor Cyan
    Write-Host "Prompt: $Prompt" -ForegroundColor Yellow
    Write-Host "Size: ${Width}x${Height}" -ForegroundColor Green
    Write-Host "Seed: $Seed" -ForegroundColor Magenta
    Write-Host "Server: $ComfyUIServer" -ForegroundColor Blue
    if ($UseScreenResolution) { Write-Host "Screen Resolution Mode: Enabled" -ForegroundColor Cyan }
    if ($SetAsWallpaper) { Write-Host "Set as Wallpaper: Enabled" -ForegroundColor Cyan }

    # ComfyUI workflow JSON template
    $WorkflowTemplate = @{
        "3" = @{
            "class_type" = "KSampler"
            "inputs" = @{
                "cfg" = $CFG
                "denoise" = 1
                "latent_image" = @("5", 0)
                "model" = @("4", 0)
                "negative" = @("7", 0)
                "positive" = @("6", 0)
                "sampler_name" = $Sampler
                "scheduler" = $Scheduler
                "seed" = $Seed
                "steps" = $Steps
            }
        }
        "4" = @{
            "class_type" = "CheckpointLoaderSimple"
            "inputs" = @{ "ckpt_name" = $Model }
        }
        "5" = @{
            "class_type" = "EmptyLatentImage"
            "inputs" = @{ "batch_size" = 1; "height" = $Height; "width" = $Width }
        }
        "6" = @{
            "class_type" = "CLIPTextEncode"
            "inputs" = @{ "clip" = @("4", 1); "text" = $Prompt }
        }
        "7" = @{
            "class_type" = "CLIPTextEncode"
            "inputs" = @{ "clip" = @("4", 1); "text" = $NegativePrompt }
        }
        "8" = @{
            "class_type" = "VAEDecode"
            "inputs" = @{ "samples" = @("3", 0); "vae" = @("4", 2) }
        }
        "9" = @{
            "class_type" = "SaveImage"
            "inputs" = @{ "filename_prefix" = "PowerShell_Generated"; "images" = @("8", 0) }
        }
    }

    function Queue-Prompt {
        param([hashtable]$Workflow)
        try {
            $promptData = @{ "prompt" = $Workflow; "client_id" = $ClientId } | ConvertTo-Json -Depth 10
            $response = Invoke-RestMethod -Uri "http://$ComfyUIServer/prompt" -Method Post -Body $promptData -ContentType "application/json"
            return $response.prompt_id
        } catch {
            Write-Error "Failed to queue prompt: $($_.Exception.Message)"
            return $null
        }
    }

    function Wait-ForCompletion {
        param([string]$PromptId)
        Write-Host "Waiting for image generation to complete..." -ForegroundColor Yellow
        $maxAttempts = 60
        $attempts = 0
        while ($attempts -lt $maxAttempts) {
            try {
                $history = Invoke-RestMethod -Uri "http://$ComfyUIServer/history/$PromptId" -Method Get
                if ($history.$PromptId) {
                    Write-Host "Generation completed!" -ForegroundColor Green
                    return $history.$PromptId
                }
                Start-Sleep -Seconds 5
                $attempts++
                Write-Host "." -NoNewline -ForegroundColor Gray
            } catch {
                Write-Host "Checking status..." -ForegroundColor Yellow
                Start-Sleep -Seconds 5
                $attempts++
            }
        }
        Write-Error "Timeout waiting for completion"
        return $null
    }

    function Download-GeneratedImage {
        param([object]$HistoryData, [string]$OutputFile)
        try {
            $imageInfo = $null
            foreach ($nodeId in $HistoryData.outputs.PSObject.Properties.Name) {
                $nodeOutput = $HistoryData.outputs.$nodeId
                if ($nodeOutput.images) {
                    $imageInfo = $nodeOutput.images[0]
                    break
                }
            }
            if (-not $imageInfo) {
                Write-Error "No image found in generation output"
                return $false
            }
            $filename = $imageInfo.filename
            $subfolder = $imageInfo.subfolder
            $type = $imageInfo.type
            $downloadUrl = "http://$ComfyUIServer/view"
            $queryParams = @()
            $queryParams += "filename=$([System.Web.HttpUtility]::UrlEncode($filename))"
            if ($subfolder) { $queryParams += "subfolder=$([System.Web.HttpUtility]::UrlEncode($subfolder))" }
            $queryParams += "type=$([System.Web.HttpUtility]::UrlEncode($type))"
            $downloadUrl += "?" + ($queryParams -join "&")
            Write-Host "Downloading image: $filename" -ForegroundColor Blue
            Invoke-WebRequest -Uri $downloadUrl -OutFile $OutputFile
            if (Test-Path $OutputFile) {
                $fileSize = (Get-Item $OutputFile).Length
                Write-Host "Image saved: $OutputFile ($([math]::Round($fileSize/1KB, 2)) KB)" -ForegroundColor Green

                # Create a low-res copy if the LowRes switch is enabled
                if ($LowRes) {
                    try {
                        Add-Type -AssemblyName System.Drawing
                        $img = [System.Drawing.Image]::FromFile($OutputFile)
                        
                        # Create a low-res copy (e.g., 640px wide, keep aspect ratio)
                        $lowResWidth = 640
                        $scale = $lowResWidth / $img.Width
                        $lowResHeight = [int]($img.Height * $scale)
                        $lowResImg = New-Object System.Drawing.Bitmap $lowResWidth, $lowResHeight
                        $g = [System.Drawing.Graphics]::FromImage($lowResImg)
                        $g.DrawImage($img, 0, 0, $lowResWidth, $lowResHeight)
                        $lowResPath = [System.IO.Path]::ChangeExtension($OutputFile, $null) + "_lowres.png"
                        $lowResImg.Save($lowResPath, [System.Drawing.Imaging.ImageFormat]::Png)
                        $g.Dispose()
                        $img.Dispose()
                        $lowResImg.Dispose()
                        Write-Host "Low-res copy saved: $lowResPath" -ForegroundColor Cyan
                    } catch {
                        Write-Warning "Failed to create low-res copy: $($_.Exception.Message)"
                    }
                }

                return $true
            } else {
                Write-Error "Failed to save image"
                return $false
            }
        } catch {
            Write-Error "Failed to download image: $($_.Exception.Message)"
            return $false
        }
    }

    function Test-ComfyUIConnection {
        try {
            Write-Host "Testing connection to ComfyUI..." -ForegroundColor Blue
            $response = Invoke-WebRequest -Uri "http://$ComfyUIServer/" -Method Get -TimeoutSec 10
            if ($response.StatusCode -eq 200) {
                Write-Host "ComfyUI is accessible" -ForegroundColor Green
                return $true
            }
        } catch {
            Write-Error "Cannot connect to ComfyUI at $ComfyUIServer. Make sure ComfyUI is running."
            Write-Host "Try starting ComfyUI with: python main.py --listen" -ForegroundColor Yellow
            return $false
        }
        return $false
    }

    # Main execution for a single prompt
    try {
        Add-Type -AssemblyName System.Web
        if (-not (Test-ComfyUIConnection)) { return }
        Write-Host "Queueing generation request..." -ForegroundColor Blue
        $promptId = Queue-Prompt -Workflow $WorkflowTemplate
        if (-not $promptId) {
            Write-Error "Failed to queue prompt"
            return
        }
        Write-Host "Prompt ID: $promptId" -ForegroundColor Cyan
        $historyData = Wait-ForCompletion -PromptId $promptId
        if (-not $historyData) {
            Write-Error "Generation failed or timed out"
            return
        }
        $success = Download-GeneratedImage -HistoryData $historyData -OutputFile $OutputPath
        if ($success) {
            Write-Host "Image generation completed successfully!" -ForegroundColor Green
            Write-Host "Output: $OutputPath" -ForegroundColor Cyan
            if ($SetAsWallpaper) {
                Write-Host "[SetAsWallpaper requested, but Set-Wallpaper function has been removed from this script.]" -ForegroundColor Yellow
            }
            if ($env:OS -eq "Windows_NT" -and $OpenImage) {
                Write-Host "Opening image..." -ForegroundColor Blue
                Start-Process $OutputPath
            }
        } else {
            Write-Error "Failed to download generated image"
        }
    } catch {
        Write-Error "Script failed: $($_.Exception.Message)"
    }
}

# Batch mode: If PromptJsonFile is provided, process all prompts in the file
if ($PromptJsonFile) {
    if (-not (Test-Path $PromptJsonFile)) {
        Write-Error "Prompt JSON file not found: $PromptJsonFile"
        exit 1
    }
    $jsonContent = Get-Content $PromptJsonFile -Raw | ConvertFrom-Json
    # Support both array and object with image_prompts property
    # Check if it's an object with image_prompts property (not an array)
    if ($jsonContent -isnot [System.Array] -and $null -ne $jsonContent.image_prompts) {
        $promptList = $jsonContent.image_prompts
        Write-Host "Found JSON with image_prompts wrapper, $($promptList.Count) prompts detected" -ForegroundColor Blue
    } else {
        $promptList = $jsonContent
        Write-Host "Found JSON array format, $($promptList.Count) prompts detected" -ForegroundColor Blue
    }
    
    # Filter out empty objects from the prompt list
    if ($promptList -is [System.Array]) {
        $originalCount = $promptList.Count
        $promptList = $promptList | Where-Object { 
            $_ -and ($_.description -or $_.title -or $_.Prompt) 
        }
        Write-Host "After filtering: $($promptList.Count) valid prompts (removed $($originalCount - $promptList.Count) empty/invalid)" -ForegroundColor Blue
    }
    
    if (-not $promptList -or $promptList.Count -eq 0) {
        Write-Error "No valid prompts found in JSON file."
        exit 1
    }
    $index = 0
    foreach ($item in $promptList) {
        $index++
        Write-Host "`n=== Generating image $index of $($promptList.Count) ===" -ForegroundColor Cyan

        # Compose the prompt from description or title/description
        $prompt = ""
        if ($item.Prompt -and $item.Prompt.Trim() -ne "") {
            $prompt = $item.Prompt
        } elseif ($item.description -and $item.description.Trim() -ne "") {
            $prompt = $item.description
        } elseif ($item.title -and $item.title.Trim() -ne "") {
            $prompt = $item.title
        } else {
            Write-Warning "No valid prompt found for item $index, skipping..."
            continue
        }

        $width = if ($null -ne $item.Width) { $item.Width } else { $Width }
        $height = if ($null -ne $item.Height) { $item.Height } else { $Height }

        # Support both camelCase and snake_case for negative prompt
        $negativePrompt = $NegativePrompt
        if ($null -ne $item.NegativePrompt) {
            $negativePrompt = $item.NegativePrompt
        } elseif ($null -ne $item.negative_prompt) {
            $negativePrompt = $item.negative_prompt
        }

        # Generate filename based on ID, tags, or fallback to title/description
        $keyword = "image"
        $idPrefix = ""
        
        # Use ID field if available for linking
        if ($null -ne $item.id -and $item.id -ne "") {
            $idPrefix = "$($item.id)_"
        }
        
        if ($item.tags -and $item.tags.Count -gt 0) {
            $keyword = ($item.tags -join "_").ToLower() -replace '[^a-z0-9_]', ''
        } elseif ($item.title) {
            $keyword = $item.title.ToLower() -replace '[^a-z0-9_]', ''
        } elseif ($item.description) {
            $keyword = ($item.description.Split(" ")[0..1] -join "_").ToLower() -replace '[^a-z0-9_]', ''
        }

        $dateStr = Get-Date -Format 'yyyyMMdd-HHmmss'
        $sizeStr = "${width}x${height}"
        $backgroundsDir = Join-Path -Path $PSScriptRoot -ChildPath 'backgrounds'
        if (-not (Test-Path $backgroundsDir)) { New-Item -ItemType Directory -Path $backgroundsDir | Out-Null }
        $outputPath = Join-Path $backgroundsDir ("${idPrefix}${keyword}_${sizeStr}_${dateStr}.png")

        $steps = if ($null -ne $item.Steps) { $item.Steps } else { $Steps }
        $cfg = if ($null -ne $item.CFG) { $item.CFG } else { $CFG }
        $seed = if ($null -ne $item.Seed) { $item.Seed } else { $Seed }
        $model = if ($null -ne $item.Model) { $item.Model } else { $Model }
        $sampler = if ($null -ne $item.Sampler) { $item.Sampler } else { $Sampler }
        $scheduler = if ($null -ne $item.Scheduler) { $item.Scheduler } else { $Scheduler }

        if ($Test) {
            Write-Host "Test Mode - Prompt ${index}:" -ForegroundColor Yellow
            if ($null -ne $item.id) { Write-Host "  ID: $($item.id)" -ForegroundColor White }
            Write-Host "  Prompt: $($prompt.Substring(0, [Math]::Min(100, $prompt.Length)))..." -ForegroundColor Cyan
            Write-Host "  Size: ${width}x${height}" -ForegroundColor Green
            Write-Host "  Output: $outputPath" -ForegroundColor Magenta
            Write-Host "  Model: $model" -ForegroundColor Blue
            Write-Host "  Scheduler: $scheduler" -ForegroundColor Gray
        } else {
            Invoke-ImageGeneration `
                -Prompt $prompt `
                -Width $width `
                -Height $height `
                -NegativePrompt $negativePrompt `
                -OutputPath $outputPath `
                -ComfyUIServer $ComfyUIServer `
                -Steps $steps `
                -CFG $cfg `
                -Seed $seed `
                -Model $model `
                -Sampler $sampler `
                -Scheduler $scheduler `
                -SetAsWallpaper:$SetAsWallpaper `
                -OpenImage:$OpenImage `
                -LowRes:$LowRes
        }

        if ($index -lt $promptList.Count -and -not $Test) {
            Start-Sleep -Seconds $DelaySeconds
        }
    }
    Write-Host "`nBatch generation complete!" -ForegroundColor Green
    exit 0
}

# Single prompt mode (default)
if (-not $Prompt) {
    Write-Error "You must provide either -Prompt or -PromptJsonFile."
    exit 1
}

Invoke-ImageGeneration `
    -Prompt $Prompt `
    -Width $Width `
    -Height $Height `
    -NegativePrompt $NegativePrompt `
    -OutputPath $OutputPath `
    -ComfyUIServer $ComfyUIServer `
    -Steps $Steps `
    -CFG $CFG `
    -Seed $Seed `
    -Model $Model `
    -Sampler $Sampler `
    -Scheduler $Scheduler `
    -SetAsWallpaper:$SetAsWallpaper `
    -OpenImage:$OpenImage `
    -LowRes:$LowRes

Write-Host "`nDone! Thanks for using ComfyUI PowerShell Generator!" -ForegroundColor Magenta