# Release Process Guide

This document describes the automated and manual release process for DroidKraft.

## Quick Release (Automated)

### Using the Just Command Runner

Install `just` if you haven't already:
```bash
cargo install just
```

Then bump the version and release:
```bash
just release 0.2.5
```

This will:
1. Update version in `Cargo.toml`
2. Update version badge in `README.md`
3. Update `Cargo.lock`
4. Generate `CHANGELOG.md` with git-cliff
5. Run formatting checks
6. Run clippy linter
7. Run tests
8. Create a git commit
9. Create a git tag
10. Push to GitHub

### Using the Bash Script

```bash
./scripts/bump_version.sh 0.2.5
```

Then push:
```bash
git push origin main
git push origin v0.2.5
```

## Manual Release Process

### 1. Update Version Numbers

Update version in the following files:

**Cargo.toml:**
```toml
[package]
version = "0.2.5"
```

**README.md:**
```markdown
![Version](https://img.shields.io/badge/version-0.2.5-blue.svg)
```

**CHANGELOG.md:**
Generate with git-cliff:
```bash
git-cliff --tag v0.2.5 -o CHANGELOG.md
```

### 2. Run Quality Checks

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

### 3. Update Documentation

Review and update if needed:
- `README.md` - Features, installation instructions
- `CHANGELOG.md` - Add release notes (if you maintain one)
- `Cargo.toml` - Description, keywords

### 4. Commit and Tag

```bash
# Stage changes
git add Cargo.toml Cargo.lock README.md

# Commit
git commit -m "chore: bump version to 0.2.5"

# Create annotated tag
git tag -a v0.2.5 -m "Release v0.2.5

New Features:
- Feature 1
- Feature 2

Bug Fixes:
- Fix 1
- Fix 2

Breaking Changes:
- Change 1 (if any)
"

# Push to GitHub
git push origin main
git push origin v0.2.5
```

### 5. GitHub Release (Automated)

Once you push a tag, GitHub Actions will automatically:
1. Run CI tests
2. Build release binaries
3. Generate and commit CHANGELOG.md
4. Update README version badge
5. Create a GitHub Release (with changelog)
6. Publish to crates.io (if `CRATES_IO_TOKEN` secret is set)

## Version Numbering

DroidKraft follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0) - Incompatible API changes
- **MINOR** (0.X.0) - New features, backwards compatible
- **PATCH** (0.0.X) - Bug fixes, backwards compatible

### Examples:

- `0.2.4` → `0.2.5` - Bug fixes or small improvements
- `0.2.5` → `0.3.0` - New features
- `0.3.0` → `1.0.0` - Major release or breaking changes

## Just Commands Reference

```bash
# Show all available commands
just

# Check current version
just version

# Run all checks before release
just release-check

# Format code
just fmt

# Run linter
just clippy

# Run tests
just test

# Build release binary
just build-release

# Generate changelog
just changelog

# Generate changelog for unreleased
just changelog-unreleased

# Generate changelog for specific version
just changelog-version 0.2.5

# Bump version (updates all files)
just bump 0.2.5

# Full release workflow
just release 0.2.5

# View changelog
just view-changelog
```

## GitHub Actions Workflows

### CI Workflow (`.github/workflows/ci.yml`)
- Triggers on: Push to main, Pull requests
- Checks: Format, Clippy, Tests
- Platforms: Linux, Windows

### Release Workflow (`.github/workflows/release.yml`)
- Triggers on: Tags matching `v*.*.*`
- Actions:
  - Build release binaries
  - Run tests
  - Generate CHANGELOG.md with git-cliff
  - Create GitHub Release (with changelog notes)
  - Publish to crates.io

### Update README and CHANGELOG Workflow (`.github/workflows/update-readme.yml`)
- Triggers on: Tags matching `v*.*.*`
- Actions:
  - Extract version from tag
  - Update README.md version badge
  - Generate CHANGELOG.md with git-cliff
  - Commit and push changes

## Git-Cliff (Changelog Generator)

### Installation

```bash
cargo install git-cliff
```

### Usage

```bash
# Generate full changelog
git-cliff -o CHANGELOG.md

# Generate changelog for specific version
git-cliff --tag v0.2.5 -o CHANGELOG.md

# Generate unreleased changes
git-cliff --unreleased -o CHANGELOG.md

# View changes between tags
git-cliff v0.2.3..v0.2.4
```

### Configuration

The changelog format is configured in `cliff.toml` and follows conventional commits:
- `feat:` → 🚀 Features
- `fix:` → 🐛 Bug Fixes
- `docs:` → 📚 Documentation
- `refactor:` → 🚜 Refactor
- `perf:` → ⚡ Performance
- `style:` → 🎨 Styling
- `test:` → 🧪 Testing
- `chore:` → ⚙️ Miscellaneous Tasks

## Publishing to crates.io

### First Time Setup

1. Create an account on [crates.io](https://crates.io/)
2. Get your API token from [crates.io/settings/tokens](https://crates.io/settings/tokens)
3. Add it to GitHub secrets as `CRATES_IO_TOKEN`

### Manual Publish

```bash
# Dry run (check for issues)
cargo publish --dry-run

# Actually publish
cargo publish
```

### Automated Publish

Once `CRATES_IO_TOKEN` is set in GitHub secrets, the release workflow will automatically publish to crates.io when you push a tag.

## Rollback Process

If something goes wrong:

```bash
# Delete local tag
git tag -d v0.2.5

# Delete remote tag
git push origin :refs/tags/v0.2.5

# Revert commit
git revert HEAD
git push origin main
```

**Note:** You cannot unpublish a crate from crates.io, but you can yank a version:
```bash
cargo yank --vers 0.2.5
```

## Pre-release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] README is up to date
- [ ] Version numbers are updated
- [ ] CHANGELOG.md is generated
- [ ] Commits follow conventional commits format
- [ ] No uncommitted changes
- [ ] On main branch

## Post-release Checklist

- [ ] Tag is pushed to GitHub
- [ ] GitHub Actions workflows completed successfully
- [ ] CHANGELOG.md is generated and committed
- [ ] README version badge is updated
- [ ] GitHub Release is created (with changelog)
- [ ] Published on crates.io
- [ ] Announcement on social media (optional)

## Troubleshooting

### git-cliff not found

Install git-cliff:
```bash
cargo install git-cliff
```

### GitHub Action fails to update README/CHANGELOG

Check that the workflow has `contents: write` permission in the workflow file.

### Cannot publish to crates.io

1. Verify `CRATES_IO_TOKEN` secret is set
2. Check that the version doesn't already exist
3. Ensure all required fields in `Cargo.toml` are filled

### Tag already exists

```bash
# Delete and recreate
git tag -d v0.2.5
git push origin :refs/tags/v0.2.5
git tag -a v0.2.5 -m "Release v0.2.5"
git push origin v0.2.5
```

## Need Help?

- Check GitHub Actions logs for detailed error messages
- Review [Cargo documentation](https://doc.rust-lang.org/cargo/)
- See [GitHub Actions documentation](https://docs.github.com/en/actions)

---

**Happy Releasing! 🚀**