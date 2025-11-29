# PowerShell installation script for cli-t

$ErrorActionPreference = "Stop"

Write-Host "Installing cli-t..." -ForegroundColor Green
Write-Host ""

# Detect architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$Target = "x86_64-pc-windows-msvc"

# GitHub repository
$Repo = "Nuu-maan/cli-t"
$Version = "latest"

# Determine download URL
if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/cli-t-$Target.zip"
} else {
    $Url = "https://github.com/$Repo/releases/download/$Version/cli-t-$Target.zip"
}

# Create temp directory
$TempDir = Join-Path $env:TEMP "cli-t-install"
if (Test-Path $TempDir) {
    Remove-Item $TempDir -Recurse -Force
}
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    Write-Host "Downloading cli-t for $Target..." -ForegroundColor Yellow
    $ZipPath = Join-Path $TempDir "cli-t.zip"
    
    try {
        Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
    } catch {
        Write-Host "Failed to download. Make sure the release exists on GitHub." -ForegroundColor Red
        exit 1
    }
    
    # Extract
    Write-Host "Extracting..." -ForegroundColor Yellow
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
    
    # Determine install directory
    $LocalBin = Join-Path $env:USERPROFILE ".local\bin"
    if (-not (Test-Path $LocalBin)) {
        New-Item -ItemType Directory -Path $LocalBin | Out-Null
    }
    
    # Install binary
    Write-Host "Installing to $LocalBin..." -ForegroundColor Yellow
    $BinaryPath = Join-Path $TempDir "cli-t.exe"
    if (Test-Path $BinaryPath) {
        Copy-Item $BinaryPath $LocalBin -Force
    } else {
        # Try alternative name
        $AltPath = Get-ChildItem $TempDir -Filter "*.exe" | Select-Object -First 1
        if ($AltPath) {
            Copy-Item $AltPath.FullName (Join-Path $LocalBin "cli-t.exe") -Force
        } else {
            Write-Host "Binary not found in archive" -ForegroundColor Red
            exit 1
        }
    }
    
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

