param(
    [string]$Command = "build",
    [switch]$Release,
    [switch]$Llvm,
    [switch]$Test,
    [switch]$Bench,
    [switch]$Clean,
    [switch]$Doc
)

$ErrorActionPreference = "Stop"

Write-Host "=== BLAZE Compiler Build Script ===" -ForegroundColor Cyan
Write-Host ""

function Build-Project {
    param([bool]$IsRelease, [bool]$UseLlvm)
    
    $buildArgs = @("build")
    
    if ($IsRelease) {
        $buildArgs += "--release"
        Write-Host "Building in RELEASE mode..." -ForegroundColor Yellow
    } else {
        Write-Host "Building in DEBUG mode..." -ForegroundColor Yellow
    }
    
    if ($UseLlvm) {
        $buildArgs += "--features", "llvm"
        Write-Host "Enabling LLVM backend..." -ForegroundColor Green
    }
    
    Write-Host "Running: cargo $($buildArgs -join ' ')" -ForegroundColor Gray
    & cargo @buildArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Build successful!" -ForegroundColor Green
    } else {
        Write-Host "✗ Build failed!" -ForegroundColor Red
        exit 1
    }
}

function Run-Tests {
    Write-Host "Running tests..." -ForegroundColor Yellow
    cargo test -- --test-threads=1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ All tests passed!" -ForegroundColor Green
    } else {
        Write-Host "✗ Some tests failed!" -ForegroundColor Red
        exit 1
    }
}

function Run-Benchmarks {
    Write-Host "Running benchmarks..." -ForegroundColor Yellow
    cargo bench
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Benchmarks complete!" -ForegroundColor Green
    } else {
        Write-Host "✗ Benchmarks failed!" -ForegroundColor Red
        exit 1
    }
}

function Clean-Project {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Clean successful!" -ForegroundColor Green
    } else {
        Write-Host "✗ Clean failed!" -ForegroundColor Red
        exit 1
    }
}

function Generate-Docs {
    Write-Host "Generating documentation..." -ForegroundColor Yellow
    cargo doc --no-deps --open
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Documentation generated!" -ForegroundColor Green
    } else {
        Write-Host "✗ Documentation failed!" -ForegroundColor Red
        exit 1
    }
}

function Show-Help {
    Write-Host "BLAZE Compiler Build Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1 [options]"
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Command <cmd>  : Build command (build/test/bench/clean/doc)"
    Write-Host "  -Release        : Build in release mode"
    Write-Host "  -Llvm           : Enable LLVM backend"
    Write-Host "  -Test           : Run tests"
    Write-Host "  -Bench          : Run benchmarks"
    Write-Host "  -Clean          : Clean build artifacts"
    Write-Host "  -Doc            : Generate documentation"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1 -Release"
    Write-Host "  .\build.ps1 -Release -Llvm"
    Write-Host "  .\build.ps1 -Test"
    Write-Host "  .\build.ps1 -Bench -Release"
}

switch ($Command.ToLower()) {
    "build" {
        if ($Clean) {
            Clean-Project
        }
        Build-Project -IsRelease $Release -UseLlvm $Llvm
        if ($Test) {
            Run-Tests
        }
    }
    "test" {
        Build-Project -IsRelease $Release -UseLlvm $Llvm
        Run-Tests
    }
    "bench" {
        Build-Project -IsRelease $true -UseLlvm $Llvm
        Run-Benchmarks
    }
    "clean" {
        Clean-Project
    }
    "doc" {
        Generate-Docs
    }
    "help" {
        Show-Help
    }
    default {
        if ($Test) {
            Run-Tests
        } elseif ($Bench) {
            Run-Benchmarks
        } elseif ($Clean) {
            Clean-Project
        } elseif ($Doc) {
            Generate-Docs
        } else {
            Build-Project -IsRelease $Release -UseLlvm $Llvm
        }
    }
}

Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Cyan
