.PHONY: help build dev release test clean format install lint all

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

all: clean format build test ## Run everything

build: dev ## Alias for dev build

dev: ## Build development version
	maturin develop

release: ## Build release version with optimizations
	maturin develop --release

test: ## Run tests
	pytest tests/ -v

clean: ## Clean build artifacts
	rm -rf target/
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info
	rm -rf sup.egg-info/
	rm -rf .pytest_cache/
	rm -rf __pycache__/
	rm -rf sup/__pycache__/
	rm -rf tests/__pycache__/
	rm -rf sup/bin/
	rm -f Cargo.lock
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete
	find . -type f -name "*.so" -delete
	find . -type f -name "*.pyd" -delete

format: format-rust format-python ## Format all code

format-rust: ## Format Rust code
	cargo fmt

format-python: ## Format Python code with black and isort
	@command -v black >/dev/null 2>&1 || { echo "Installing black..."; pip install black; }
	@command -v isort >/dev/null 2>&1 || { echo "Installing isort..."; pip install isort; }
	black sup/ tests/
	isort sup/ tests/

lint: lint-rust lint-python ## Run all linters

lint-rust: ## Lint Rust code
	cargo clippy -- -D warnings

lint-python: ## Lint Python code with ruff
	@command -v ruff >/dev/null 2>&1 || { echo "Installing ruff..."; pip install ruff; }
	ruff check sup/ tests/

install: ## Install the package in editable mode
	pip install -e .

wheel: ## Build wheel distribution
	maturin build --release

dev-deps: ## Install development dependencies
	pip install maturin pytest black isort ruff

ci: ## Run CI checks (format check, lint, test)
	cargo fmt -- --check
	cargo clippy -- -D warnings
	black --check sup/ tests/
	pytest tests/