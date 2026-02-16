# 🐍 Contributing to WovenSnake

Hiss! Welcome to the nest. We are excited that you want to contribute to **WovenSnake**.
To ensure a smooth molting process (PR review), please follow these guidelines.

## 🛠️ The Development Flow

1.  **Fork** the repository to your own GitHub account.
2.  **Clone** your fork locally.
3.  **Create a Branch** for your feature or fix:
    *   `feature/new-fang`
    *   `fix/broken-tail`
    *   `docs/better-hiss`

## 🧪 Testing (Critical)

We have a rigorous automated playground. **Before pushing**, you MUST verify your changes:

1.  **Unit & Integration Tests**:
    ```bash
    cargo test
    ```
    *All tests must pass green.*

2.  **The Playground Simulation**:
    Run the PowerShell automation script to verify end-to-end functionality:
    ```powershell
    ./scripts/validate_playground.ps1
    ```
    *This generates a report in `reports/playground_report.html`. Check it!*

3.  **Linting**:
    We use strict Clippy settings.
    ```bash
    cargo clippy -- -D warnings
    cargo fmt -- --check
    ```

## 📝 Pull Request Guidelines

*   **Title**: Use [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat: add sat solver`, `fix: install crash on windows`).
*   **Description**: Explain *what* you changed and *why*.
*   **Link Issues**: If this fixes a bug, link it (e.g., `Fixes #42`).
*   **One Thing Per PR**: Keep PRs small and focused.

## 🐛 Reporting Bugs

Please check existing issues first. If you find a new bug, open an issue using the **Bug Report** template.

Thank you for helping WovenSnake grow! 🐍
