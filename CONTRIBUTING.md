# Contributing to synapsis-core

Thank you for your interest in contributing! Here are some guidelines.

## Development Setup

```bash
git clone https://github.com/MethodWhite/synapsis-core.git
cd synapsis-core
cargo build
cargo test
```

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy --all-features` for linting  
- Follow Rust 2021 edition idioms
- Use `thiserror` for error types
- Document all public APIs with doc comments

## Feature Flags

synapsis-core uses feature flags extensively:

- `pqc` — Post-quantum cryptography (kyber, dilithium)
- `postgres` — PostgreSQL backend support
- `full` — All features enabled

When adding code behind a feature flag, ensure the feature-gated code compiles:

```bash
cargo build --features full
cargo test --features full --lib
```

## Pull Request Process

1. Ensure all existing tests pass
2. Add tests for new functionality
3. Update the CHANGELOG.md
4. Run `cargo clippy --all-features` and fix any warnings

## Commit Messages

Use conventional commits:

- `feat:` — New feature
- `fix:` — Bug fix
- `refactor:` — Code refactoring
- `test:` — Tests
- `docs:` — Documentation
- `ci:` — CI/CD changes
- `deps:` — Dependency updates

## Code of Conduct

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md) in all interactions.
