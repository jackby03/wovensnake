---
description: Workflow for implementing new features in WovenSnake
---

Follow these steps when implementing a new feature for WovenSnake:

1. **Branching**: Create a new branch from `main` with the prefix `feature/`.
   ```bash
   git checkout -b feature/your-feature-name main
   ```

2. **Implementation**:
   - Write your code in Rust following the modular structure in `src/`.
   - One file per CLI subcommand in `src/cli/`. Add the module to `src/cli/mod.rs` and a match arm in `src/main.rs`.
   - Keep cross-platform compatibility in mind: use `#[cfg(unix)]` / `#[cfg(windows)]` where needed.
   - Do not use `woven add` in examples — the correct command is `woven install <pkg>`.

3. **Testing**:
   - Add unit tests in `tests/unit/` or inline `#[cfg(test)]` in the source file.
   - Add integration tests in `tests/integration/`.
   ```bash
   cargo test
   ```

4. **Playground Validation** — end-to-end smoke test:
   ```bash
   # macOS / Linux
   ./scripts/validate_playground.sh

   # Windows
   ./scripts/validate_playground.ps1
   ```
   Verify the generated report in `reports/playground_report.html`.

5. **Linting**:
   ```bash
   cargo clippy -- -D warnings
   cargo fmt -- --check
   ```

6. **Commit & PR**:
   - Use conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`.
   - Create a PR targeting `main`.

7. **Verification & Merge (Mandatory)**:
   - Wait for all CI checks to be green. **Never merge on red or pending CI.**
   - Merge method: **Squash and Merge** only.

8. **Post-Merge Cleanup**:
   ```bash
   git push origin --delete feature/your-feature-name
   git checkout main && git pull origin main
   git branch -D feature/your-feature-name
   ```

> **Versioning note**: features that change `src/` require a MINOR version bump (`0.3.x → 0.4.0`) and a release tag. Changes limited to `scripts/`, `README`, or `.agent/` do NOT need a tag — just push to `main`.
