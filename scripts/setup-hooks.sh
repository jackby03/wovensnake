#!/bin/sh
# Configure git to use the shared hooks in .githooks/.
# Run this once after cloning the repository.
#
# Usage: sh scripts/setup-hooks.sh

set -e

git config core.hooksPath .githooks
echo "Git hooks configured. Pre-push format check is now active."
echo "To skip a hook on a specific push: git push --no-verify"
