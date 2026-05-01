#!/bin/bash
set -euo pipefail

# Run tests first
echo "Running tests..."
if ! cargo test; then
    echo "Tests failed. Aborting publish."
    exit 1
fi

echo "Running release mixed runtime speed gate..."
if ! cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture; then
    echo "Release mixed runtime speed gate failed. Aborting publish."
    exit 1
fi

# Bump patch version in Cargo.toml
echo "Bumping patch version..."
CURRENT_VERSION=$(grep '^version = "' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
PATCH=$((PATCH + 1))
NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
sed -i '' "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
echo "Version bumped: ${CURRENT_VERSION} -> ${NEW_VERSION}"

# Build for web
echo "Building web Wasm package..."
wasm-pack build --target web --out-dir pkg/web --out-name mons-web

# Build for nodejs 
echo "Building node Wasm package..."
wasm-pack build --target nodejs --out-dir pkg/node --out-name mons-rust

# Modify package.json to use mons-web as the name
sed -i '' 's/"name": "mons-rust"/"name": "mons-web"/' pkg/web/package.json
# Verify the change was made
if grep -q '"name": "mons-web"' pkg/web/package.json; then
    echo "Package name successfully changed to mons-web"
else
    echo "Failed to change package name to mons-web"
    exit 1
fi

echo "Checking release package surface..."
./scripts/assert-release-package-surface.sh pkg/web pkg/node

# Publish web package
cd pkg/web
npm publish --access public

# Publish nodejs package
cd ../node
# Ensure the package.json has the correct name for nodejs (should already be mons-rust)
npm publish --access public

# Return to project root
cd ../..

# Remove build artifacts
rm -rf pkg
