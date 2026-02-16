# Release Guide

This document describes how to release surge-tui to Homebrew and Nix.

## Prerequisites

1. GitHub repository set up
2. Homebrew tap repository created (optional, for personal tap)
3. Nix flakes enabled (optional, for Nix Flakes support)

## Release Process

### 1. Prepare Release

```bash
# Make release script executable
chmod +x scripts/release.sh

# Run release script
./scripts/release.sh 0.1.0
```

This script will:
- Update version in `Cargo.toml`
- Build release binaries (English and Chinese)
- Run tests
- Create tar archives
- Calculate SHA256 checksums
- Create git tag

### 2. Push to GitHub

```bash
# Push changes
git push origin main

# Push tag (triggers GitHub Actions release workflow)
git push origin v0.1.0
```

GitHub Actions will automatically:
- Build release binaries on macOS
- Create GitHub Release
- Upload artifacts

### 3. Install via Homebrew

#### Option A: Homebrew Tap (Recommended for initial release)

```bash
# 1. Create tap repository on GitHub
# Repository name: homebrew-tap or homebrew-surge

# 2. Add formula
cd ~/homebrew-tap
mkdir -p Formula
cp /path/to/surge-tui/homebrew/surge-tui.rb Formula/

# 3. Update formula
# - Replace YOUR_USERNAME with your GitHub username
# - Update sha256 from checksums.txt

# 4. Commit and push
git add Formula/surge-tui.rb
git commit -m "Add surge-tui formula"
git push

# 5. Users can install with:
brew tap YOUR_USERNAME/tap
brew install surge-tui
```

#### Option B: Official Homebrew Core (After project maturity)

1. Fork [homebrew-core](https://github.com/Homebrew/homebrew-core)
2. Add `Formula/surge-tui.rb`
3. Submit PR following [Homebrew contribution guide](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request)

### 4. Install via Nix

#### Option A: Nix Flakes (Recommended)

Users can install directly from GitHub:

```bash
# Install
nix profile install github:YOUR_USERNAME/surge-tui

# Or add to flake.nix
{
  inputs.surge-tui.url = "github:YOUR_USERNAME/surge-tui";
}
```

#### Option B: nixpkgs (After project maturity)

1. Fork [nixpkgs](https://github.com/NixOS/nixpkgs)
2. Add `pkgs/by-name/su/surge-tui/package.nix`
3. Calculate sha256:
   ```bash
   nix-prefetch-url --unpack https://github.com/YOUR_USERNAME/surge-tui/archive/v0.1.0.tar.gz
   ```
4. Update `nix/default.nix` with the sha256
5. Test build:
   ```bash
   nix-build -A surge-tui
   ```
6. Submit PR following [nixpkgs contributing guide](https://github.com/NixOS/nixpkgs/blob/master/CONTRIBUTING.md)

## Updating Homebrew Formula

After each release:

```bash
# 1. Get new SHA256
shasum -a 256 dist/surge-tui-v0.1.0-macos-en.tar.gz

# 2. Update homebrew/surge-tui.rb
# - Update version
# - Update url
# - Update sha256

# 3. Test formula
brew install --build-from-source ./homebrew/surge-tui.rb
brew test surge-tui
brew audit --strict surge-tui

# 4. Commit and push
git add homebrew/surge-tui.rb
git commit -m "Update surge-tui to 0.1.0"
git push
```

## Updating Nix Package

After each release:

```bash
# 1. Calculate new sha256
nix-prefetch-url --unpack https://github.com/YOUR_USERNAME/surge-tui/archive/v0.1.0.tar.gz

# 2. Update nix/default.nix or flake.nix
# - Update version
# - Update rev
# - Update sha256 (for default.nix)
# - cargoHash will be calculated automatically

# 3. Test build
nix build

# 4. Commit and push
git add nix/default.nix flake.nix
git commit -m "Update to 0.1.0"
git push
```

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

Examples:
- `0.1.0` - Initial release
- `0.1.1` - Bug fix
- `0.2.0` - New feature (search)
- `1.0.0` - First stable release

## Checklist

Before releasing:

- [ ] All tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created
- [ ] GitHub Release created
- [ ] Homebrew formula updated
- [ ] Nix package updated
- [ ] Release announcement (optional)

## Troubleshooting

### Homebrew formula audit fails

```bash
# Check specific warnings
brew audit --strict --online surge-tui

# Fix common issues:
# - Update dependencies
# - Fix license detection
# - Add test block
```

### Nix build fails

```bash
# Check build log
nix build --show-trace

# Common issues:
# - Wrong cargoHash (let it auto-calculate)
# - Missing dependencies
# - Platform mismatch
```

### GitHub Actions fails

- Check `.github/workflows/release.yml`
- Verify secrets are set (if needed)
- Check runner compatibility (macOS version)

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Nix Package Guide](https://nixos.org/manual/nixpkgs/stable/#chap-quick-start)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Semantic Versioning](https://semver.org/)
