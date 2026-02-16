# Release Guide

This document describes how to build and publish surge-tui to Homebrew and Nix.

## Build Process

### Local Build

```bash
# English version (default)
cargo build --release
./target/release/surge-tui

# Chinese version
cargo build --release --no-default-features --features lang-zh-cn
./target/release/surge-tui
```

### Release Build

Use the release script:

```bash
./scripts/release.sh 0.1.0
```

This will:
1. Update version in `Cargo.toml`
2. Build both English and Chinese binaries
3. Run tests
4. Create distribution archives in `dist/`
5. Generate SHA256 checksums
6. Create git tag

## Publishing to Homebrew

### Option 1: Personal Tap (Recommended for initial releases)

1. **Create a Homebrew tap repository**
   ```bash
   # On GitHub, create: homebrew-tap (or homebrew-surge)
   # Clone locally
   git clone https://github.com/YOUR_USERNAME/homebrew-tap.git
   cd homebrew-tap
   mkdir -p Formula
   ```

2. **Copy and update formula**
   ```bash
   cp /path/to/surge-tui/homebrew/surge-tui.rb Formula/
   ```

3. **Update formula with actual values**
   - Replace `YOUR_USERNAME` with your GitHub username
   - Update `url` to point to GitHub release tarball
   - Calculate and update `sha256`:
     ```bash
     # After creating GitHub release
     curl -L https://github.com/YOUR_USERNAME/surge-tui/archive/v0.1.0.tar.gz | shasum -a 256
     ```

4. **Test locally**
   ```bash
   brew install --build-from-source ./Formula/surge-tui.rb
   surge-tui --version
   ```

5. **Publish**
   ```bash
   git add Formula/surge-tui.rb
   git commit -m "Add surge-tui v0.1.0"
   git push
   ```

6. **Users can install with**
   ```bash
   brew tap YOUR_USERNAME/tap
   brew install surge-tui
   ```

### Option 2: Official Homebrew Core

For wider distribution:

1. Fork [homebrew-core](https://github.com/Homebrew/homebrew-core)
2. Add `Formula/surge-tui.rb`
3. Test thoroughly
4. Submit PR with description

Requirements:
- Stable version (not pre-release)
- Good documentation
- Passes all brew audit checks

## Publishing to Nix

### Option 1: Personal Overlay (Quick start)

1. **Create flake.nix in surge-tui repo**
   ```nix
   {
     description = "Surge TUI - Terminal interface for macOS Surge";

     inputs = {
       nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
       flake-utils.url = "github:numtide/flake-utils";
     };

     outputs = { self, nixpkgs, flake-utils }:
       flake-utils.lib.eachDefaultSystem (system:
         let pkgs = nixpkgs.legacyPackages.${system};
         in {
           packages.default = pkgs.callPackage ./nix/default.nix { };
         });
   }
   ```

2. **Update nix/default.nix**
   - Replace `YOUR_USERNAME`
   - Calculate `sha256`:
     ```bash
     nix-prefetch-url --unpack https://github.com/YOUR_USERNAME/surge-tui/archive/v0.1.0.tar.gz
     ```
   - Calculate `cargoHash`:
     ```bash
     # Build once, it will fail and show correct hash
     nix build
     # Copy the hash from error message
     ```

3. **Test locally**
   ```bash
   nix build
   ./result/bin/surge-tui --version
   ```

4. **Users can install with**
   ```bash
   nix profile install github:YOUR_USERNAME/surge-tui
   ```

### Option 2: Submit to nixpkgs

For official Nix packages:

1. Fork [nixpkgs](https://github.com/NixOS/nixpkgs)
2. Add package to `pkgs/by-name/su/surge-tui/package.nix`
3. Test with `nix-build -A surge-tui`
4. Submit PR

## GitHub Release Checklist

After running `./scripts/release.sh`:

- [ ] Push commits and tags
  ```bash
  git push origin main
  git push origin v0.1.0
  ```

- [ ] Create GitHub Release
  - Go to https://github.com/YOUR_USERNAME/surge-tui/releases/new
  - Tag version: `v0.1.0`
  - Release title: `v0.1.0`
  - Description: Copy from CHANGELOG
  - Attach files from `dist/`:
    - `surge-tui-v0.1.0-macos-en.tar.gz`
    - `surge-tui-v0.1.0-macos-zh.tar.gz`
    - `surge-tui-v0.1.0-checksums.txt`

- [ ] Update Homebrew formula with release URL and SHA256

- [ ] Update Nix package with release info

- [ ] Test installation
  ```bash
  # Homebrew
  brew tap YOUR_USERNAME/tap
  brew install surge-tui

  # Nix
  nix profile install github:YOUR_USERNAME/surge-tui
  ```

## Version Scheme

Follow [Semantic Versioning](https://semver.org/):

- `0.1.0` - Initial release
- `0.1.x` - Bug fixes
- `0.x.0` - New features (backwards compatible)
- `x.0.0` - Breaking changes

## Pre-release Checklist

Before running release script:

- [ ] Update CHANGELOG.md
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt`
- [ ] Documentation updated
- [ ] README accurate

## Distribution Binaries

We distribute two variants:

1. **surge-tui** (English) - Default, `lang-en-us` feature
2. **surge-tui-zh** (Chinese) - Chinese version, `lang-zh-cn` feature

Users choose their preferred language at install time.

## Troubleshooting

### Homebrew formula fails audit

```bash
brew audit --strict surge-tui
brew test surge-tui
```

Common issues:
- Missing `test do` block
- Incorrect license
- Dependencies not declared

### Nix build fails

```bash
# Check derivation
nix show-derivation

# Build with verbose output
nix build --print-build-logs

# Common issues:
# - Wrong cargoHash (build once to get correct hash)
# - Missing dependencies
# - Incorrect src hash
```

### Release script fails

```bash
# Run steps manually:
cargo build --release
cargo test
git tag v0.1.0
```
