# Installation Guide

## System Requirements

- **OS**: Windows 10+, macOS 10.15+, Ubuntu 20.04+
- **RAM**: Minimum 4GB (8GB recommended)
- **Disk Space**: 2GB for LLVM + 500MB for BLAZE

## Step-by-Step Installation

### 1. Install Rust

**All Platforms:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Restart your terminal, then verify:
```bash
rustc --version
cargo --version
```

### 2. Install LLVM 15

#### Ubuntu/Debian
```bash
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 15

sudo apt-get install -y llvm-15-dev libpolly-15-dev clang-15

echo 'export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15' >> ~/.bashrc
source ~/.bashrc
```

Verify:
```bash
llvm-config-15 --version
```

#### macOS
```bash
brew install llvm@15

echo 'export LLVM_SYS_150_PREFIX=$(brew --prefix llvm@15)' >> ~/.zshrc
source ~/.zshrc
```

Verify:
```bash
$(brew --prefix llvm@15)/bin/llvm-config --version
```

#### Windows

**Option 1: Chocolatey (Recommended)**
```powershell
choco install llvm --version=15.0.7

$env:LLVM_SYS_150_PREFIX = "C:\Program Files\LLVM"
[System.Environment]::SetEnvironmentVariable("LLVM_SYS_150_PREFIX", "C:\Program Files\LLVM", "User")
```

**Option 2: Manual Installation**
1. Download LLVM 15.0.7 from GitHub Releases
2. Run installer: LLVM-15.0.7-win64.exe
3. Add to PATH: C:\Program Files\LLVM\bin
4. Set LLVM_SYS_150_PREFIX=C:\Program Files\LLVM

Verify:
```powershell
llvm-config --version
```

### 3. Install C/C++ Compiler

#### Ubuntu/Debian
```bash
sudo apt-get install build-essential
```

#### macOS
```bash
xcode-select --install
```

#### Windows
```powershell
choco install mingw
```

### 4. Clone and Build BLAZE

```bash
git clone https://github.com/yourusername/blaze.git
cd blaze

cargo build --release

cargo install --path .
```

### 5. Verify Installation

```bash
blaze --version

echo 'fn main() { println("Hello!"); }' > test.blz
blaze check test.blz
```

## Troubleshooting

### "Could not find LLVM"

**Linux/macOS:**
```bash
find /usr -name "llvm-config*" 2>/dev/null

export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15

echo 'export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15' >> ~/.bashrc
source ~/.bashrc
```

**Windows:**
```powershell
where llvm-config

$env:LLVM_SYS_150_PREFIX = "C:\Program Files\LLVM"
[System.Environment]::SetEnvironmentVariable("LLVM_SYS_150_PREFIX", "C:\Program Files\LLVM", "User")
```

### Linker Error

**Ubuntu/Debian:**
```bash
sudo apt-get install build-essential
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
```powershell
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
```

### LLVM Version Mismatch

```bash
sudo apt-get remove llvm-14*
sudo ./llvm.sh 15
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
```

### Out of Memory

```bash
cargo build

cargo build --release -j 2
```

### Permission Denied (Linux/macOS)

```bash
sudo chown -R $USER:$USER ~/.cargo
cargo build --release
```

## Platform-Specific Notes

### Linux

**Fedora/RHEL:**
```bash
sudo dnf install llvm15-devel clang
```

**Arch Linux:**
```bash
sudo pacman -S llvm15 clang
```

### macOS

- Apple Silicon (M1/M2): LLVM works natively, no Rosetta needed
- Intel: Standard installation works

### Windows

- WSL2 is supported - follow Linux instructions
- Native Windows builds require Visual Studio 2019+
- MinGW-w64 is supported as alternative to MSVC

## Uninstallation

```bash
cargo uninstall blaze

cd blaze
cargo clean

sudo apt-get remove llvm-15*

brew uninstall llvm@15

choco uninstall llvm
```

## Next Steps

After installation:
1. Read the Language Guide in README.md
2. Try the Examples in examples/
3. Join our Community on GitHub Discussions
