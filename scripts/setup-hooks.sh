#!/bin/bash
# Setup script for wait-for development environment
# This script installs and configures pre-commit hooks and development tools

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Emoji for better UX
SUCCESS="✅"
ERROR="❌"
INFO="ℹ️"
WARNING="⚠️"
ROCKET="🚀"

echo -e "${BLUE}${ROCKET} Setting up wait-for development environment${NC}"
echo "=================================================="

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install Rust tools
install_rust_tools() {
    echo -e "\n${INFO} Installing required Rust tools..."

    # Install rustfmt and clippy if not already installed
    if ! rustup component list --installed | grep -q rustfmt; then
        echo "Installing rustfmt..."
        rustup component add rustfmt
    fi

    if ! rustup component list --installed | grep -q clippy; then
        echo "Installing clippy..."
        rustup component add clippy
    fi

    # Install additional cargo tools
    local tools=(
        "cargo-audit:security auditing"
        "cargo-deny:license and security checking"
        "cargo-machete:unused dependency detection"
        "cargo-outdated:dependency update checking"
        "cargo-release:release automation"
        "cargo-watch:file watching"
        "cargo-expand:macro expansion"
        "cargo-llvm-cov:code coverage"
    )

    for tool_info in "${tools[@]}"; do
        IFS=':' read -r tool description <<< "$tool_info"

        if ! command_exists "$tool"; then
            echo "Installing $tool ($description)..."
            cargo install "$tool" || echo -e "${WARNING} Failed to install $tool, continuing..."
        else
            echo -e "${SUCCESS} $tool already installed"
        fi
    done
}

# Function to setup pre-commit
setup_precommit() {
    echo -e "\n${INFO} Setting up pre-commit hooks..."

    if command_exists python3; then
        # Install pre-commit via pip
        if ! command_exists pre-commit; then
            echo "Installing pre-commit..."
            if command_exists pip3; then
                pip3 install --user pre-commit
            elif command_exists pip; then
                pip install --user pre-commit
            else
                echo -e "${WARNING} pip not found, skipping pre-commit installation"
                return 1
            fi
        else
            echo -e "${SUCCESS} pre-commit already installed"
        fi

        # Install pre-commit hooks
        echo "Installing pre-commit hooks..."
        pre-commit install --hook-type pre-commit
        pre-commit install --hook-type commit-msg
        pre-commit install --hook-type pre-push

        echo -e "${SUCCESS} Pre-commit hooks installed"

        # Optional: run pre-commit on all files to verify setup
        read -p "Run pre-commit on all files to verify setup? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "Running pre-commit on all files..."
            pre-commit run --all-files || echo -e "${WARNING} Some pre-commit checks failed"
        fi

    else
        echo -e "${WARNING} Python not found, skipping pre-commit setup"
        return 1
    fi
}

# Function to setup lefthook (faster alternative)
setup_lefthook() {
    echo -e "\n${INFO} Setting up Lefthook (fast alternative to pre-commit)..."

    if command_exists lefthook; then
        echo -e "${SUCCESS} Lefthook already installed"
    else
        echo "Installing Lefthook..."

        # Try different installation methods
        if command_exists brew; then
            brew install lefthook
        elif command_exists go; then
            go install github.com/evilmartians/lefthook@latest
        elif command_exists npm; then
            npm install -g @arkweid/lefthook
        else
            echo -e "${WARNING} Could not install Lefthook automatically"
            echo "Please install manually: https://github.com/evilmartians/lefthook#installation"
            return 1
        fi
    fi

    # Install lefthook hooks
    echo "Installing Lefthook hooks..."
    lefthook install

    echo -e "${SUCCESS} Lefthook hooks installed"
}

# Function to create secrets baseline for detect-secrets
setup_secrets_baseline() {
    echo -e "\n${INFO} Setting up secrets detection baseline..."

    if command_exists detect-secrets; then
        if [ ! -f ".secrets.baseline" ]; then
            echo "Creating secrets baseline..."
            detect-secrets scan --baseline .secrets.baseline
            echo -e "${SUCCESS} Secrets baseline created"
        else
            echo -e "${SUCCESS} Secrets baseline already exists"
        fi
    else
        echo -e "${WARNING} detect-secrets not found, install with: pip install detect-secrets"
    fi
}

# Function to verify setup
verify_setup() {
    echo -e "\n${INFO} Verifying development setup..."

    # Check Rust toolchain
    if command_exists rustc && command_exists cargo; then
        echo -e "${SUCCESS} Rust toolchain: $(rustc --version)"
    else
        echo -e "${ERROR} Rust toolchain not found"
        return 1
    fi

    # Check formatting
    if cargo fmt --version >/dev/null 2>&1; then
        echo -e "${SUCCESS} rustfmt: $(cargo fmt --version)"
    else
        echo -e "${ERROR} rustfmt not available"
    fi

    # Check linting
    if cargo clippy --version >/dev/null 2>&1; then
        echo -e "${SUCCESS} clippy: $(cargo clippy --version)"
    else
        echo -e "${ERROR} clippy not available"
    fi

    # Check if project builds
    echo "Checking if project builds..."
    if cargo check --all-targets --all-features; then
        echo -e "${SUCCESS} Project builds successfully"
    else
        echo -e "${ERROR} Project build failed"
        return 1
    fi

    # Test hooks if installed
    if [ -f ".git/hooks/pre-commit" ]; then
        echo -e "${SUCCESS} Git hooks are installed"
    else
        echo -e "${WARNING} Git hooks not found"
    fi
}

# Function to show usage instructions
show_usage() {
    echo -e "\n${ROCKET} Development Environment Setup Complete!"
    echo "=============================================="
    echo
    echo -e "${INFO} Available options:"
    echo
    echo "🔧 Pre-commit (Python-based, more features):"
    echo "   pre-commit run --all-files    # Run all hooks"
    echo "   pre-commit run <hook-name>     # Run specific hook"
    echo "   pre-commit autoupdate          # Update hook versions"
    echo
    echo "⚡ Lefthook (Go-based, faster):"
    echo "   lefthook run pre-commit        # Run pre-commit hooks"
    echo "   lefthook run pre-push          # Run pre-push hooks"
    echo "   lefthook install               # Reinstall hooks"
    echo
    echo "🦀 Cargo aliases (defined in .cargo/config.toml):"
    echo "   cargo lint                     # Strict linting"
    echo "   cargo ci-check                 # Full CI check"
    echo "   cargo pre-release              # Release preparation"
    echo "   cargo audit                    # Security audit"
    echo
    echo "📚 Documentation:"
    echo "   cargo doc-open                 # Build and open docs"
    echo "   cargo doc-check                # Check docs build"
    echo
    echo -e "${SUCCESS} Happy coding! 🎉"
}

# Main execution
main() {
    echo -e "${INFO} Checking prerequisites..."

    # Check if we're in a git repository
    if [ ! -d ".git" ]; then
        echo -e "${ERROR} Not in a git repository"
        exit 1
    fi

    # Check if Rust is installed
    if ! command_exists rustc || ! command_exists cargo; then
        echo -e "${ERROR} Rust not found. Please install from https://rustup.rs/"
        exit 1
    fi

    # Install Rust tools
    install_rust_tools

    # Setup hooks (user choice)
    echo -e "\n${INFO} Choose hook system:"
    echo "1) Pre-commit (Python, more features, slower)"
    echo "2) Lefthook (Go, faster, fewer features)"
    echo "3) Both"
    echo "4) Skip hooks setup"

    read -p "Enter choice (1-4): " -n 1 -r choice
    echo

    case $choice in
        1)
            setup_precommit
            ;;
        2)
            setup_lefthook
            ;;
        3)
            setup_precommit
            setup_lefthook
            ;;
        4)
            echo "Skipping hooks setup"
            ;;
        *)
            echo -e "${WARNING} Invalid choice, skipping hooks setup"
            ;;
    esac

    # Setup additional tools
    setup_secrets_baseline

    # Verify everything works
    verify_setup

    # Show usage instructions
    show_usage
}

# Run main function
main "$@"
