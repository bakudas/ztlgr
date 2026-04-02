.PHONY: help build run test clean install dev fmt lint doc

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## Build the project
	cargo build --release

run: ## Run the application
	cargo run --release

test: ## Run tests
	cargo test --all-features

clean: ## Clean build artifacts
	cargo clean

install: ## Install locally
	cargo install --path .

dev: ## Run in development mode
	cargo run

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy
	cargo clippy --all-features -- -D warnings

doc: ## Generate documentation
	cargo doc --no-deps --open

watch: ## Watch for changes and rebuild
	cargo watch -x build

# Quick test targets
test-unit: ## Run unit tests
	cargo test --lib

test-integration: ## Run integration tests
	cargo test --test integration

# Database management
db-new: ## Create a new vault
	@echo "Creating new vault..."
	@cargo run -- new $(ARGS)

db-backup: ## Backup current vault
	@echo "Backing up vault..."
	@cargo run -- backup

# Development setup
setup: ## Setup development environment
	rustup component add clippy rustfmt
	cargo install cargo-watch cargo-edit
	@echo "Development environment ready!"

# Check everything
check: fmt lint test ## Run all checks before commit
	@echo "All checks passed!"

# Release
release: ## Create a release build
	cargo build --release
	cargo test
	@echo "Release build ready in target/release/"