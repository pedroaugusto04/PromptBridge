# PromptBridge Installation Script for Windows
# This script downloads the latest binary and sets up configuration
# 
# Quick Install:
#   irm https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/scripts/install.ps1 | iex
#
# This script will:
#   1. Download the latest PromptBridge binary
#   2. Install it to your PATH
#   3. Run interactive configuration (init-config)
#   4. Install keyboard shortcut helper (install-shortcut)

Write-Host "=== PromptBridge Installation for Windows ===" -ForegroundColor Green
Write-Host ""

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$binary = "promptbridge-${arch}-pc-windows-msvc.zip"

# Create temporary directory
$tmpDir = Join-Path $env:TEMP "promptbridge-install"
New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

try {
    # Get latest release version
    Write-Host "Checking for latest release..." -ForegroundColor Cyan
    try {
        $latestVersion = (Invoke-RestMethod -Uri "https://api.github.com/repos/pedroaugusto04/PromptBridge/releases/latest").tag_name
        if (-not $latestVersion) {
            throw "Failed to get version"
        }
    } catch {
        Write-Host "Warning: Could not fetch latest version from GitHub API (rate limited)" -ForegroundColor Yellow
        Write-Host "Using fallback version: v0.3.3" -ForegroundColor Yellow
        $latestVersion = "v0.3.3"
    }
    Write-Host "Latest version: $latestVersion" -ForegroundColor Green

    # Download latest release
    Write-Host "Downloading PromptBridge $binary..." -ForegroundColor Cyan
    $latestUrl = "https://github.com/pedroaugusto04/PromptBridge/releases/download/${latestVersion}/${binary}"
    $zipPath = Join-Path $tmpDir "promptbridge.zip"
    
    try {
        Invoke-WebRequest -Uri $latestUrl -OutFile $zipPath -UseBasicParsing
    } catch {
        Write-Host "Failed to download binary. Please check your internet connection." -ForegroundColor Red
        exit 1
    }

    # Extract binary
    Write-Host "Extracting binary..." -ForegroundColor Cyan
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

    # Determine installation directory
    $installDir = Join-Path $env:USERPROFILE ".cargo\bin"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    # Install binary
    Write-Host "Installing to $installDir..." -ForegroundColor Cyan
    $binaryPath = Join-Path $tmpDir "promptbridge.exe"
    if (Test-Path $binaryPath) {
        Copy-Item -Path $binaryPath -Destination $installDir -Force
    } else {
        # Try without .exe extension
        $binaryPath = Join-Path $tmpDir "promptbridge"
        Copy-Item -Path $binaryPath -Destination (Join-Path $installDir "promptbridge.exe") -Force
    }

    # Create configuration directory
    Write-Host "Setting up configuration..." -ForegroundColor Cyan
    $configDir = Join-Path $env:APPDATA "promptbridge"
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null

    # Download example config if config doesn't exist
    $configPath = Join-Path $configDir "promptbridge.toml"
    if (-not (Test-Path $configPath)) {
        Write-Host "Creating default configuration..." -ForegroundColor Cyan
        $exampleUrl = "https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/promptbridge.example.toml"
        try {
            Invoke-WebRequest -Uri $exampleUrl -OutFile $configPath -UseBasicParsing
        } catch {
            Write-Host "Failed to download example config. You'll need to create it manually." -ForegroundColor Yellow
        }
    } else {
        Write-Host "Configuration already exists, skipping..." -ForegroundColor Yellow
    }

    # Add to PATH if needed
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir*") {
        Write-Host ""
        Write-Host "$installDir is not in your PATH." -ForegroundColor Yellow
        Write-Host "   Adding to PATH..." -ForegroundColor Cyan
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
        Write-Host "   PATH updated. You may need to restart your terminal." -ForegroundColor Green
    }

    # Run interactive configuration
    Write-Host ""
    Write-Host "Running interactive configuration..." -ForegroundColor Cyan
    $promptbridgePath = Join-Path $installDir "promptbridge.exe"
    & $promptbridgePath init-config

    # Install keyboard shortcut
    Write-Host ""
    Write-Host "Installing keyboard shortcut..." -ForegroundColor Cyan
    & $promptbridgePath install-shortcut

    Write-Host ""
    Write-Host "=== Installation Complete ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "IMPORTANT: You MUST restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "After restarting, verify installation:" -ForegroundColor Cyan
    Write-Host "   promptbridge --version"
    Write-Host ""
    Write-Host "Your configuration is already set up! The script ran:" -ForegroundColor Green
    Write-Host "   ✓ promptbridge init-config (interactive configuration)"
    Write-Host "   ✓ promptbridge install-shortcut (keyboard shortcut helper)"
    Write-Host ""
    Write-Host "To configure your global hotkey:" -ForegroundColor Cyan
    Write-Host "   1. Install AutoHotkey v2: https://www.autohotkey.com/"
    Write-Host "   2. Double-click: $env:APPDATA\promptbridge\promptbridge.ahk"
    Write-Host "   3. Press Ctrl+Alt+T to translate selected text"
    Write-Host ""

} finally {
    # Clean up temporary directory
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
