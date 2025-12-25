# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
