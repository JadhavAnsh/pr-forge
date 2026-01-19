# GitHub Actions Workflows

This directory contains automated CI/CD workflows for PR-Forge.

## Workflows

### 1. **Cross-Platform Build** (`build.yml`)
**Triggers**: Push to `main` branch, pull requests, manual dispatch

**What it does**:
- Builds for all platforms: Linux, macOS (Intel & ARM64), Windows
- Runs cargo check and tests
- Creates optimized release binaries
- Uploads artifacts for download
- Automatically creates releases with tagged binaries
- Runs full test suite and linting

**Builds**:
- ✅ Linux (x86_64)
- ✅ macOS (Intel - x86_64)
- ✅ macOS (ARM64)
- ✅ Windows (MSVC)

**Release Assets Generated**:
- `pr-forge-linux-x86_64.tar.gz`
- `pr-forge-macos-x86_64.tar.gz`
- `pr-forge-macos-arm64.tar.gz`
- `pr-forge-windows-x86_64.zip`

**Matrix Strategy**:
```yaml
- Runs in parallel on GitHub-hosted runners
- Each job takes ~10-15 minutes
- Uses cargo cache for faster builds
- Strips binaries on Linux/macOS for smaller artifacts
```

### 2. **Pull Request Checks** (`pr-checks.yml`)
**Triggers**: Pull requests to `main`/`develop`, push to `develop`

**What it does**:
- Runs `cargo check` for compilation
- Executes full test suite
- Validates code formatting (rustfmt)
- Runs clippy linter
- Performs security audit

**Jobs**:
- ✅ Cargo Check
- ✅ Unit Tests
- ✅ Format Check
- ✅ Clippy Linting
- ✅ Security Audit

**Prevents Merge If**:
- Code doesn't compile
- Tests fail
- Format violations detected
- Clippy warnings (treated as errors)
- Known vulnerabilities found

### 3. **Nightly Build** (`nightly.yml`)
**Triggers**: Daily at 2 AM UTC, manual dispatch

**What it does**:
- Tests with Rust nightly compiler
- Security vulnerability scanning
- Dependency update checking
- Creates issues for vulnerabilities

**Jobs**:
- ✅ Nightly Build (all platforms)
- ✅ Security Audit
- ✅ Dependency Updates

**Features**:
- Continues on error (doesn't block releases)
- Auto-creates issues for security problems
- Tracks outdated dependencies
- Checks for unused dependencies

---

## Usage

### Running Workflows Manually
In GitHub Actions tab → Select workflow → "Run workflow" button

```bash
# Or via CLI with GitHub CLI
gh workflow run build.yml
```

### Viewing Build Status
1. Go to Actions tab on GitHub
2. Select workflow (Build, PR Checks, or Nightly)
3. View build logs and artifacts

### Downloading Artifacts
After successful build:
1. Actions → Build Job → Artifacts
2. Download `pr-forge-{os}` artifacts
3. Extract and use the binary

---

## Build Times

**Typical Build Times**:
- Linux: ~8-10 minutes
- macOS (Intel): ~12-15 minutes
- macOS (ARM64): ~12-15 minutes
- Windows: ~10-12 minutes

**Total Time**: ~15-20 minutes (parallel execution)

**Cache Benefits**:
- First build: ~20 minutes
- Subsequent builds: ~8-10 minutes (with cargo cache hits)

---

## Environment Variables

**Available in all workflows**:
- `CARGO_TERM_COLOR`: `always` (colored output)
- `RUST_BACKTRACE`: `1` (detailed error info)

**GitHub Secrets** (if needed):
```yaml
# Add via Settings → Secrets → New repository secret
GROQ_API_KEY: (for API testing)
```

---

## Cache Strategy

**Cached Items**:
- `~/.cargo/registry` - Downloaded crates
- `~/.cargo/git` - Git dependencies
- `target/` - Compiled dependencies

**Cache Key**:
- `Cargo.lock` hash - Invalidates on dependency changes
- Runner OS - Separate caches per platform

**Storage**:
- Up to 5 GB per repository
- 7-day retention
- Shared across branches

---

## Release Process

### Automatic Release Creation
When pushing to `main`:
1. Build triggers on all platforms
2. Tests and linting run in parallel
3. Release job creates GitHub Release
4. Binaries uploaded as release assets
5. Release notes auto-generated from commit message

### Manual Release
```bash
git push origin main
# GitHub Actions automatically creates release
# No manual steps needed
```

### Release Artifacts
Each release includes:
- Pre-built binaries for all platforms
- Auto-generated release notes
- Build number as version tag

---

## Troubleshooting

### Build Fails on Windows
**Issue**: Linker error (exit code 143)
**Solution**: Windows runners have more resources; clean rebuild

### Tests Fail Locally but Pass in CI
**Likely Causes**:
- Environment differences
- Missing system dependencies
- Cargo cache issues

**Solution**:
```bash
cargo clean
cargo test
```

### PR Blocked by Clippy Warnings
**Solution**: Run locally and fix
```bash
cargo clippy --fix
cargo fmt
```

### Artifacts Not Generated
**Check**:
1. Build status (not failed)
2. Cargo output for errors
3. Disk space in runner

---

## Performance Optimization

### Speed Up Builds
1. **Use cache** - Automatic, no action needed
2. **Reduce jobs** - Modify matrix for fewer platforms
3. **Parallel tests** - Already configured

### Reduce Cost
- GitHub Actions free tier: 2000 minutes/month
- Current usage: ~50 minutes per full build
- Sufficient for daily + PR builds

### Monitor Usage
Settings → Security & analysis → Actions → Usage

---

## Security Considerations

### API Keys & Secrets
- ✅ Never commit `.env` files
- ✅ Use GitHub Secrets for sensitive data
- ✅ Workflows have automatic redaction

### Dependency Scanning
- ✅ Nightly security audits
- ✅ Auto-created issues for vulnerabilities
- ✅ Clippy checks for unsafe code

### Build Integrity
- ✅ Signed by GitHub
- ✅ Runner verification
- ✅ Artifact checksums

---

## Customization

### Add New Platform
Edit `build.yml` matrix:
```yaml
- os: Your Platform
  runs-on: your-runner
  target: your-target
  artifact: binary-name
```

### Change Build Triggers
Edit `on:` section in workflow files:
```yaml
on:
  push:
    branches: [main, develop]  # Add more branches
  schedule:
    - cron: '0 * * * *'       # Different schedule
```

### Add New Jobs
Copy job template and customize:
```yaml
new-job:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    # Add steps here
```

---

## Monitoring

### Workflow Status Badge
Add to README.md:
```markdown
![Build Status](https://github.com/YOUR/REPO/workflows/Cross-Platform%20Build/badge.svg)
```

### Subscribe to Notifications
1. Go to Actions page
2. Click bell icon
3. Select notification preferences

### View Historical Data
1. Actions → Workflows
2. Click workflow
3. View run history and statistics

---

## Common Patterns

### Run Specific Job
```bash
# Use workflow dispatch to trigger specific workflow
gh workflow run build.yml --ref main
```

### Force Rebuild (Clear Cache)
```bash
# Delete cache in Settings → Actions → Caches
# Then push to trigger rebuild
```

### Skip Workflow
Add to commit message:
```
[skip ci]
[skip github]
```

---

## Next Steps

### To Use These Workflows
1. ✅ Already in `.github/workflows/` directory
2. Push to GitHub repository
3. Go to Actions tab
4. Enable workflows if needed
5. First build starts automatically

### To Download Binaries
1. Go to Actions tab
2. Select "Cross-Platform Build"
3. Click latest successful run
4. Download artifacts

### To Test Locally First
```bash
# Before pushing
cargo check
cargo test
cargo clippy
cargo fmt
```

---

## Support

**Workflow Issues**:
1. Check Actions log for error
2. Search GitHub Actions docs
3. Review similar workflows in other repos

**Build Failures**:
- Check platform-specific setup
- Review dependencies for platform
- Test locally first

---

**Last Updated**: January 19, 2026
**Status**: ✅ Ready for use
