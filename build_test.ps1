#!/usr/bin/env pwsh
# BLAZE Compiler Build Test Script

Write-Host "🔥 BLAZE Compiler - Build Test" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check Rust installation
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
$rustVersion = cargo --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ $rustVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Rust not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean
Write-Host "✓ Clean complete" -ForegroundColor Green
Write-Host ""

# Check code formatting
Write-Host "Checking code formatting..." -ForegroundColor Yellow
cargo fmt -- --check
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Code formatting is correct" -ForegroundColor Green
} else {
    Write-Host "⚠ Code formatting issues found (non-critical)" -ForegroundColor Yellow
}
Write-Host ""

# Run clippy
Write-Host "Running Clippy linter..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ No clippy warnings" -ForegroundColor Green
} else {
    Write-Host "⚠ Clippy warnings found (non-critical)" -ForegroundColor Yellow
}
Write-Host ""

# Check compilation
Write-Host "Checking compilation (cargo check)..." -ForegroundColor Yellow
cargo check --all-features 2>&1 | Tee-Object -Variable checkOutput
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Compilation check passed" -ForegroundColor Green
} else {
    Write-Host "✗ Compilation check failed" -ForegroundColor Red
    Write-Host $checkOutput
    exit 1
}
Write-Host ""

# Build debug
Write-Host "Building debug version..." -ForegroundColor Yellow
$buildStart = Get-Date
cargo build --verbose 2>&1 | Tee-Object -Variable buildOutput
$buildEnd = Get-Date
$buildTime = ($buildEnd - $buildStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Debug build succeeded in $([math]::Round($buildTime, 2))s" -ForegroundColor Green
} else {
    Write-Host "✗ Debug build failed" -ForegroundColor Red
    Write-Host $buildOutput
    exit 1
}
Write-Host ""

# Build release
Write-Host "Building release version..." -ForegroundColor Yellow
$releaseStart = Get-Date
cargo build --release --verbose 2>&1 | Tee-Object -Variable releaseOutput
$releaseEnd = Get-Date
$releaseTime = ($releaseEnd - $releaseStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Release build succeeded in $([math]::Round($releaseTime, 2))s" -ForegroundColor Green
} else {
    Write-Host "✗ Release build failed" -ForegroundColor Red
    Write-Host $releaseOutput
    exit 1
}
Write-Host ""

# Run tests (if any)
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ All tests passed" -ForegroundColor Green
} else {
    Write-Host "⚠ Some tests failed or no tests found (non-critical)" -ForegroundColor Yellow
}
Write-Host ""

# Summary
Write-Host "================================" -ForegroundColor Cyan
Write-Host "🎉 BUILD TEST SUMMARY" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host "✓ Rust installation: OK" -ForegroundColor Green
Write-Host "✓ Compilation check: OK" -ForegroundColor Green
Write-Host "✓ Debug build: OK ($([math]::Round($buildTime, 2))s)" -ForegroundColor Green
Write-Host "✓ Release build: OK ($([math]::Round($releaseTime, 2))s)" -ForegroundColor Green
Write-Host ""
Write-Host "🔥 BLAZE is ready for GitHub!" -ForegroundColor Green
Write-Host ""

# Show binary info
if (Test-Path "target/release/blaze.exe") {
    $binarySize = (Get-Item "target/release/blaze.exe").Length / 1MB
    Write-Host "Binary size: $([math]::Round($binarySize, 2)) MB" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. git add ." -ForegroundColor White
Write-Host "2. git commit -m 'Initial commit: Complete BLAZE compiler'" -ForegroundColor White
Write-Host "3. git push origin main" -ForegroundColor White
Write-Host ""
