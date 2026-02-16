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
6.  **PR Submission**:
    *   Commit changes using conventional commits.
    *   Create a PR targeting `main`.
7.  **Phase 3: Verification & Merge (Mandatory)**:
    *   **Wait for CI**: Explicitly check CI status using `mcp_MCP_DOCKER_get_commit` or `mcp_MCP_DOCKER_pull_request_read`.
    *   **NEVER merge if CI is red or pending.**
    *   **Merge**: Squash and Merge into `main`.
    *   **Cleanup**:
        *   Delete remote branch: `git push origin --delete <branch>`
        *   Switch to main and pull: `git checkout main && git pull origin main`
        *   Delete local branch: `git branch -D <branch>`
