# PR-Forge GitHub Actions Workflows

This directory contains automated CI/CD workflows for PR-Forge. All workflows are designed to maintain code quality, security, and cross-platform compatibility.

## 📋 Workflow Summary

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| [build.yml](#build-workflow) | Push to main, Manual | Cross-platform builds (Windows, macOS, Linux) |
| [pr-checks.yml](#pr-checks-workflow) | Pull Request to main | Quality gates for PRs |
| [nightly.yml](#nightly-workflow) | Daily 2 AM UTC, Manual | Nightly builds and security audits |
| [dependencies.yml](#dependencies-workflow) | Weekly Monday, Manual | Dependency updates and security checks |
| [performance.yml](#performance-workflow) | Code changes, Manual | Compile time and binary size analysis |

---

## Build Workflow

**File**: `build.yml`  
**Trigger**: Push to main branch, Manual trigger (`workflow_dispatch`)  
**Status Badge**: [![Build Status](../../workflows/build/badge.svg)](../../actions/workflows/build.yml)

### Purpose
Creates optimized release builds for Windows, macOS, and Linux, with automatic release creation.

### Platforms
- **Linux**: x86_64 (GNU toolchain)
- **macOS**: Intel x86_64 + Apple Silicon (ARM64)
- **Windows**: x86_64 (MSVC)

### Outputs
- Compiled binaries in GitHub Releases
- Build artifacts available for download
- Automated semantic versioning

### Job Breakdown

#### 1. Check Job
- Runs `cargo check` across all platforms
- Validates compilation without building
- **Duration**: ~2-3 minutes

#### 2. Test Job
- Executes all unit tests
- Runs on ubuntu-latest only (platform-independent tests)
- **Duration**: ~1-2 minutes

#### 3. Format Check
- Validates Rust code formatting
- Uses `cargo fmt` with `--check` flag
- Prevents style inconsistencies

#### 4. Lint Job
- Runs clippy linter
- Detects common Rust mistakes
- Enforces best practices

#### 5. Build Matrix
Parallel builds across platforms:

```
┌─────────────────────────────────────────┐
│          Platform Matrix                │
├──────────┬──────────┬──────────┬────────┤
│ Linux    │ macOS    │ macOS    │Windows │
│ x86_64   │ x86_64   │ ARM64    │ x86_64 │
│ GNU      │ Clang    │ Clang    │ MSVC   │
└──────────┴──────────┴──────────┴────────┘
```

Each platform:
1. Installs dependencies (Ubuntu only)
2. Builds release binary: `cargo build --release`
3. Runs tests: `cargo test --release`
4. Creates archive (.tar.gz or .zip)
5. Uploads artifact

#### 6. Release Job
- Creates GitHub Release with semantic versioning
- Attaches platform-specific binaries
- Tags commit with version
- Only runs on main branch after successful build

### Customization

**Change build matrix**:
```yaml
strategy:
  matrix:
    platform:
      - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
      # Add more platforms as needed
```

**Modify release behavior**:
```yaml
- name: Create Release
  if: github.ref == 'refs/heads/main' && success()
  # Adjust if: condition for different branches
```

**Set custom version**:
```yaml
env:
  CARGO_VERSION: 1.0.0  # Override in workflow or secrets
```

### Performance Tips
- Caching is automatic via actions/cache
- First run slower, subsequent runs leverage cache
- Linux builds faster than Windows/macOS
- Parallel jobs reduce total time

---

## PR Checks Workflow

**File**: `pr-checks.yml`  
**Trigger**: Pull Request to main/develop  
**Status**: Required check for merge

### Purpose
Validates code quality before merge. All checks must pass.

### Jobs

#### Security Check
- Runs `cargo audit` for known vulnerabilities
- Fails if vulnerabilities found
- **Duration**: ~30 seconds

#### Tests
- Full test suite: `cargo test`
- Code coverage included
- **Duration**: ~2-3 minutes

#### Formatting
- Enforces Rust style guide
- `cargo fmt --check`
- No auto-fix (manual fix required)
- **Duration**: ~30 seconds

#### Clippy Linting
- Static analysis for code improvements
- Blocks on warnings (strict mode)
- **Duration**: ~1-2 minutes

#### Documentation
- Checks doc comments
- Validates README
- **Duration**: ~30 seconds

### Merge Requirements
All jobs must pass:
- ✅ Security check
- ✅ Tests (100% pass rate)
- ✅ Format check
- ✅ Clippy linting
- ✅ Docs validation

### Review Process
1. Push to feature branch
2. Create PR against main/develop
3. GitHub Actions automatically runs
4. Fix any failures
5. Re-push changes
6. Actions re-runs automatically
7. Once all pass, ready for review

---

## Nightly Workflow

**File**: `nightly.yml`  
**Trigger**: Daily at 2 AM UTC, Manual trigger  
**Status**: Advisory (non-blocking)

### Purpose
Extended testing with nightly Rust, security audits, and dependency checks.

### Jobs

#### Nightly Build
Tests with unstable Rust features:
- Uses `rust-toolchain: nightly`
- Catches future compatibility issues
- May have warnings (non-blocking)
- **Duration**: ~3-5 minutes

#### Security Audit
- Full RustSec database check
- Scans all transitive dependencies
- Creates GitHub Issue if vulnerabilities found
- **Duration**: ~1-2 minutes

#### Outdated Dependencies Check
- Identifies newer versions available
- Creates Issue with upgrade suggestions
- Helps keep stack current
- **Duration**: ~1 minute

#### Unused Dependencies
- Checks for unused crates in Cargo.toml
- Suggests removal of bloat
- **Duration**: ~30 seconds

### Issue Creation
When issues detected:
- Label: `dependencies`, `security`, `maintenance`
- Includes actionable details
- Links to advisory database
- Example title: "📦 Dependency Updates Available"

### Manual Trigger
Run immediately without waiting:
```bash
gh workflow run nightly.yml
```

---

## Dependencies Workflow

**File**: `dependencies.yml`  
**Trigger**: Weekly (Monday 9 AM UTC), Manual, Push to Cargo.toml  
**Status**: Advisory

### Purpose
Proactive dependency management and security scanning.

### Jobs

#### 1. Check Updates
- Lists outdated packages
- Creates Issues for available updates
- Shows exact version bumps needed
- Example output:
  ```
  reqwest                    0.11.0 -> 0.12.0
  tokio                      1.35.0 -> 1.36.0
  ```

#### 2. Security Check
- Runs RustSec audit
- Creates critical Issues for vulnerabilities
- Prevents security regressions
- Auto-labels as `security`, `critical`

#### 3. Verify Lock File
- Ensures Cargo.lock is up-to-date
- Prevents accidental version drift
- Fails if lock file stale
- Run locally: `cargo update --dry-run`

#### 4. License Check
- Audits all dependency licenses
- Generates license report
- Checks for incompatible licenses
- Outputs: MIT, Apache-2.0, GPL, etc.
- Artifact: `license-report` (30-day retention)

#### 5. Duplicate Check
- Identifies duplicate dependencies
- Analyzes full dependency tree
- Suggests consolidations
- Artifact: `dependency-tree` (30-day retention)

### Issue Labels
- `dependencies`: General updates
- `security`: Security vulnerabilities
- `maintenance`: Maintenance work
- `critical`: Requires immediate attention

### Local Testing
```bash
# Check outdated locally
cargo outdated

# Run security audit
cargo audit

# Analyze tree
cargo tree --duplicates
```

---

## Performance Workflow

**File**: `performance.yml`  
**Trigger**: Code changes in src/*, Manual  
**Status**: Informational

### Purpose
Monitor build performance and binary size across changes.

### Jobs

#### Benchmark
- Runs Rust benchmarks (if available)
- Generates benchmark report
- Compares performance metrics
- **Duration**: ~5-10 minutes
- **Artifact**: `benchmark-results` (90-day retention)

#### Compile Time
- Measures debug build time
- Measures release build time
- Tracks resource usage (memory, CPU)
- Shows elapsed time and peak memory
- **Duration**: ~3-5 minutes
- **Artifact**: `compile-times` (90-day retention)

#### Binary Size Analysis
- Reports release binary size (e.g., 15MB)
- Shows section breakdown (.text, .data, etc.)
- Tracks dependency sizes
- **Duration**: ~2-3 minutes
- **Artifact**: `size-analysis` (90-day retention)
- **PR Comment**: Auto-posts analysis on PRs

#### Memory Safety (Miri)
- Runs MIRI interpreter on test suite
- Detects undefined behavior
- Checks for memory unsafety
- Non-blocking (informational)
- **Duration**: ~2-3 minutes
- **Artifact**: `miri-results` (30-day retention)

### PR Integration
Binary size automatically posted to PR comments:
```markdown
### 📊 Binary Size Analysis

**Release Binary Size**: 14.2 MB

### Section Sizes
[... size breakdown ...]
```

### Baseline Comparisons
- First run: Establishes baseline
- Subsequent runs: Compares against baseline
- Alerts if size increases >10%
- Helps catch bloat early

### Local Testing
```bash
# Measure compile time
time cargo build --release

# Check binary size
ls -lh target/release/pr-forge

# Run miri checks
cargo miri test --lib
```

---

## 🚀 Quick Start

### For Maintainers

1. **Enable Workflows**
   ```bash
   git add .github/workflows/
   git commit -m "Add GitHub Actions workflows"
   git push origin main
   ```

2. **Configure Secrets** (if needed)
   - Go to: Settings → Secrets and variables → Actions
   - Add any required secrets (API keys, etc.)

3. **Verify Workflows**
   - Check: Actions tab → See all workflows
   - Monitor: Recent runs and logs

### For Contributors

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes**
   - Edit code, add tests
   - Local `cargo test` passes

3. **Push and Create PR**
   ```bash
   git push origin feature/my-feature
   # Create PR via GitHub UI
   ```

4. **Address Feedback**
   - GitHub Actions runs automatically
   - Fix any failures shown
   - Push fixes, Actions re-runs

5. **Merge When Green**
   - All checks pass ✅
   - Code review approved ✅
   - Merge to main

---

## 🔧 Troubleshooting

### Build Fails on Windows

**Symptom**: Exit code 143 or 101
**Causes**: 
- Antivirus interference
- Insufficient RAM
- Disk space issues

**Solutions**:
1. Increase runner memory (upgrade GH runner)
2. Disable antivirus for test folder
3. Clean cache: Settings → Caches → Delete

### Tests Timeout

**Symptom**: Job takes >30 min
**Causes**:
- Network I/O in tests
- Slow CI runner
- Hanging processes

**Solutions**:
```yaml
# Add timeout to prevent hangs
timeout-minutes: 15
```

### Cache Issues

**Symptom**: Stale dependencies or build failures
**Solution**:
```bash
# Manual cache clear
gh workflow run build.yml -f clear-cache=true
```

### Permission Denied Errors

**Symptom**: `permission denied` on macOS/Linux
**Cause**: Binary not executable in artifact
**Solution**:
```yaml
- name: Make executable
  run: chmod +x target/release/pr-forge
```

---

## 📊 Monitoring & Alerts

### Check Status
- **GitHub UI**: Actions tab shows real-time status
- **CLI**: `gh workflow list` and `gh run list`
- **Email**: GitHub notifies on failures

### Set Up Alerts
1. Go to: Actions → Select workflow
2. Click: Create Status Badge
3. Copy markdown
4. Add to README.md

### Dashboards
- GitHub Dashboard: Settings → Actions → Usage
- Cost: View runner usage and costs
- Logs: Detailed job logs available for 90 days

---

## 💡 Best Practices

✅ **Do**:
- Keep workflows DRY (use composite actions)
- Cache aggressively (registry, dependencies)
- Test locally before pushing
- Document custom steps
- Use semantic versioning for releases

❌ **Don't**:
- Hardcode secrets in workflows
- Use `always()` unless necessary
- Ignore test failures
- Store large artifacts
- Commit generated files

---

## 🔐 Security

### Secret Management
- Never commit secrets or API keys
- Use GitHub Secrets for sensitive data
- Rotate regularly (90-day max)
- Audit access logs

### Example: Adding Groq API Key
1. Go to: Settings → Secrets and variables → Actions
2. Click: New repository secret
3. Name: `GROQ_API_KEY`
4. Value: `gsk_...` (from Groq dashboard)
5. Reference in workflow:
   ```yaml
   env:
     GROQ_API_KEY: ${{ secrets.GROQ_API_KEY }}
   ```

### Dependency Scanning
- RustSec automatically checks vulnerabilities
- Blocks PRs if vulnerabilities found
- Weekly reports for information
- Subscribe to RustSec advisories

---

## 📈 Performance Metrics

### Typical Times
| Job | Time | Notes |
|-----|------|-------|
| Check | 2-3m | Fastest, no linking |
| Test | 2-3m | Platform-independent |
| Format | 30s | Usually instant |
| Lint (Clippy) | 1-2m | Depends on code changes |
| Build Linux | 3-5m | Fastest to compile |
| Build macOS x86 | 4-6m | Medium |
| Build macOS ARM | 5-7m | Cross-compilation slower |
| Build Windows | 6-8m | Most dependencies needed |
| Total | ~15-20m | All jobs in parallel |

### Cost Estimation
- **Public repo**: FREE (unlimited Actions minutes)
- **Private repo**: 
  - 2000 minutes/month included
  - Linux: 1x cost
  - macOS: 10x cost
  - Windows: 2x cost

Example monthly cost (3 builds/day):
- 3 builds × 20 min = 60 min/day
- 60 × 30 = 1800 min/month
- Mostly Linux: ~$100-150/month

---

## 🤝 Contributing

To modify workflows:
1. Create feature branch
2. Edit .github/workflows/*.yml
3. Test locally with `act` tool
4. Create PR, get review
5. Merge to main
6. Workflow changes live immediately

### Testing Workflows Locally
Install `act`: https://github.com/nektos/act

```bash
# List available workflows
act -l

# Run specific workflow
act -j build

# Run with event
act -e push
```

---

## 📚 Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Syntax Reference](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [Runner Specifications](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners)
- [Marketplace Actions](https://github.com/marketplace?type=actions)
- [RustSec Advisory Database](https://rustsec.org/)

---

**Last Updated**: 2024  
**Maintained By**: PR-Forge Team  
**Status**: ✅ Active
