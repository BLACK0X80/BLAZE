#!/bin/bash
# Test Coverage Script for BLAZE Compiler
# This script runs tests with coverage analysis using tarpaulin or grcov

set -e

echo -e "\033[36mBLAZE Compiler Test Coverage Analysis\033[0m"
echo -e "\033[36m======================================\033[0m"
echo ""

# Check if cargo-tarpaulin is installed
TARPAULIN_INSTALLED=false
if command -v cargo-tarpaulin &> /dev/null; then
    TARPAULIN_INSTALLED=true
    echo -e "\033[32m✓ cargo-tarpaulin is installed\033[0m"
else
    echo -e "\033[33m✗ cargo-tarpaulin is not installed\033[0m"
fi

# Check if grcov is installed
GRCOV_INSTALLED=false
if command -v grcov &> /dev/null; then
    GRCOV_INSTALLED=true
    echo -e "\033[32m✓ grcov is installed\033[0m"
else
    echo -e "\033[33m✗ grcov is not installed\033[0m"
fi

echo ""

if [ "$TARPAULIN_INSTALLED" = false ] && [ "$GRCOV_INSTALLED" = false ]; then
    echo -e "\033[33mNo coverage tool found.\033[0m"
    echo ""
    echo -e "\033[36mTo install cargo-tarpaulin, run:\033[0m"
    echo "  cargo install cargo-tarpaulin"
    echo ""
    echo -e "\033[36mOr to install grcov, run:\033[0m"
    echo "  cargo install grcov"
    echo ""
    exit 1
fi

# Run coverage with tarpaulin if available
if [ "$TARPAULIN_INSTALLED" = true ]; then
    echo -e "\033[36mRunning tests with cargo-tarpaulin...\033[0m"
    echo ""
    
    cargo tarpaulin \
        --out Html \
        --out Xml \
        --output-dir target/coverage \
        --exclude-files "tests/*" \
        --exclude-files "benches/*" \
        --exclude-files "examples/*" \
        --timeout 300 \
        --verbose
    
    echo ""
    echo -e "\033[32m✓ Coverage report generated successfully!\033[0m"
    echo "  HTML report: target/coverage/tarpaulin-report.html"
    echo "  XML report: target/coverage/cobertura.xml"
    echo ""
    
    # Open HTML report in browser (Linux/macOS)
    if [ -f "target/coverage/tarpaulin-report.html" ]; then
        echo -e "\033[36mOpening coverage report in browser...\033[0m"
        if command -v xdg-open &> /dev/null; then
            xdg-open target/coverage/tarpaulin-report.html
        elif command -v open &> /dev/null; then
            open target/coverage/tarpaulin-report.html
        fi
    fi
# Run coverage with grcov if tarpaulin is not available
elif [ "$GRCOV_INSTALLED" = true ]; then
    echo -e "\033[36mRunning tests with grcov...\033[0m"
    echo ""
    
    # Clean previous coverage data
    rm -rf target/coverage
    mkdir -p target/coverage
    
    # Set environment variables for coverage
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-Cinstrument-coverage"
    export LLVM_PROFILE_FILE="target/coverage/cargo-test-%p-%m.profraw"
    
    echo -e "\033[36mBuilding and running tests with coverage instrumentation...\033[0m"
    cargo test
    
    echo ""
    echo -e "\033[36mGenerating coverage report...\033[0m"
    
    grcov . \
        --binary-path ./target/debug/deps/ \
        --source-dir . \
        --output-type html \
        --branch \
        --ignore-not-existing \
        --ignore "tests/*" \
        --ignore "benches/*" \
        --ignore "examples/*" \
        --output-path target/coverage/html
    
    echo ""
    echo -e "\033[32m✓ Coverage report generated successfully!\033[0m"
    echo "  HTML report: target/coverage/html/index.html"
    echo ""
    
    # Open HTML report in browser (Linux/macOS)
    if [ -f "target/coverage/html/index.html" ]; then
        echo -e "\033[36mOpening coverage report in browser...\033[0m"
        if command -v xdg-open &> /dev/null; then
            xdg-open target/coverage/html/index.html
        elif command -v open &> /dev/null; then
            open target/coverage/html/index.html
        fi
    fi
    
    # Clean up environment variables
    unset CARGO_INCREMENTAL
    unset RUSTFLAGS
    unset LLVM_PROFILE_FILE
fi

echo ""
echo -e "\033[32mCoverage analysis complete!\033[0m"
