# Makefile for SSO Platform

.PHONY: help build test dev clean docker-build docker-run setup fmt clippy

# Default target
help:
	@echo "Available targets:"
	@echo "  build        - Build the project in release mode"
	@echo "  test         - Run all tests"
	@echo "  dev          - Start development server"
	@echo "  clean        - Clean build artifacts"
	@echo "  setup        - Set up development environment"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy linter"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  docker-down  - Stop Docker Compose"

# Build the project
build:
	cargo build --release --bin auth-platform

# Run tests
test:
	cargo test --all

# Start development server
dev:
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file from template"; fi
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file from template"; fi
	AUTH__ENVIRONMENT=development cargo run --bin auth-platform

# Clean build artifacts
clean:
	cargo clean
	docker system prune -f

# Set up development environment
setup:
	@echo "Setting up development environment..."
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file"; fi
	@echo "Please update .env file with your configuration"
	@echo "Install Rust if not already installed: https://rustup.rs/"

# Format code
fmt:
	cargo fmt --all

# Run clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Build Docker image
docker-build:
	docker build -t auth-platform .

# Run with Docker Compose
docker-run:
	docker-compose up -d

# Stop Docker Compose
docker-down:
	docker-compose down

# Run database migrations (when implemented)
migrate:
	@echo "Database migrations will be implemented in later tasks"

# Install development dependencies
install-deps:
	@echo "Installing development dependencies..."
	cargo install sqlx-cli --no-default-features --features rustls,mysql,sqlite
	cargo install cargo-tarpaulin  # For test coverage
	cargo install cargo-watch      # For file watching during development

# Watch for changes and rebuild
watch:
	cargo watch -x "run"

# Run security audit
audit:
	cargo audit

# Check for outdated dependencies
outdated:
	cargo outdated