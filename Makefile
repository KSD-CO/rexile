.PHONY: help ci check fmt fmt-check clippy test build doc-test bench examples
.PHONY: per-file-bench memory-bench basic log-processing

# Default target
help:
	@echo "ReXile - Fast Regex Engine"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "CI & Testing:"
	@echo "  make ci                     - Full CI check (matches GitHub Actions)"
	@echo "  make check                  - Quick check (fmt, clippy, test)"
	@echo "  make fmt                    - Format code"
	@echo "  make fmt-check              - Check code formatting"
	@echo "  make clippy                 - Run clippy linter"
	@echo "  make test                   - Run all tests"
	@echo "  make build                  - Build project"
	@echo "  make doc-test               - Run documentation tests"
	@echo ""
	@echo "Benchmarks:"
	@echo "  make bench                  - Run all benchmarks"
	@echo "  make per-file-bench         - Real-world GRL file benchmark"
	@echo "  make memory-bench           - Memory comparison benchmark"
	@echo ""
	@echo "Examples:"
	@echo "  make examples               - Run key examples"
	@echo "  make basic                  - Basic usage example"
	@echo "  make log-processing         - Log processing example"

# =============================================================================
# CI CHECKS (matches GitHub Actions)
# =============================================================================

ci: fmt-check clippy build test doc-test examples
	@echo "✅ All CI checks passed!"

# Quick check without full build
check: fmt clippy test
	@echo "✅ Quick check passed!"

fmt:
	@echo "🔧 Formatting code..."
	@cargo fmt

fmt-check:
	@echo "🔍 Checking code formatting..."
	@cargo fmt -- --check

clippy:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets

test:
	@echo "🧪 Running tests..."
	@cargo test --verbose

build:
	@echo "🔨 Building project..."
	@cargo build --verbose

doc-test:
	@echo "📚 Running doc tests..."
	@cargo test --doc --verbose

# =============================================================================
# BENCHMARKS
# =============================================================================

bench:
	@echo "📊 Running benchmarks..."
	@cargo bench --bench rexile_benchmark

# =============================================================================
# EXAMPLES
# =============================================================================

examples: basic log-processing performance
	@echo "✅ Key examples completed!"

basic:
	@echo "=== Basic Usage Example ==="
	@cargo run --example basic_usage

log-processing:
	@echo "=== Log Processing Example ==="
	@cargo run --example log_processing

performance:
	@echo "=== Performance Example ==="
	@cargo run --release --example performance

# =============================================================================
# DEVELOPMENT
# =============================================================================

# Build documentation and open in browser
doc:
	@echo "📖 Building documentation..."
	@cargo doc --no-deps --open

# Run clippy with all lints
clippy-pedantic:
	@echo "🔍 Running clippy (pedantic)..."
	@cargo clippy --all-targets -- -W clippy::pedantic

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

# Check package before publish
package-check:
	@echo "📦 Checking package..."
	@cargo package --list
	@echo ""
	@echo "To publish:"
	@echo "  1. Update version in Cargo.toml"
	@echo "  2. git tag v0.1.0"
	@echo "  3. git push origin v0.1.0"
	@echo "  4. GitHub Actions will auto-publish to crates.io"
