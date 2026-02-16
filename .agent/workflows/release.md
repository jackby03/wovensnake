---
description: Workflow for releasing a new version of WovenSnake
---

Follow these steps to prepare and trigger a new release:

1.  **Preparation**: Ensure you are on the `main` branch and it is up-to-date.
2.  **Version Bump**:
    *   Update the version in `Cargo.toml`.
    *   Update the version in `scripts/validate_playground.ps1` if necessary (look for `🐍 WovenSnake Playground Automation`).
3.  **Changelog**:
    *   Update `CHANGELOG.md` with new features, fixes, and improvements under a new version heading.
    *   Use the format `## [X.Y.Z] - YYYY-MM-DD`.
4.  **Final Verification**:
    ```bash
    cargo test
    ./scripts/validate_playground.ps1
    ```
5.  **Commit and Tag**:
    *   Commit the version bump and changelog update:
        ```bash
        git add Cargo.toml CHANGELOG.md scripts/validate_playground.ps1
        git commit -m "chore: release vX.Y.Z"
        ```
    *   Create a tag:
        ```bash
        git tag -a vX.Y.Z -m "Release vX.Y.Z"
        ```
6.  **Push**:
    *   Push the changes and the tag:
        ```bash
        git push origin main
        git push origin vX.Y.Z
        ```
7.  **Deployment**: The GitHub Action `release.yml` will automatically build the binaries and publish to Crates.io upon detection of the new tag.
8.  **Post-Release**: Verify the GitHub Release page and the Crates.io listing.
