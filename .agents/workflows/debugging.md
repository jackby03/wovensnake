---
description: Workflow for systematic debugging and troubleshooting in WovenSnake
---

Use this workflow when you encounter bugs, failing tests, or unexpected behaviour.

1. **Reproduce**:
   - Create a minimal reproduction case in `/tmp/woven-debug-test/`.
   - If it's a CLI issue, run through the basic flow: `woven init` → `woven install <pkg>` → `woven run python -c "import pkg"`.

2. **Enable Logging**:
   ```bash
   # macOS / Linux
   RUST_LOG=debug woven install

   # Windows (PowerShell)
   $env:RUST_LOG="debug"; .\woven.exe install
   ```

3. **Enable Backtraces**:
   ```bash
   # macOS / Linux
   RUST_BACKTRACE=1 woven install

   # Windows (PowerShell)
   $env:RUST_BACKTRACE=1; .\woven.exe install
   ```

4. **Isolate the Component**:
   - **CLI parsing** → `src/cli/`
   - **Dependency resolution / PyPI** → `src/core/resolver.rs`, `src/dependencies/`
   - **Artifact selection** → `src/core/selection.rs`
   - **Python management** → `src/core/python_manager.rs`
   - **Lockfile** → `src/core/lock.rs`
   - Run specific tests:
     ```bash
     cargo test core::resolver
     cargo test unit_models
     ```

5. **Common Issues**:
   | Symptom | Likely cause |
   |---|---|
   | `GLIBC_X.Y not found` | Linux binary not built with musl — check release.yml target |
   | Package not found on PyPI | Range specifier passed to versioned API endpoint — see `src/core/resolver.rs` |
   | `permission denied` on `.so` file | Unix mode not preserved on wheel extraction — see `src/dependencies/package.rs` |
   | Wrong wheel downloaded (Linux wheel on macOS) | Platform detection — see `current_platform()` in `src/cli/install.rs` |

6. **Inspect Project State**:
   - `wovenpkg.json` — declared dependencies
   - `wovenpkg.lock` — resolved artifact URLs and hashes
   - `~/.wovensnake/cache/` — global content-addressable cache

7. **Fix and Validate**:
   - Apply the fix on a `fix/` or `hotfix/` branch.
   - Run tests and playground validation:
     ```bash
     cargo test
     ./scripts/validate_playground.sh   # or .ps1 on Windows
     ```
   - Create a PR targeting `main`.

8. **Verification & Merge (Mandatory)**:
   - Wait for CI to go green. **Never merge on red.**
   - Merge method: **Squash and Merge** only.
   - Post-merge: delete remote and local branch, pull `main`.

> **Versioning note**: bug fixes in `src/` require a PATCH bump (`0.3.5 → 0.3.6`) and a release tag.
