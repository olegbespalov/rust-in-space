.PHONY: help check fmt clippy build build-release test run run-release clean ci install-hooks

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
	@echo "  make run-release - Run the game (release)"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make ci         - Run CI checks (same as CI pipeline)"
	@echo "  make install-hooks - Install pre-commit hook"

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

# Run the game (release)
run-release:
	@echo "Running the game (release)..."
	cargo run --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# CI checks (same as CI pipeline)
ci: fmt clippy build test

# Install pre-commit hook
install-hooks:
	@echo "Installing pre-commit hook..."
	@mkdir -p .git/hooks
	@cp scripts/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed successfully!"

