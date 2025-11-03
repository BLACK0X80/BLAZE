# 🔄 BLAZE CI/CD Documentation

## Overview

BLAZE uses GitHub Actions for continuous integration and deployment. The workflow automatically installs LLVM 15 on all platforms and runs comprehensive tests.

---

## 🎯 Workflow Jobs

### 1. **Test Job** (Multi-Platform)

Runs on: Ubuntu, Windows, macOS

**Steps:**
1. ✅ Checkout code
2. ✅ Install Rust (stable)
3. ✅ **Install LLVM 15 automatically**
   - Ubuntu: Via `llvm.sh` script
   - macOS: Via Homebrew
   - Windows: Via Chocolatey
4. ✅ Setup caching (registry, index, build)
5. ✅ Check code formatting
6. ✅ Run Clippy linter
7. ✅ Check compilation
8. ✅ Build debug version
9. ✅ Build release version
10. ✅ Run all tests
11. ✅ Run documentation tests

**Matrix:**
- Operating Systems: 3 (Ubuntu, Windows, macOS)
- Rust Versions: 1 (stable)
- **Total**: 3 configurations

### 2. **Build Without LLVM**

Runs on: Ubuntu

Tests that the project can be built without LLVM backend for contributors who don't have LLVM installed.

**Steps:**
1. ✅ Checkout code
2. ✅ Install Rust
3. ✅ Check with `--no-default-features`
4. ✅ Build with `--no-default-features`
5. ✅ Test with `--no-default-features`

### 3. **Build Release Binaries**

Runs on: Ubuntu, Windows, macOS

Builds production-ready binaries for all platforms.

**Artifacts:**
- `blaze-linux-x86_64`
- `blaze-windows-x86_64.exe`
- `blaze-macos-x86_64`

**Steps:**
1. ✅ Checkout code
2. ✅ Install Rust
3. ✅ **Install LLVM 15**
4. ✅ Build release with optimizations
5. ✅ Strip binary (Unix only)
6. ✅ Upload artifacts

### 4. **Code Quality Checks**

Runs on: Ubuntu

Ensures code quality and documentation standards.

**Checks:**
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Documentation build (`cargo doc`)

### 5. **Security Audit**

Runs on: Ubuntu

Scans dependencies for security vulnerabilities.

---

## 🔧 LLVM Installation Details

### Ubuntu/Linux

```bash
# Download and run official LLVM installation script
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 15 all

# Install development files
sudo apt-get install -y llvm-15-dev libpolly-15-dev

# Set environment variable
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
```

**Time**: ~3-5 minutes

### macOS

```bash
# Update Homebrew
brew update

# Install LLVM 15
brew install llvm@15

# Set environment variable
export LLVM_SYS_150_PREFIX=$(brew --prefix llvm@15)
```

**Time**: ~5-8 minutes

### Windows

```powershell
# Install via Chocolatey
choco install llvm --version=15.0.7 --allow-downgrade -y

# Set environment variable
$env:LLVM_SYS_150_PREFIX = "C:\Program Files\LLVM"
```

**Time**: ~4-6 minutes

---

## ⚡ Performance Optimization

### Caching Strategy

The workflow caches three critical paths:

1. **Cargo Registry** (`~/.cargo/registry`)
   - Contains downloaded crate metadata
   - Reduces download time

2. **Cargo Git Index** (`~/.cargo/git`)
   - Contains Git repositories of dependencies
   - Speeds up dependency resolution

3. **Build Artifacts** (`target/`)
   - Contains compiled dependencies
   - Dramatically reduces build time

**Cache Keys:**
- Registry: `{OS}-cargo-registry-{Cargo.lock hash}`
- Index: `{OS}-cargo-index-{Cargo.lock hash}`
- Build: `{OS}-{rust version}-cargo-build-{Cargo.lock hash}`

### Expected Build Times

| Job | First Run | Cached Run |
|-----|-----------|------------|
| Test (Ubuntu) | 12-15 min | 3-5 min |
| Test (Windows) | 15-18 min | 4-6 min |
| Test (macOS) | 14-17 min | 3-5 min |
| Build Release | 10-12 min | 2-4 min |
| Code Quality | 8-10 min | 2-3 min |

---

## 🎯 Workflow Triggers

### Push Events
- Branches: `main`, `master`, `develop`
- Runs all jobs

### Pull Request Events
- Target branches: `main`, `master`, `develop`
- Runs all jobs
- Must pass before merge

### Manual Trigger
- Available via "Actions" tab
- "Run workflow" button

---

## 📊 Status Badges

Add these to your README.md:

```markdown
![CI](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg)
![Build](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg?event=push)
![Tests](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg?job=test)
```

---

## 🔍 Viewing Results

### On GitHub
1. Go to your repository
2. Click **Actions** tab
3. Select a workflow run
4. View logs for each job

### Artifacts
1. Go to workflow run
2. Scroll to **Artifacts** section
3. Download binaries:
   - `blaze-linux-x86_64`
   - `blaze-windows-x86_64`
   - `blaze-macos-x86_64`

---

## ⚠️ Troubleshooting

### LLVM Installation Fails

**Ubuntu:**
```bash
# Manual fix
sudo apt-get update
sudo apt-get install -y wget lsb-release software-properties-common
```

**macOS:**
```bash
# Update Homebrew first
brew update
brew upgrade
```

**Windows:**
```powershell
# Ensure Chocolatey is updated
choco upgrade chocolatey
```

### Build Fails After LLVM Install

Check environment variable:
```bash
echo $LLVM_SYS_150_PREFIX  # Unix
echo $env:LLVM_SYS_150_PREFIX  # Windows
```

### Cache Issues

Clear caches:
1. Go to **Actions** → **Caches**
2. Delete all caches
3. Re-run workflow

---

## 🚀 Best Practices

### For Contributors

1. **Before Creating PR:**
   ```bash
   cargo fmt --all
   cargo clippy --all-features
   cargo test --all-features
   ```

2. **Check locally:**
   ```bash
   # Without LLVM
   cargo check --no-default-features
   
   # With LLVM
   cargo check --all-features
   ```

3. **Run full test suite:**
   ```bash
   cargo test --all-features --verbose
   ```

### For Maintainers

1. **Review CI logs** before merging
2. **Download artifacts** to test binaries
3. **Check security audit** results
4. **Monitor build times** for performance regression

---

## 📈 Metrics

### Current Status
- ✅ **Multi-platform support**: 3 platforms
- ✅ **Automatic LLVM installation**: All platforms
- ✅ **Comprehensive testing**: Unit, integration, doc tests
- ✅ **Code quality**: Format, lint, doc checks
- ✅ **Security**: Dependency auditing
- ✅ **Artifacts**: Binary builds for all platforms

### Future Improvements
- [ ] Code coverage reporting (codecov.io)
- [ ] Performance benchmarking
- [ ] Release automation
- [ ] Docker image building
- [ ] WebAssembly target

---

## 🔐 Security

### Secret Variables

None required for basic CI/CD.

Optional (for enhanced features):
- `CODECOV_TOKEN` - Code coverage reporting
- `CRATES_IO_TOKEN` - Automated publishing

### Audit

The workflow runs `cargo audit` to check for:
- Known security vulnerabilities
- Unmaintained dependencies
- Yanked crates

---

## 📝 Examples

### Successful Run

```
✓ Install LLVM (Ubuntu) - 3m 24s
✓ Check compilation - 1m 12s
✓ Build debug - 2m 34s
✓ Build release - 3m 45s
✓ Run tests - 1m 56s
```

### Failed Run

```
✓ Install LLVM (Ubuntu) - 3m 24s
✓ Check compilation - 1m 12s
✗ Run clippy - 0m 45s
  error: unused import: `std::collections::HashMap`
```

---

## 🎉 Summary

The BLAZE CI/CD workflow provides:

1. ✅ **Automatic LLVM installation** on all platforms
2. ✅ **Multi-platform testing** (Linux, Windows, macOS)
3. ✅ **Comprehensive test coverage**
4. ✅ **Code quality enforcement**
5. ✅ **Security auditing**
6. ✅ **Release binary building**
7. ✅ **Efficient caching** for fast builds

**Result**: Fully automated testing and deployment pipeline!

---

**Last Updated**: 2025-11-03  
**Workflow Version**: 2.0  
**Status**: Production Ready ✅
