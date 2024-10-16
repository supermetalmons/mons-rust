#!/bin/bash

# Build for web
wasm-pack build --target web --out-dir pkg/web --out-name mons-web

# Build for nodejs
wasm-pack build --target nodejs --out-dir pkg/node --out-name mons-rust

# Publish web package
cd pkg/web
# Modify package.json to use mons-web as the name
sed -i '' 's/"name": "mons-rust"/"name": "mons-web"/' package.json
# Verify the change was made
if grep -q '"name": "mons-web"' package.json; then
    echo "Package name successfully changed to mons-web"
else
    echo "Failed to change package name to mons-web"
    exit 1
fi
npm publish --access public

# Publish nodejs package
cd ../node
# Ensure the package.json has the correct name for nodejs (should already be mons-rust)
npm publish --access public

# Return to project root
cd ../..

# Remove build artifacts
rm -rf pkg