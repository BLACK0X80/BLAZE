# Test Coverage Script for BLAZE Compiler
# This script runs tests with coverage analysis using tarpaulin or grcov

Write-Host "BLAZE Compiler Test Coverage Analysis" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Check if cargo-tarpaulin is installed
$tarpaulinInstalled = $false
try {
    $null = cargo tarpaulin --version 2>&1
    $tarpaulinInstalled = $true
    Write-Host "✓ cargo-tarpaulin is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ cargo-tarpaulin is not installed" -ForegroundColor Yellow
}

# Check if grcov is installed
$grcovInstalled = $false
try {
    $null = grcov --version 2>&1
    $grcovInstalled = $true
    Write-Host "✓ grcov is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ grcov is not installed" -ForegroundColor Yellow
}

Write-Host ""

if (-not $tarpaulinInstalled -and -not $grcovInstalled) {
    Write-Host "No coverage tool found. Installing cargo-tarpaulin..." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To install cargo-tarpaulin, run:" -ForegroundColor Cyan
    Write-Host "  cargo install cargo-tarpaulin" -ForegroundColor White
    Write-Host ""
    Write-Host "Or to install grcov, run:" -ForegroundColor Cyan
    Write-Host "  cargo install grcov" -ForegroundColor White
    Write-Host ""
    exit 1
}

# Run coverage with tarpaulin if available
if ($tarpaulinInstalled) {
    Write-Host "Running tests with cargo-tarpaulin..." -ForegroundColor Cyan
    Write-Host ""
    
    cargo tarpaulin `
        --out Html `
        --out Xml `
        --output-dir target/coverage `
        --exclude-files "tests/*" `
        --exclude-files "benches/*" `
        --exclude-files "examples/*" `
        --timeout 300 `
        --verbose
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "✓ Coverage report generated successfully!" -ForegroundColor Green
        Write-Host "  HTML report: target/coverage/tarpaulin-report.html" -ForegroundColor White
        Write-Host "  XML report: target/coverage/cobertura.xml" -ForegroundColor White
        Write-Host ""
        
        # Open HTML report in browser
        $htmlReport = "target/coverage/tarpaulin-report.html"
        if (Test-Path $htmlReport) {
            Write-Host "Opening coverage report in browser..." -ForegroundColor Cyan
            Start-Process $htmlReport
        }
    } else {
        Write-Host ""
        Write-Host "✗ Coverage analysis failed" -ForegroundColor Red
        exit 1
    }
}
# Run coverage with grcov if tarpaulin is not available
elseif ($grcovInstalled) {
    Write-Host "Running tests with grcov..." -ForegroundColor Cyan
    Write-Host ""
    
    # Clean previous coverage data
    if (Test-Path "target/coverage") {
        Remove-Item -Recurse -Force "target/coverage"
    }
    New-Item -ItemType Directory -Force -Path "target/coverage" | Out-Null
    
    # Set environment variables for coverage
    $env:CARGO_INCREMENTAL = "0"
    $env:RUSTFLAGS = "-Cinstrument-coverage"
    $env:LLVM_PROFILE_FILE = "target/coverage/cargo-test-%p-%m.profraw"
    
    Write-Host "Building and running tests with coverage instrumentation..." -ForegroundColor Cyan
    cargo test
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "Generating coverage report..." -ForegroundColor Cyan
        
        grcov . `
            --binary-path ./target/debug/deps/ `
            --source-dir . `
            --output-type html `
            --branch `
            --ignore-not-existing `
            --ignore "tests/*" `
            --ignore "benches/*" `
            --ignore "examples/*" `
            --output-path target/coverage/html
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "✓ Coverage report generated successfully!" -ForegroundColor Green
            Write-Host "  HTML report: target/coverage/html/index.html" -ForegroundColor White
            Write-Host ""
            
            # Open HTML report in browser
            $htmlReport = "target/coverage/html/index.html"
            if (Test-Path $htmlReport) {
                Write-Host "Opening coverage report in browser..." -ForegroundColor Cyan
                Start-Process $htmlReport
            }
        } else {
            Write-Host ""
            Write-Host "✗ Coverage report generation failed" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host ""
        Write-Host "✗ Tests failed" -ForegroundColor Red
        exit 1
    }
    
    # Clean up environment variables
    Remove-Item Env:\CARGO_INCREMENTAL
    Remove-Item Env:\RUSTFLAGS
    Remove-Item Env:\LLVM_PROFILE_FILE
}

Write-Host ""
Write-Host "Coverage analysis complete!" -ForegroundColor Green
