# Release Process

This document describes how to create releases for ztlgr.

## Prerequisites

Before creating a release, ensure:

1. All tests pass: `cargo test --lib --all-features`
2. Code is formatted: `cargo fmt --all -- --check`
3. No clippy warnings: `cargo clippy --all-features -- -D warnings`
4. Documentation builds: `cargo doc --no-deps`
5. CHANGELOG.md is updated

## Release Checklist

### 1. Version Bump

Update the version in `Cargo.toml`:

```toml
version = "0.2.0"  # Update this
```

### 2. Update CHANGELOG

Add release notes to `CHANGELOG.md`:

```markdown
## [0.2.0] - 2025-04-03

### Added
- New feature description

### Changed
- Change description

### Fixed
- Bug fix description
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
```

### 4. Create Git Tag

```bash
# For stable releases
git tag -a v0.2.0 -m "Release 0.2.0"

# For pre-releases (alpha/beta/rc)
git tag -a v0.2.0-beta.1 -m "Release 0.2.0-beta.1"
```

### 5. Push Tag

```bash
git push origin v0.2.0
```

### 6. Automatic Release Process

Once the tag is pushed, GitHub Actions will:

1. **Create a GitHub Release** with auto-generated release notes
2. **Build binaries** for:
   - Linux x86_64
   - Linux ARM64
   - macOS x86_64 (Intel)
   - macOS ARM64 (Apple Silicon)
   - Windows x86_64
3. **Upload binaries** to the GitHub release
4. **Generate checksums** for verification
5. **Publish to crates.io** (stable releases only)

The release will appear at: `https://github.com/bakudas/ztlgr/releases/tag/v0.2.0`

## Release Assets

Each release includes:

| Platform | Architecture | File Name | Format |
|----------|--------------|-----------|--------|
| Linux | x86_64 | `ztlgr-VERSION-linux-x86_64.tar.gz` | tar.gz |
| Linux | ARM64 | `ztlgr-VERSION-linux-aarch64.tar.gz` | tar.gz |
| macOS | x86_64 | `ztlgr-VERSION-macos-x86_64.tar.gz` | tar.gz |
| macOS | ARM64 | `ztlgr-VERSION-macos-aarch64.tar.gz` | tar.gz |
| Windows | x86_64 | `ztlgr-VERSION-windows-x86_64.zip` | zip |

Plus corresponding `.sha256` checksum files.

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): Backwards-compatible features
- **PATCH** version (0.0.X): Backwards-compatible bug fixes

Pre-release versions:
- `v1.0.0-alpha.1` - Alpha release (internal testing)
- `v1.0.0-beta.1` - Beta release (public testing)
- `v1.0.0-rc.1` - Release candidate

## Troubleshooting

### Build Fails

1. Check CI logs in GitHub Actions
2. Fix the issue locally
3. Delete the tag: `git push --delete origin v0.2.0`
4. Delete local tag: `git tag -d v0.2.0`
5. Fix and commit
6. Re-create tag and push

### crates.io Publish Fails

1. Check if version already exists on crates.io
2. Verify `Cargo.toml` metadata is correct
3. Ensure `CRATES_IO_TOKEN` secret is set in GitHub repository
4. Manually publish: `cargo publish --token <token>`

### Missing Assets

1. Wait for all build jobs to complete
2. Check individual build job logs
3. Re-run failed jobs from GitHub Actions UI

## Manual Release (Fallback)

If GitHub Actions fails, you can create a release manually:

```bash
# Build locally
cargo build --release

# Package binary
cd target/release
tar -czf ztlgr-$(git describe --tags)-linux-x86_64.tar.gz ztlgr

# Create GitHub release manually
# Upload binary and checksum
```

## Post-Release Tasks

After successful release:

1. **Verify release** on GitHub
2. **Test installation** from release binaries
3. **Update Homebrew formula** (after adding Homebrew support)
4. **Update AUR package** (after adding AUR support)
5. **Announce release** on:
   - GitHub Discussions
   - Reddit (r/rust, r/commandline)
   - Twitter/Mastodon
   - Discord servers (Rust Community, etc.)

## CI/CD Workflows

### CI Workflow (`ci.yml`)

Runs on every push and pull request:
- **Test matrix**: Ubuntu, macOS, Windows + Stable/Beta Rust
- **Format check**: `cargo fmt --check`
- **Clippy**: `cargo clippy -- -D warnings`
- **Documentation**: `cargo doc`
- **Security audit**: `cargo audit`

### Release Workflow (`release.yml`)

Runs on tag push (`v*.*.*`):
- **Create release**: Auto-generates release notes
- **Build**: Cross-compile for all platforms
- **Package**: Create archives with checksums
- **Upload**: Attach binaries to GitHub release
- **Publish**: Upload to crates.io (stable only)

## Secrets Required

Configure these secrets in repository settings:

- `GITHUB_TOKEN`: Automatically provided by GitHub
- `CRATES_IO_TOKEN`: Get from https://crates.io/me

To add `CRATES_IO_TOKEN`:
1. Log into crates.io
2. Generate token: https://crates.io/me
3. Go to repo → Settings → Secrets and variables → Actions
4. Add new secret: Name = `CRATES_IO_TOKEN`, Value = your token

## Monitoring

After release, monitor:

1. **GitHub Issues**: Watch for bug reports
2. **crates.io**: Check download stats
3. **GitHub Actions**: Verify all workflow runs succeed
4. **User Feedback**: Check discussions, Discord, etc.

## Rollback

If critical bugs are found:

1. **Yank from crates.io**: `cargo yank --vers 0.2.0`
2. **Mark release as pre-release** on GitHub
3. **Delete release** if necessary
4. **Fix bugs** and release new version
5. **Git tag new version**: `v0.2.1`