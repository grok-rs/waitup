# Makefile for wait-for project
# Provides convenient commands for common development tasks

.PHONY: help lint-quick lint-reasonable lint-strict lint-ci lint-fix format check test clean install

help: ## Show this help message
	@echo "Available commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Lint commands
lint-quick: ## Run quick lints (pre-commit level)
	@scripts/lint.sh quick

lint-reasonable: ## Run reasonable lints (development level)
	@scripts/lint.sh reasonable

lint-strict: ## Run strict lints (pre-push level)
	@scripts/lint.sh strict

lint-ci: ## Run CI-equivalent lints
	@scripts/lint.sh ci

lint-fix: ## Auto-fix formatting and clippy issues
	@scripts/lint.sh fix

# Individual checks
format: ## Check code formatting
	@cargo fmt --all -- --check

format-fix: ## Fix code formatting
	@cargo fmt --all

check: ## Quick compile check
	@cargo check --all-targets --all-features

test: ## Run tests
	@cargo test --all-features

test-quick: ## Run quick unit tests only
	@cargo test --lib --quiet

doc: ## Build documentation
	@cargo doc --no-deps --all-features

doc-open: ## Build and open documentation
	@cargo doc --no-deps --all-features --open

# Utility commands
clean: ## Clean build artifacts
	@cargo clean

install: ## Install development tools
	@echo "Installing development tools..."
	@cargo install cargo-audit cargo-deny cargo-machete

# Default target
.DEFAULT_GOAL := help
