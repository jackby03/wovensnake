# Project Context

## Stack
- Language: Rust 2024 Edition (stricter lints via Clippy)

## Architecture
- `src/cli/` — User-facing command line interface commands.
- `src/core/` — Domain logic (Python version management, metadata parsing, venv handling, etc.).
- `src/dependencies/` — Integration bounds.

## Conventions
- Use `cargo clippy -- -D warnings` and ensure `cargo fmt` formatting.
- Reference `.agents/skills/` for fine-grained task execution (e.g. `rust-coding`).
- CI validates pushes on `main` inside specific directories (like `src/`, `tests/`, `scripts/`).

## Crucial
- Ensure tests still pass using `cargo test` after refactoring.
