# PowerShell installation script for cli-t

$ErrorActionPreference = "Stop"

Write-Host "Installing cli-t..." -ForegroundColor Green
Write-Host ""

# GitHub repository
$Repo = "Nuu-maan/cli-t"
$Target = "x86_64-pc-windows-msvc"

# Try to download from releases first
$ReleaseUrl = "https://github.com/$Repo/releases/latest/download/cli-t-$Target.tar.gz"
$TempDir = Join-Path $env:TEMP "cli-t-install"
if (Test-Path $TempDir) {
    Remove-Item $TempDir -Recurse -Force
}
New-Item -ItemType Directory -Path $TempDir | Out-Null

$TarGzPath = Join-Path $TempDir "cli-t.tar.gz"

try {
    Write-Host "Checking for pre-built release..." -ForegroundColor Yellow
    
    try {
        Invoke-WebRequest -Uri $ReleaseUrl -OutFile $TarGzPath -UseBasicParsing -ErrorAction Stop
        Write-Host "Downloading pre-built binary..." -ForegroundColor Green
        
        # Extract tar.gz
        Write-Host "Extracting..." -ForegroundColor Yellow
        tar -xzf $TarGzPath -C $TempDir
        
        # Find the exe - check common names and search recursively
        $BinaryPath = $null
        $SearchNames = @("cli-t.exe", "cli_t.exe")
        foreach ($name in $SearchNames) {
            $found = Get-ChildItem $TempDir -Filter $name -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($found) {
                $BinaryPath = $found.FullName
                break
            }
        }
        
        # Fallback: find any exe
        if (-not $BinaryPath) {
            $AnyExe = Get-ChildItem $TempDir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($AnyExe) {
                $BinaryPath = $AnyExe.FullName
            }
        }
        
        if (-not $BinaryPath) {
            throw "Binary not found in archive"
        }
        
        Write-Host "Pre-built binary downloaded successfully!" -ForegroundColor Green
        
    } catch {
        Write-Host "No pre-built release found. Building from source..." -ForegroundColor Yellow
        
        # Check if Rust is installed
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Host "Error: Rust/Cargo is not installed." -ForegroundColor Red
            Write-Host "Please install Rust from https://rustup.rs/ first." -ForegroundColor Yellow
            Write-Host "Or wait for a pre-built release to be available." -ForegroundColor Yellow
            exit 1
        }
        
        # Clone and build
        Write-Host "Cloning repository..." -ForegroundColor Yellow
        $RepoDir = Join-Path $TempDir "cli-t"
        git clone --depth 1 "https://github.com/$Repo.git" $RepoDir
        
        if (-not $?) {
            Write-Host "Failed to clone repository." -ForegroundColor Red
            exit 1
        }
        
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
    }
    
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
    
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
} finally {
    # Cleanup
    if (Test-Path $TempDir) {
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}