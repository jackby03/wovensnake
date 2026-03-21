# 🧶 Contributing to WovenSnake

Thank you for your interest in contributing to WovenSnake! This guide walks you through everything you need to get started, from setting up your environment to opening a pull request.

---

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Cloning and Building](#cloning-and-building)
- [Running the Test Suite](#running-the-test-suite)
- [Code Conventions](#code-conventions)
- [Git Hooks](#git-hooks)
- [Branch Strategy](#branch-strategy)
- [Opening a Pull Request](#opening-a-pull-request)

---

## 🛠️ Prerequisites

Before contributing, make sure you have the following installed:

| Tool | Version | Purpose |
| :--- | :--- | :--- |
| **Rust** | stable (latest) | Build the project |
| **rustfmt** | bundled with Rust | Code formatting |
| **clippy** | bundled with Rust | Linting |
| **Python** | 3.12+ | Required by integration tests |

### Install the Rust Toolchain

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then add the required components:

```bash
rustup component add rustfmt clippy
```

Verify your setup:

```bash
rustc --version   # should print 'rustc 1.xx.x (stable)'
cargo --version
```

---

## 🚀 Cloning and Building

```bash
# Clone the repository
git clone https://github.com/jackby03/wovensnake.git
cd wovensnake

# Build the project in debug mode
cargo build

# Build an optimized release binary
cargo build --release
```

The release binary is placed at `target/release/woven`.

---

## 🧪 Running the Test Suite

WovenSnake has unit, integration, acceptance, and system tests. Run the full suite with:

```bash
cargo test --verbose
```

To run a specific test target:

```bash
cargo test --test unit_models
cargo test --test unit_python
cargo test --test integration_core
cargo test --test integration_features
cargo test --test acceptance
cargo test --test system
```

> **Note:** The integration and system tests require Python 3.12 to be available in your PATH.

---

## 📐 Code Conventions

All pull requests must pass the quality gates enforced by CI before merging.

### Formatting

WovenSnake uses [`rustfmt`](https://github.com/rust-lang/rustfmt) with a project-level configuration in [`.rustfmt.toml`](.rustfmt.toml) (`max_width = 120`, Unix newlines). Run the formatter before committing:

```bash
cargo fmt
```

To check formatting without modifying files (as CI does):

```bash
cargo fmt -- --check
```

### Linting

[Clippy](https://github.com/rust-lang/rust-clippy) is run with warnings-as-errors. Your code must be lint-free:

```bash
cargo clippy -- -D warnings
```

Project-level Clippy settings live in [`clippy.toml`](clippy.toml).

---

## 🪝 Git Hooks

A pre-push hook is included in `.githooks/pre-push` that runs `cargo fmt --check` before every push, blocking pushes with unformatted code. Activate it once after cloning:

```bash
git config core.hooksPath .githooks
```

To skip the hook for a single push (not recommended):

```bash
git push --no-verify
```

---

## 🌿 Branch Strategy

WovenSnake follows **Trunk-Based Development**:

- **`main`** is the single source of truth and is always in a releasable state.
- All work happens in **short-lived feature branches** branched directly from `main`.
- Branches should be small and focused — one feature or fix per branch.
- Merge back to `main` via a pull request as soon as the work is complete and CI is green.
- Avoid long-running branches; rebase on `main` frequently to minimise merge conflicts.

Suggested branch naming:

```
feat/<short-description>
fix/<short-description>
docs/<short-description>
chore/<short-description>
```

---

## 🤝 Opening a Pull Request

1. **Fork** the repository and create your branch from `main`.
2. Make your changes, ensuring all quality gates pass locally:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
3. **Push** your branch and open a pull request against `main`.
4. Fill in the pull request template — link the related issue and tick the checklist.
5. A maintainer will review your PR. Please respond to feedback promptly.
6. Once approved and CI is green, the maintainer will merge your PR.

---

## 💬 Need Help?

Open an [issue](https://github.com/jackby03/wovensnake/issues) if you have a question or run into a problem. We are happy to help!
