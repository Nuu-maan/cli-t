# PowerShell script to build and package release binaries

Write-Host "Building release binaries for all platforms..." -ForegroundColor Green
Write-Host ""

# Build Windows
Write-Host "Building Windows (x86_64-pc-windows-msvc)..." -ForegroundColor Yellow
Push-Location client
cargo build --release --target x86_64-pc-windows-msvc
Pop-Location

Write-Host ""
Write-Host "Creating archives..." -ForegroundColor Yellow

# Create Windows archive
if (Test-Path "client\target\x86_64-pc-windows-msvc\release\cli-t.exe") {
    Write-Host "Creating Windows archive..." -ForegroundColor Green
    $zipPath = "cli-t-x86_64-pc-windows-msvc.zip"
    if (Test-Path $zipPath) {
        Remove-Item $zipPath
    }
    Compress-Archive -Path "client\target\x86_64-pc-windows-msvc\release\cli-t.exe" -DestinationPath $zipPath
    Write-Host "Created: $zipPath" -ForegroundColor Green
}

Write-Host ""
Write-Host "Windows binary ready!" -ForegroundColor Green
Write-Host ""
Write-Host "Note: For Linux and macOS binaries, you need to:" -ForegroundColor Yellow
Write-Host "1. Use a Linux/Mac machine, OR" -ForegroundColor Yellow
Write-Host "2. Set up cross-compilation toolchains" -ForegroundColor Yellow
Write-Host ""
Write-Host "Archives are ready in the project root for uploading to GitHub releases." -ForegroundColor Green

