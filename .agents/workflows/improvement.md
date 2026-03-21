---
description: Workflow for improvements and refactoring in WovenSnake
---

Follow these steps when making improvements or refactoring code in WovenSnake:

1. **Branching**: Create a new branch from `main` with the prefix `refactor/` or `perf/`.
   ```bash
   git checkout -b refactor/your-improvement-name main
   ```

2. **Scope the Change**:
   - Identify what the improvement touches: CLI, core, dependencies, scripts, or docs.
   - If the change is limited to `scripts/`, `README.md`, or `.agent/` — no version bump or release tag is needed. Just push to `main`.
   - If `src/` or `Cargo.toml` is changed, a PATCH or MINOR bump is required.

3. **Benchmarking** (for performance work):
   - Run before and after to justify the change.
   - Use `cargo build --release` for representative timing.

4. **Refactoring**:
   - Maintain the existing public API where possible.
   - Keep cross-platform compatibility (`#[cfg(unix)]`, `#[cfg(windows)]`).
   - Do not introduce `woven add` references — the command is `woven install <pkg>`.

5. **Verification**:
   ```bash
   cargo test
   ```

6. **Playground Validation** — crucial for refactors that touch the install or resolution path:
   ```bash
   # macOS / Linux
   ./scripts/validate_playground.sh

   # Windows
   ./scripts/validate_playground.ps1
   ```

7. **Linting**:
   ```bash
   cargo clippy -- -D warnings
   cargo fmt -- --check
   ```

8. **Commit & PR**:
   - Use conventional commits: `refactor:`, `perf:`, `chore:`.
   - Create a PR targeting `main`.

9. **Verification & Merge (Mandatory)**:
   - Wait for CI to go green. **Never merge on red.**
   - Merge method: **Squash and Merge** only.

10. **Post-Merge Cleanup**:
    ```bash
    git push origin --delete refactor/your-improvement-name
    git checkout main && git pull origin main
    git branch -D refactor/your-improvement-name
    ```
