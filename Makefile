.PHONY: help check fmt clippy build build-release test run run-dev run-release clean ci install-hooks bump-version

# Default target
help:
	@echo "Available commands:"
	@echo "  make check      - Run all checks (fmt, clippy, build, test)"
	@echo "  make fmt        - Check code formatting"
	@echo "  make fmt-fix    - Fix code formatting"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make build      - Build the project (debug)"
	@echo "  make build-release - Build the project (release)"
	@echo "  make test       - Run tests"
	@echo "  make run        - Run the game (debug)"
	@echo "  make run-dev    - Run the developer menu (levels and shop)"
	@echo "  make run-release - Run the game (release)"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make ci         - Run the Rust checks from CI"
	@echo "  make install-hooks - Install pre-commit hook"
	@echo "  make bump-version VERSION=0.6.1 - Set the version in Cargo.toml and Cargo.lock"

# Run all checks
check: fmt clippy build test

# Check formatting
fmt:
	@echo "Checking code formatting..."
	cargo fmt -- --check

# Fix formatting
fmt-fix:
	@echo "Fixing code formatting..."
	cargo fmt

# Run clippy
clippy:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Build the project
build:
	@echo "Building project (debug)..."
	cargo build --verbose

# Build the project (release)
build-release:
	@echo "Building project (release)..."
	cargo build --release --verbose

# Run tests
test:
	@echo "Running tests..."
	cargo test --verbose

# Run the game
run:
	@echo "Running the game (debug)..."
	cargo run

# Run the game with developer tools
run-dev:
	@echo "Running the game (developer menu)..."
	cargo run -- --dev

# Run the game (release)
run-release:
	@echo "Running the game (release)..."
	cargo run --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Rust checks run by CI
ci: fmt clippy build test

bump-version: export RELEASE_VERSION = $(value VERSION)
bump-version:
	@python3 scripts/release-version.py bump "$$RELEASE_VERSION"

# Install pre-commit hook
install-hooks:
	@command -v gitleaks >/dev/null 2>&1 || { echo "Install Gitleaks 8.30.1 or newer and add it to PATH first." >&2; exit 1; }
	@echo "Installing pre-commit hook..."
	@mkdir -p .git/hooks
	@cp scripts/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed successfully!"

