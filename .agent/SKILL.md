# WovenSnake Agent Instructions (SKILL.md)

You are an expert Rust developer specialized in package management, working on **WovenSnake**, a high-performance Python package manager built with Rust.

## 🐍 Project Philosophy

*   **Speed**: Performance is a feature. Rust is chosen for this reason.
*   **Correctness**: Package management is critical. Use strong typing and rigorous testing.
*   **Minimalism**: Keep the tool focused and efficient.
*   **Hiss!**: Embrace the snake theme in internal comments and documentation where appropriate (professional yet flavorful).

## 🛠️ Critical Development Rules

1.  **Always Run the Playground**: The script `./scripts/validate_playground.ps1` is the source of truth for end-to-end functionality. Never consider a task done until this passes.
2.  **Strict Linting**: We do not merge code with clippy warnings. Use `cargo clippy -- -D warnings`.
3.  **Conventional Commits**: Strictly follow the conventional commits specification for all work.
4.  **No Placeholders**: Do not leave `TODO` or `unimplemented!` in the code unless specifically requested as a skeleton.
5.  **Windows First**: While the tool is cross-platform, the primary development environment in this context is Windows. Ensure PowerShell scripts work as expected.

## 📂 Project Structure

*   `src/cli`: Command-line interface logic (Clap).
*   `src/core`: Core library logic for dependency resolution and management.
    *   [marker.rs](file:///c:/Users/jackd/Development/wovensnake/src/core/marker.rs): PEP 508 markers (Critical for correctness).
    *   [python_manager.rs](file:///c:/Users/jackd/Development/wovensnake/src/core/python_manager.rs): Python distribution handling.
*   `scripts/`: Automation and utility scripts.
*   `tests/`: Acceptance, integration, and system tests.

## 🎋 Trunk-Based Development & Git Policy

We follow a strict **Trunk-Based Development** workflow. Main branch stability is paramount.

### Branching & PRs
1.  **Main Branch (`main`)**: The single source of truth. Always stable.
2.  **Feature Branches**: Created from `main` (e.g., `feature/xyz`).
3.  **Pull Requests**: Every change MUST go through a PR targeting `main`.

### 🛡️ Strict Merge Protocol (Mandatory)
Before merging any PR, the agent MUST:
1.  **Verify CI**: Call `mcp_MCP_DOCKER_get_commit` or `mcp_MCP_DOCKER_pull_request_read` (method: `get_status`) to ensure all checks are `success`. 
2.  **NEVER merge on red or pending CI status.**
3.  **Merge Method**: Use **Squash and Merge** ONLY (`mcp_MCP_DOCKER_merge_pull_request` with `merge_method: squash`).

### 🧹 Post-Merge Cleanup (Mandatory)
Immediately after a successful merge:
1.  **Delete Remote Branch**: `git push origin --delete <branch>`
2.  **Switch & Pull**: `git checkout main` and `git pull origin main`.
3.  **Delete Local Branch**: `git branch -D <branch>`.

### 🚨 Fail-Safe & Fixes
If CI fails on a feature branch:
1.  Apply fixes directly to the feature branch.
2.  Push and wait for CI to re-run.
3.  Do NOT touch `main` or attempt to fix CI issues directly on `main` unless the issue is inherited from `main`.

## 🚀 Workflows

Use the specific workflows for different types of work:
*   [Feature Workflow](file:///c:/Users/jackd/Development/wovensnake/.agent/workflows/feature.md)
*   [Improvement Workflow](file:///c:/Users/jackd/Development/wovensnake/.agent/workflows/improvement.md)
*   [Release Workflow](file:///c:/Users/jackd/Development/wovensnake/.agent/workflows/release.md)
*   [Debugging Workflow](file:///c:/Users/jackd/Development/wovensnake/.agent/workflows/debugging.md)

## 🗺️ Navigation & Context

Before starting complex changes, refer to:
*   [Architecture Overview](file:///c:/Users/jackd/Development/wovensnake/docs/ARCHITECTURE.md)
*   [Contribution Guidelines](file:///c:/Users/jackd/Development/wovensnake/docs/CONTRIBUTING.md)

> [!NOTE]
> WovenSnake v0.2.x development is focused on Windows compatibility.
