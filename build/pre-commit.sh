#!/usr/bin/env bash

# Pre-commit hook for StarRocks Admin
# This script runs clippy and rustfmt to ensure code quality

set -e

echo "Running clippy with strict checks..."
cd backend

# Fix clippy warnings automatically
cargo clippy --fix --all-targets --allow-dirty --allow-staged --allow-no-vcs -- --deny warnings --allow clippy::uninlined-format-args

# Format code
cargo fmt

# Run clippy check (fail if any warnings)
if ! cargo clippy --all-targets -- --deny warnings --allow clippy::uninlined-format-args; then
  echo "❌ Clippy check failed"
  exit 1
fi

echo "✅ All checks passed!"

