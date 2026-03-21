---
description: Workflow for releasing a new version of WovenSnake
---

## When to release

Only create a release when **Rust source code changes** (`src/`, `Cargo.toml`).

| Change | Bump | Example |
|---|---|---|
| Bug fix | PATCH | `0.3.5 → 0.3.6` |
| New feature | MINOR | `0.3.x → 0.4.0` |
| Breaking change | MAJOR | `0.x → 1.0.0` |

Changes to `scripts/`, `README.md`, `CHANGELOG.md`, or `.agent/` go directly to `main` — **no tag, no release**. Scripts are served from the `main` branch URL and are immediately available without a new binary.

---

## Release steps

1. **Ensure `main` is up-to-date**:
   ```bash
   git checkout main && git pull origin main
   ```

2. **Bump version in `Cargo.toml`**:
   ```toml
   version = "X.Y.Z"
   ```

3. **Update `CHANGELOG.md`** — add a new section at the top:
   ```markdown
   ## [X.Y.Z] - YYYY-MM-DD

   ### Added / Changed / Fixed
   - ...
   ```

4. **Verify locally**:
   ```bash
   cargo test
   cargo clippy -- -D warnings

   # macOS / Linux
   ./scripts/validate_playground.sh

   # Windows
   ./scripts/validate_playground.ps1
   ```

5. **Commit**:
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore: release vX.Y.Z"
   git push origin main
   ```

6. **Tag and push** (this triggers the release workflow):
   ```bash
   git tag -a vX.Y.Z -m "WovenSnake vX.Y.Z"
   git push origin vX.Y.Z
   ```

7. **Monitor the workflow** at Actions → Release & Publish. It will build:
   - `woven-linux-amd64` (musl static binary)
   - `woven-macos-amd64`
   - `woven-macos-arm64`
   - `woven-windows-amd64.exe`

8. **Post-release verification**:
   - Check the GitHub Release page for all 4 assets.
   - Test `curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.sh | bash` on a clean machine.
   - The Crates.io publish step will fail unless `CARGO_REGISTRY_TOKEN` is configured — that is expected if not publishing to crates.io.

---

## Manual release (without a new tag)

If you need to re-release an existing tag (e.g., to add a missing asset):

1. Go to GitHub → Actions → **Release & Publish** → **Run workflow**.
2. Enter the existing tag (e.g., `v0.3.6`).
3. The workflow will check out that tag, build, and upload assets to the existing release.
