# Releasing Clash

This document describes how to release a new version of Clash.

## Prerequisites

1. Ensure all tests pass: `cargo test`
2. Update version in `Cargo.toml`
3. Update `CHANGELOG.md` with release notes
4. Commit all changes
5. Set up crates.io token (one-time):
   - Get token from https://crates.io/settings/tokens
   - Add as `CARGO_REGISTRY_TOKEN` secret in GitHub repo settings

## Creating a Release

1. **Tag the release:**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **GitHub Actions automatically:**
   - Creates a GitHub Release
   - Builds binaries for all platforms:
     - macOS (Intel & ARM)
     - Linux (x64 & ARM)
     - Windows (x64)
   - Uploads binaries to the release
   - Publishes to crates.io
   - Updates Homebrew formula (if tap repo exists)

3. **Manual steps after release:**
   - Verify binaries are uploaded to GitHub Release
   - Test installation methods:
     ```bash
     # Test curl installation
     curl -fsSL https://clash.sh/install.sh | sh

     # Test Homebrew (if tap is set up)
     brew update
     brew install clash-sh/tap/clash

     # Test crates.io (wait ~5 minutes for indexing)
     cargo install clash-sh
     ```

## Setting up Homebrew Tap (One-time)

1. Create new repository: `github.com/clash-sh/homebrew-tap`

2. Create `Formula/` directory in the repo

3. Copy `homebrew/clash.rb` to `Formula/clash.rb`

4. Create a GitHub token with repo access for the tap repository

5. Add the token as `TAP_GITHUB_TOKEN` secret in main Clash repository

## Installation Methods

After release, users can install via:

### curl/wget
```bash
curl -fsSL https://raw.githubusercontent.com/clash-sh/clash/main/install.sh | sh
```

### Homebrew
```bash
brew install clash-sh/tap/clash
```

### Direct download
```bash
# macOS ARM
curl -LO https://github.com/clash-sh/clash/releases/latest/download/clash-aarch64-apple-darwin.tar.gz
tar -xzf clash-aarch64-apple-darwin.tar.gz
sudo mv clash /usr/local/bin/

# Linux x64
curl -LO https://github.com/clash-sh/clash/releases/latest/download/clash-x86_64-unknown-linux-gnu.tar.gz
tar -xzf clash-x86_64-unknown-linux-gnu.tar.gz
sudo mv clash /usr/local/bin/
```

### From source
```bash
cargo install --git https://github.com/clash-sh/clash
```

## Versioning

We follow semantic versioning:
- `v0.1.0` - Initial release
- `v0.1.1` - Bug fixes
- `v0.2.0` - New features
- `v1.0.0` - Stable API

## Troubleshooting

### Release workflow fails

1. Check GitHub Actions logs
2. Ensure all target architectures build locally:
   ```bash
   cargo build --target x86_64-apple-darwin
   cargo build --target aarch64-apple-darwin
   ```

### Homebrew formula not updating

1. Check that `TAP_GITHUB_TOKEN` secret is set
2. Verify token has write access to tap repository
3. Check logs in the `update-homebrew` job

### Binary not found for platform

The install script falls back to source installation if binaries aren't available. This requires Rust to be installed.