---
description: Workflow for systematic debugging and troubleshooting in WovenSnake
---

Use this workflow when you encounter bugs, failing tests, or unexpected behavior.

1.  **Reproduce**:
    *   Create a minimal reproduction case.
    *   If it's a CLI issue, try to reproduce it in the `playground` (using the same steps as in the failing scenario).
2.  **Enable Logging**:
    *   Run the command with `RUST_LOG=debug` to see detailed execution logs.
    *   On Windows (PowerShell): `$env:RUST_LOG="debug"; ./target/debug/woven.exe install`
3.  **Check Backtraces**:
    *   If the program crashes, enable backtraces: `$env:RUST_BACKTRACE=1`.
4.  **Isolate the Component**:
    *   Determine if the issue is in the CLI parsing, Python management, Venv creation, or Dependency resolution.
    *   Run specific unit tests for the suspected component: `cargo test core::marker`
5.  **Verify Environment Markers**:
    *   Many issues stem from incorrect environment marker evaluation. Check `src/core/marker.rs` logic against the current environment.
6.  **Inspection**:
    *   Inspect `wovenpkg.json` and `wovenpkg.lock` for inconsistencies.
    *   Check the `reports/playground_report.html` if the validation script failed.
7.  **Fix and Validate**:
    *   Apply the fix on a `fix/` or `hotfix/` branch.
    *   Run tests and `./scripts/validate_playground.ps1`.
    *   Create a PR targeting `main`.
8.  **Phase 3: Verification & Merge (Mandatory)**:
    *   **Wait for CI**: Ensure checks pass on GitHub.
    *   **Merge**: Squash and Merge only.
    *   **Cleanup**: Delete remote and local branches.
    *   **Sync**: Pull `main` locally.
