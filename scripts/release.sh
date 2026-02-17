#!/usr/bin/env bash
set -euo pipefail

# Surge TUI Release Script
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0

VERSION=${1:-}

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

echo "🚀 Releasing surge-tui v$VERSION"
echo ""

# 1. Update version in Cargo.toml
echo "📝 Updating Cargo.toml version..."
sed -i.bak "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# 2. Build release binaries
echo "🔨 Building release binaries..."
echo "  - English version (default)"
/Users/keyp/.cargo/bin/cargo build --release

echo "  - Chinese version"
/Users/keyp/.cargo/bin/cargo build --release --no-default-features --features lang-zh-cn
mv target/release/surge-tui target/release/surge-tui-zh

# 3. Run tests
echo "🧪 Running tests..."
/Users/keyp/.cargo/bin/cargo test --release

# 4. Create archives
echo "📦 Creating archives..."
mkdir -p dist

# macOS universal binary (if needed, otherwise skip)
# lipo -create target/release/surge-tui target/aarch64-apple-darwin/release/surge-tui \
#      -output dist/surge-tui-universal

# English version
tar -czf "dist/surge-tui-v$VERSION-macos-en.tar.gz" -C target/release surge-tui
echo "  ✓ dist/surge-tui-v$VERSION-macos-en.tar.gz"

# Chinese version
tar -czf "dist/surge-tui-v$VERSION-macos-zh.tar.gz" -C target/release surge-tui-zh
echo "  ✓ dist/surge-tui-v$VERSION-macos-zh.tar.gz"

# 5. Calculate SHA256
echo "🔐 Calculating SHA256..."
(cd dist && shasum -a 256 surge-tui-v$VERSION-*.tar.gz > surge-tui-v$VERSION-checksums.txt)
cat dist/surge-tui-v$VERSION-checksums.txt

# 6. Git operations
echo "📌 Creating git tag..."
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION" || true
git tag -a "v$VERSION" -m "Release v$VERSION"

echo ""
echo "✅ Release preparation complete!"
echo ""
echo "Next steps:"
echo "  1. Push changes and tag:"
echo "     git push origin main"
echo "     git push origin v$VERSION"
echo ""
echo "  2. Create GitHub Release:"
echo "     - Go to https://github.com/YOUR_USERNAME/surge-tui/releases/new"
echo "     - Tag: v$VERSION"
echo "     - Upload files from dist/"
echo ""
echo "  3. Update Homebrew formula:"
echo "     - Update URL to the GitHub release"
echo "     - Update sha256 from checksums.txt"
echo "     - Submit PR to homebrew tap"
echo ""
echo "  4. Update Nix package:"
echo "     - Update version and rev"
echo "     - Run: nix-prefetch-url --unpack https://github.com/YOUR_USERNAME/surge-tui/archive/v$VERSION.tar.gz"
echo "     - Update sha256 in nix/default.nix"
echo "     - Submit PR to nixpkgs"
