---
description: Workflow for implementing new features in WovenSnake
---

Follow these steps when implementing a new feature for WovenSnake:

1.  **Branching**: Create a new branch from `main` with the prefix `feature/`.
    ```bash
    git checkout -b feature/your-feature-name main
    ```
2.  **Implementation**:
    *   Write your code in Rust.
    *   Ensure semantic consistency with the existing codebase (Hiss!).
    *   Follow the modular structure in `src/`.
3.  **Testing**:
    *   Add unit tests in the same file or under `tests/unit`.
    *   Add integration tests under `tests/integration`.
    *   Run all tests:
        ```bash
        cargo test
        ```
// turbo
4.  **Playground Validation**:
    *   Run the playground simulation to ensure no regressions in user flow:
        ```powershell
        ./scripts/validate_playground.ps1
        ```
    *   Verify the generated report in `reports/playground_report.html`.
5.  **Linting**:
    *   Run clippy and fmt:
        ```bash
        cargo clippy -- -D warnings
        cargo fmt -- --check
        ```
6.  **Committing**: Use conventional commits (e.g., `feat: provide better errors`).
7.  **PR Preparation**: Document your changes and link any relevant issues.
