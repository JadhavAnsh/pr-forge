# PR-Forge

**Forge better pull requests from your branch.**

A senior-grade CLI tool that analyzes a Git branch and generates professional, review-ready Pull Request descriptions with embedded standards enforcement.

## Features

- **Branch Analysis**: Automatically compares your feature branch against a base branch
- **Commit Aggregation**: Groups and normalizes commits by intent (feat, fix, refactor, chore)
- **Smart Rules Engine**: Detects missing tests, breaking changes, config modifications, and poor commit quality
- **Professional Output**: Generates structured PR descriptions with:
  - Executive summary
  - Key changes grouped by type
  - Files and areas touched
  - Commits analyzed
  - Impact assessment
  - Risks & notes from rule violations
  - Reviewer checklist
- **Multiple Formats**: Output as Markdown (default), plain text, or JSON
- **No Remote Calls**: Fully local analysis—fast, safe, and deterministic

## Installation

### From Source

```bash
git clone <repository>
cd pr-forge
cargo build --release
./target/release/pr-forge --help
```

## Quick Start

### Basic Usage

```bash
# Analyze current branch against default base (main/master/develop)
pr-forge feature/auth-refactor

# Specify explicit base branch
pr-forge feature/auth-refactor --base develop

# Output as JSON for CI integration
pr-forge feature/auth-refactor --format json

# Short mode (concise output)
pr-forge feature/auth-refactor --short
```

### Example Output

```markdown
## Summary
Added new authentication middleware and improved auth flow maintainability.

## Key Changes
• Centralized JWT validation logic
• Fixed missing token edge case handling
• Removed duplicate middleware checks

## Files / Areas Touched
- src/auth/jwt.rs
- src/middleware/mod.rs
- tests/auth_test.rs

## Commits Analyzed
- a1b2c3d: feat: centralize jwt validation
- d4e5f6g: fix: handle missing token edge case
- h7i8j9k: refactor: extract auth utils

## Impact
This PR improves code quality and maintainability. New functionality is available for end users.

## Risks & Notes
- ⚠️ No test changes detected for code modifications.
- ⚠️ Configuration files changed. Verify environment updates are documented.

## Reviewer Checklist
✅ Code changes are logical and well-organized
☐ Tests are present and cover new functionality
☐ Documentation is updated if needed
✅ No breaking changes introduced
✅ Configuration changes are documented
```

## CLI Options

```
USAGE:
    pr-forge <BRANCH> [OPTIONS]

ARGS:
    <BRANCH>  The branch to analyze

OPTIONS:
    -b, --base <BRANCH>       Base branch to compare against (default: main/master/develop)
    -f, --format <FORMAT>     Output format: markdown, plain, json [default: markdown]
    -s, --short               Generate shorter PR description
    -r, --repo <PATH>         Repository path [default: .]
    -h, --help                Print help
    -v, --version             Print version
```

## Rules Engine (V1)

The tool includes four default rules that evaluate your branch:

### 1. **Tests Rule**
- **Detects**: Code changes without corresponding test modifications
- **Severity**: ⚠️ Warning
- **Message**: "No test changes detected for code modifications."

### 2. **Breaking Change Rule**
- **Detects**: Potential API-breaking changes, deleted public files, breaking commit messages
- **Severity**: 🚨 Critical / ⚠️ Warning
- **Message**: "Potential breaking changes detected. Consider adding migration notes."

### 3. **Config Change Rule**
- **Detects**: Modifications to `.env`, `.yml`, `.json`, `docker-compose`, or config files
- **Severity**: ⚠️ Warning
- **Message**: "Configuration files changed. Verify environment updates are documented."

### 4. **Commit Quality Rule**
- **Detects**: Non-descriptive commit messages like "fix", "update", "wip"
- **Severity**: ℹ️ Info
- **Message**: "X commit message(s) are non-descriptive; PR summary compensates for this."

## Module Architecture

```
pr-forge/
├── src/
│   ├── main.rs              # Entry point & orchestration
│   ├── error.rs             # Custom error types
│   ├── cli/
│   │   ├── mod.rs
│   │   └── args.rs          # Argument parsing with clap
│   ├── git/
│   │   ├── mod.rs
│   │   ├── repo.rs          # Repository operations
│   │   ├── branch.rs        # Branch utilities
│   │   ├── commit.rs        # Commit analysis
│   │   └── diff.rs          # Diff utilities (V2+)
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── commit_analyzer.rs   # Classify commits
│   │   ├── change_classifier.rs # Group changes
│   │   └── file_analyzer.rs     # Analyze file patterns
│   ├── rules/
│   │   ├── mod.rs
│   │   ├── engine.rs        # Rule trait & evaluation
│   │   ├── ruleset.rs       # Default ruleset factory
│   │   └── rules/
│   │       ├── mod.rs       # All rule implementations
│   │       ├── tests_rule.rs
│   │       ├── breaking_change_rule.rs
│   │       ├── config_change_rule.rs
│   │       └── commit_quality_rule.rs
│   ├── pr/
│   │   ├── mod.rs
│   │   ├── model.rs         # Core data models
│   │   ├── builder.rs       # PR description builder
│   │   └── formatter.rs     # Format for display
│   └── output/
│       ├── mod.rs           # Output dispatcher
│       ├── markdown.rs      # Markdown formatting
│       ├── plain.rs         # Plain text formatting
│       └── json.rs          # JSON output
```

## Data Flow

```
Git Repository
    ↓
[Branch Analyzer] → Extract commits & files
    ↓
[Analysis Models] → CommitInfo, BranchAnalysis
    ↓
[Rules Engine] → Evaluate & generate findings
    ↓
[PR Builder] → Generate narrative sections
    ↓
[Output Formatter] → Markdown/Plain/JSON
    ↓
Console Output
```

## Design Principles

1. **No Guessing**: The tool detects signals, it doesn't guess intent
2. **Deterministic**: All analysis is local and reproducible
3. **Pluggable**: Rules can be easily extended (V2+)
4. **Senior-First**: Output matches senior-team standards
5. **Safe**: Never modifies branches, commits, or working tree

## Contributing

This is a solo project, but future contributions may be accepted. See the module architecture above for where to add features.

**Built by engineers, for engineers.**
Forge better pull requests.
