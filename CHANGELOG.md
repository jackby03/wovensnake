# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-02-20

### Fixed
- **Linux binary**: recompiled with `x86_64-unknown-linux-musl` (fully static, no glibc dependency). Resolves `GLIBC_2.39 not found` errors on older distributions.
- **TLS backend**: switched `reqwest` from `native-tls` (OpenSSL) to `rustls-tls` (pure Rust). Eliminates the OpenSSL system dependency, enabling musl builds and reducing binary size.

## [0.3.1] - 2026-02-20

### Fixed
- **Linux binary**: recompiled with `x86_64-unknown-linux-musl` (fully static, no glibc dependency). Resolves `GLIBC_2.39 not found` errors on older distributions.

## [0.3.0] - 2026-02-20

### Added
- **macOS/Linux Compatibility**: Full native support for macOS (arm64 & x86_64) and Linux (amd64 & aarch64). Platform detection is now based on compile-time target rather than defaulting to `manylinux` on all non-Windows platforms.
- **Wheel Platform Parsing**: New `platform_from_filename()` logic correctly classifies `macosx_*`, `manylinux_*`, `win_amd64`, and `py3-none-any` wheels from PyPI filenames.
- **Artifact Selection Fallback**: `select_artifact` now tries exact platform → family fallback (macOS arm64 → x86_64 via Rosetta 2, Linux aarch64 → manylinux) → universal wheel → source distribution.
- **Unix Permissions**: `.so` libraries and scripts extracted from wheels now have their Unix file modes preserved (fixes `permission denied` errors on macOS/Linux).
- **Python Symlink**: On Unix, `python → python3` symlink is created after managed Python installs so both names work.
- **Interactive Installer** (`scripts/install.sh`): ASCII banner, colored step progress, spinner animation, arch-aware binary selection (arm64 with amd64 Rosetta fallback), `--yes`, `--no-modify-path`, and `--install-dir` flags.
- **Bash Playground Validator** (`scripts/validate_playground.sh`): macOS/Linux equivalent of the existing PowerShell script, generating the same HTML report.
- **macOS arm64 Release Binary**: CI now builds a native `woven-macos-arm64` binary on `macos-14` runners.

### Fixed
- **PyPI Resolver**: Range specifiers (e.g. `>=2,<4`) are no longer passed directly to the versioned PyPI endpoint (`/pypi/{name}/{version}/json`), which returned 404. The resolver now fetches the latest release for range constraints and applies version filtering locally.
- **Installer Asset Check**: `install.sh` now follows HTTP redirects when checking if a GitHub Release asset exists, preventing false 404s on valid URLs.

## [0.2.0] - 2025-12-25

### Added
- **New Command**: `woven add <package>` to dynamically add dependencies from PyPI.
- **New Command**: `woven clean` to remove the virtual environment, local packages, and lockfile.
- **UX Module**: Standardized CLI output with colors, headers, and consistent status icons.
- **Global Cache**: Implemented a content-addressable global cache in `~/.wovensnake/cache` for faster installs across projects.
- **Multi-platform Support**: Improved PATH handling and dynamic `site-packages` resolution for Windows and Unix-like systems.
- **Python Distribution Manager**: Embedded a metadata catalog plus `scripts/fetch_python_metadata.py` so `python_manager` can resolve the latest assets from `python-build-standalone`, cache the URLs, and run UV-style post-install patches (EXTERNALLY-MANAGED + canonical executables).

### Changed
- **Binary Name**: Renamed the CLI command from `wovensnake` to `woven` for a more concise experience.
- **Lockfile Logic**: Enhanced lockfile synchronization to be more robust and handle platform-specific artifacts better.
- **UX Improvements**: Removed emojis from automated reports for a more professional look.
- **Validation Script**: Playground automation now captures stdout/stderr with `RUST_BACKTRACE=1`, marks non-zero steps as failures, and destroys the temporary directory it created.

### Fixed
- Hardcoded `python3.10` paths in installation logic.
- Cross-platform PATH separator issues in the `run` command.
- Redundant clones and unused imports identified by Clippy.

## [0.1.0] - 2025-12-20

### Added
- Initial release of WovenSnake.
- Basic `init`, `install`, `remove`, `list`, and `run` commands.
- Virtual environment management.
- Basic lockfile support (`wovenpkg.lock`).
- Parallel dependency resolution.
