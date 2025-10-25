#!/usr/bin/env bash

# Pre-commit hook for StarRocks Admin
# Following rustfs standard: fmt + clippy + check

set -e

echo "🔧 Running pre-commit checks..."
cd backend

# 1. Format code
echo "📝 Formatting code..."
cargo fmt --all

# 2. Run clippy (fix + strict check)
echo "🔍 Running clippy checks..."
DATABASE_URL="sqlite:../build/data/starrocks-admin.db" cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs --all-targets
DATABASE_URL="sqlite:../build/data/starrocks-admin.db" cargo clippy --all-targets --all-features -- -D warnings

# 3. Run cargo check
echo "🔨 Running cargo check..."
cargo check --all-targets

echo "✅ All pre-commit checks passed!"
