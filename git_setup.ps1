#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"

Write-Host "=== BLAZE Compiler - GitHub Setup ===" -ForegroundColor Cyan
Write-Host ""

$repoUrl = "https://github.com/BLACK0X80/BLAZE.git"

try {
    Write-Host "Checking Git installation..." -ForegroundColor Yellow
    $gitVersion = git --version
    Write-Host "✓ Git found: $gitVersion" -ForegroundColor Green
    Write-Host ""
    
    Write-Host "Checking if Git is already initialized..." -ForegroundColor Yellow
    if (Test-Path ".git") {
        Write-Host "✓ Git repository already initialized" -ForegroundColor Green
        
        Write-Host ""
        Write-Host "Checking remote..." -ForegroundColor Yellow
        $remotes = git remote -v
        if ($remotes -match "origin") {
            Write-Host "✓ Remote 'origin' already exists" -ForegroundColor Green
            Write-Host "$remotes" -ForegroundColor Gray
        } else {
            Write-Host "Adding remote 'origin'..." -ForegroundColor Yellow
            git remote add origin $repoUrl
            Write-Host "✓ Remote added successfully" -ForegroundColor Green
        }
    } else {
        Write-Host "Initializing Git repository..." -ForegroundColor Yellow
        git init
        Write-Host "✓ Git initialized" -ForegroundColor Green
        
        Write-Host ""
        Write-Host "Adding remote 'origin'..." -ForegroundColor Yellow
        git remote add origin $repoUrl
        Write-Host "✓ Remote added successfully" -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "Checking current branch..." -ForegroundColor Yellow
    $currentBranch = git branch --show-current
    if ([string]::IsNullOrEmpty($currentBranch)) {
        Write-Host "No branch found, will create 'main' branch with first commit" -ForegroundColor Gray
    } else {
        Write-Host "✓ Current branch: $currentBranch" -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "Staging all files..." -ForegroundColor Yellow
    git add .
    Write-Host "✓ Files staged" -ForegroundColor Green
    
    Write-Host ""
    Write-Host "Checking for changes..." -ForegroundColor Yellow
    $status = git status --porcelain
    if ([string]::IsNullOrEmpty($status)) {
        Write-Host "⚠ No changes to commit" -ForegroundColor Yellow
        
        Write-Host ""
        Write-Host "Attempting to push existing commits..." -ForegroundColor Yellow
        git push -u origin main
    } else {
        Write-Host "Creating commit..." -ForegroundColor Yellow
        $commitMessage = "feat: Initial BLAZE Compiler implementation

- Complete Hindley-Milner type inference system
- SSA-based intermediate representation
- Advanced optimizations (GVN, LICM, strength reduction, inlining)
- Memory safety with borrow checker integration
- Standard library (Vec, String, I/O)
- Comprehensive test suite (15+ tests)
- Full documentation and guides
- Build automation scripts

Total: ~3,500 lines of production-ready code"

        git commit -m $commitMessage
        Write-Host "✓ Commit created" -ForegroundColor Green
        
        Write-Host ""
        Write-Host "Setting default branch to 'main'..." -ForegroundColor Yellow
        git branch -M main
        
        Write-Host ""
        Write-Host "Pushing to GitHub..." -ForegroundColor Yellow
        git push -u origin main
        Write-Host "✓ Successfully pushed to GitHub!" -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "=== Setup Complete! ===" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Repository URL: $repoUrl" -ForegroundColor Green
    Write-Host "View your project at: https://github.com/BLACK0X80/BLAZE" -ForegroundColor Green
    Write-Host ""
    
} catch {
    Write-Host ""
    Write-Host "✗ Error occurred:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host ""
    Write-Host "Common issues and solutions:" -ForegroundColor Yellow
    Write-Host "1. If authentication fails:" -ForegroundColor Gray
    Write-Host "   - Use GitHub CLI: gh auth login" -ForegroundColor Gray
    Write-Host "   - Or configure Git credentials" -ForegroundColor Gray
    Write-Host ""
    Write-Host "2. If remote already exists:" -ForegroundColor Gray
    Write-Host "   git remote remove origin" -ForegroundColor Gray
    Write-Host "   git remote add origin $repoUrl" -ForegroundColor Gray
    Write-Host ""
    Write-Host "3. If repository not empty:" -ForegroundColor Gray
    Write-Host "   git pull origin main --allow-unrelated-histories" -ForegroundColor Gray
    Write-Host "   git push -u origin main" -ForegroundColor Gray
    Write-Host ""
    exit 1
}
