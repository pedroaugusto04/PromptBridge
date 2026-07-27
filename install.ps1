# PromptBridge Installation Script for Windows
# This script downloads the latest binary and sets up configuration

Write-Host "🚀 Installing PromptBridge..." -ForegroundColor Green

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$binary = "promptbridge-${arch}-pc-windows-msvc.zip"

# Create temporary directory
$tmpDir = Join-Path $env:TEMP "promptbridge-install"
New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

try {
    # Download latest release
    Write-Host "📥 Downloading PromptBridge $binary..." -ForegroundColor Cyan
    $latestUrl = "https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/${binary}"
    $zipPath = Join-Path $tmpDir "promptbridge.zip"
    
    try {
        Invoke-WebRequest -Uri $latestUrl -OutFile $zipPath -UseBasicParsing
    } catch {
        Write-Host "❌ Failed to download binary. Please check your internet connection." -ForegroundColor Red
        exit 1
    }

    # Extract binary
    Write-Host "📦 Extracting binary..." -ForegroundColor Cyan
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

    # Determine installation directory
    $installDir = Join-Path $env:USERPROFILE ".cargo\bin"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    # Install binary
    Write-Host "🔧 Installing to $installDir..." -ForegroundColor Cyan
    $binaryPath = Join-Path $tmpDir "promptbridge.exe"
    if (Test-Path $binaryPath) {
        Copy-Item -Path $binaryPath -Destination $installDir -Force
    } else {
        # Try without .exe extension
        $binaryPath = Join-Path $tmpDir "promptbridge"
        Copy-Item -Path $binaryPath -Destination (Join-Path $installDir "promptbridge.exe") -Force
    }

    # Create configuration directory
    Write-Host "⚙️  Setting up configuration..." -ForegroundColor Cyan
    $configDir = Join-Path $env:APPDATA "promptbridge"
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null

    # Download example config if config doesn't exist
    $configPath = Join-Path $configDir "promptbridge.toml"
    if (-not (Test-Path $configPath)) {
        Write-Host "📄 Creating default configuration..." -ForegroundColor Cyan
        $exampleUrl = "https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/promptbridge.example.toml"
        try {
            Invoke-WebRequest -Uri $exampleUrl -OutFile $configPath -UseBasicParsing
        } catch {
            Write-Host "⚠️  Failed to download example config. You'll need to create it manually." -ForegroundColor Yellow
        }
    } else {
        Write-Host "ℹ️  Configuration already exists, skipping..." -ForegroundColor Yellow
    }

    # Add to PATH if needed
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir*") {
        Write-Host ""
        Write-Host "⚠️  $installDir is not in your PATH." -ForegroundColor Yellow
        Write-Host "   Adding to PATH..." -ForegroundColor Cyan
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
        Write-Host "   PATH updated. You may need to restart your terminal." -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "✅ PromptBridge installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📝 Next steps:" -ForegroundColor Cyan
    Write-Host "   1. Restart your terminal or refresh your PATH"
    Write-Host "   2. Verify installation: promptbridge --version"
    Write-Host "   3. Configure keyboard shortcut: promptbridge install-shortcut"
    Write-Host ""

} finally {
    # Clean up temporary directory
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
