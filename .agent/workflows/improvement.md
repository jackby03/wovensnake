---
description: Workflow for improvements and refactoring in WovenSnake
---

Follow these steps when making improvements or refactoring code in WovenSnake:

1.  **Branching**: Create a new branch from `main` with the prefix `refactor/` or `perf/`.
    ```bash
    git checkout -b refactor/your-improvement-name main
    ```
2.  **Benchmarking (Optional but Recommended)**:
    *   If making performance improvements, run existing benchmarks or create new ones to justify the change.
3.  **Refactoring**:
    *   Maintain the existing API where possible to avoid breaking changes.
    *   Keep the "Hiss!" spirit and project style.
4.  **Verification**:
    *   Run existing tests to ensure no regressions:
        ```bash
        cargo test
        ```
// turbo
5.  **Playground Validation**:
    *   Crucial for refactors! Ensure the end-to-end flow remains intact:
        ```powershell
        ./scripts/validate_playground.ps1
        ```
6.  **Linting**:
    *   Run clippy and fmt:
        ```bash
        cargo clippy -- -D warnings
        cargo fmt -- --check
        ```
7.  **PR Submission**:
    *   Commit using conventional commits.
    *   Create a PR targeting `main`.
8.  **Phase 3: Verification & Merge (Mandatory)**:
    *   **Wait for CI**: Ensure checks pass on GitHub.
    *   **Merge**: Squash and Merge only.
    *   **Cleanup**: Delete remote and local branches.
    *   **Sync**: Pull `main` locally.
