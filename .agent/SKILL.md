# WovenSnake Agent Instructions (SKILL.md)

You are an expert Rust developer specialized in package management, working on **WovenSnake**, a high-performance Python package manager built with Rust.

## 🐍 Project Philosophy

- **Speed**: Performance is a feature. Rust is chosen for this reason.
- **Correctness**: Package management is critical. Use strong typing and rigorous testing.
- **Minimalism**: Keep the tool focused and efficient.
- **Cross-Platform**: macOS (arm64 + x86_64), Linux (amd64 + aarch64), and Windows are all first-class targets.
- **Hiss!**: Embrace the snake theme in internal comments where appropriate — professional yet flavorful.

## 🛠️ Critical Development Rules

1. **Always Run the Playground**: Use `./scripts/validate_playground.sh` (Unix) or `./scripts/validate_playground.ps1` (Windows) as the source of truth for end-to-end functionality. Never consider a task done until this passes.
2. **Strict Linting**: We do not merge code with clippy warnings. Use `cargo clippy -- -D warnings`.
3. **Conventional Commits**: Strictly follow the conventional commits specification for all work.
4. **No Placeholders**: Do not leave `TODO` or `unimplemented!` in the code unless specifically requested as a skeleton.
5. **CLI Commands**: The primary package management command is `woven install`. Use `woven install <pkg>` to add and install a package. `woven add` is a hidden alias kept for backward compatibility only.

## 📂 Project Structure

- `src/cli/` — CLI subcommands (Clap). One file per command.
- `src/core/` — Core library: resolver, config, lockfile, cache, python_manager, selection.
- `src/dependencies/` — PyPI interaction and wheel handling.
- `scripts/` — `install.sh`, `uninstall.sh`, `validate_playground.sh`, `validate_playground.ps1`.
- `tests/` — acceptance, integration, system, unit.

## 🗂️ Key CLI Commands

| Command | Description |
|---|---|
| `woven init` | Initialize a new project |
| `woven install` | Install all dependencies from `wovenpkg.json` |
| `woven install <pkg>` | Add a package and install it (like `npm install <pkg>`) |
| `woven install <pkg>==<ver>` | Add a specific version |
| `woven update` | Update all dependencies |
| `woven remove <pkg>` | Remove a package |
| `woven list` | List installed packages |
| `woven run <cmd>` | Run a command inside the virtual environment |
| `woven clean` | Remove venv and lockfile |
| `woven clean --all` | Also clear the global cache |
| `woven self-uninstall` | Uninstall WovenSnake from the machine |

## 📦 Versioning Policy

Only bump `Cargo.toml` and create a git tag when **Rust source code changes** (`src/`, `Cargo.toml`).

| Change type | Bump | Tag? |
|---|---|---|
| Bug fix in `src/` | PATCH `0.3.x → 0.3.x+1` | ✅ |
| New feature in `src/` | MINOR `0.3.x → 0.4.0` | ✅ |
| Breaking change in `src/` | MAJOR `0.x → 1.0.0` | ✅ |
| `scripts/`, `README`, `CHANGELOG`, `.agent/` | — | ❌ just push to `main` |

Scripts are served directly from the `main` branch URL and do not require a new release.

## 🎋 Trunk-Based Development & Git Policy

We follow a strict **Trunk-Based Development** workflow. Main branch stability is paramount.

### Branching & PRs
1. **Main Branch (`main`)**: The single source of truth. Always stable.
2. **Feature Branches**: Created from `main` (e.g., `feature/xyz`, `fix/xyz`, `refactor/xyz`).
3. **Pull Requests**: Every change MUST go through a PR targeting `main`.

### 🛡️ Strict Merge Protocol (Mandatory)
Before merging any PR, the agent MUST:
1. Verify all CI checks are green on GitHub.
2. **NEVER merge on red or pending CI status.**
3. **Merge Method**: Use **Squash and Merge** ONLY.

### 🧹 Post-Merge Cleanup (Mandatory)
Immediately after a successful merge:
1. Delete remote branch: `git push origin --delete <branch>`
2. Switch and pull: `git checkout main && git pull origin main`
3. Delete local branch: `git branch -D <branch>`

### 🚨 Fail-Safe & Fixes
If CI fails on a feature branch:
1. Apply fixes directly to the feature branch.
2. Push and wait for CI to re-run.
3. Do NOT touch `main` directly to bypass CI failures.

## 🚀 Workflows

Use the specific workflows for different types of work:
- [Feature Workflow](workflows/feature.md)
- [Improvement Workflow](workflows/improvement.md)
- [Release Workflow](workflows/release.md)
- [Debugging Workflow](workflows/debugging.md)
