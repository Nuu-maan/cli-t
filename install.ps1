# PowerShell installation script for cli-t

$ErrorActionPreference = "Stop"

Write-Host "Installing cli-t..." -ForegroundColor Green
Write-Host ""

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust/Cargo is not installed." -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/ first." -ForegroundColor Yellow
    exit 1
}

# GitHub repository
$Repo = "Nuu-maan/cli-t"

# Create temp directory
$TempDir = Join-Path $env:TEMP "cli-t-install"
if (Test-Path $TempDir) {
    Remove-Item $TempDir -Recurse -Force
}
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    Write-Host "Building cli-t from source..." -ForegroundColor Yellow
    
    # Clone repository
    Write-Host "Cloning repository..." -ForegroundColor Yellow
    $RepoDir = Join-Path $TempDir "cli-t"
    git clone --depth 1 "https://github.com/$Repo.git" $RepoDir
    
    if (-not $?) {
        Write-Host "Failed to clone repository." -ForegroundColor Red
        exit 1
    }
    
    # Build
    Write-Host "Building client (this may take a few minutes)..." -ForegroundColor Yellow
    Push-Location (Join-Path $RepoDir "client")
    cargo build --release
    
    if (-not $?) {
        Write-Host "Build failed." -ForegroundColor Red
        exit 1
    }
    
    $BinaryPath = Join-Path $RepoDir "target\release\cli-t.exe"
    if (-not (Test-Path $BinaryPath)) {
        Write-Host "Binary not found after build." -ForegroundColor Red
        exit 1
    }
    
    Pop-Location
    
    # Determine install directory
    $LocalBin = Join-Path $env:USERPROFILE ".local\bin"
    if (-not (Test-Path $LocalBin)) {
        New-Item -ItemType Directory -Path $LocalBin | Out-Null
    }
    
    # Install binary
    Write-Host "Installing to $LocalBin..." -ForegroundColor Yellow
    Copy-Item $BinaryPath (Join-Path $LocalBin "cli-t.exe") -Force
    
    # Add to PATH if not already there
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$LocalBin*") {
        [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$LocalBin", "User")
        Write-Host "Added $LocalBin to PATH" -ForegroundColor Green
        Write-Host "You may need to restart your terminal for PATH changes to take effect" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "✓ cli-t installed successfully!" -ForegroundColor Green
    Write-Host "Run 'cli-t' to start chatting" -ForegroundColor Green
    Write-Host ""
    
} finally {
    # Cleanup
    if (Test-Path $TempDir) {
        Remove-Item $TempDir -Recurse -Force
    }
}
