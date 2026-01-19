# PR-Forge

**Forge better pull requests from your branch.**

A senior-grade CLI tool that analyzes a Git branch and generates professional, review-ready Pull Request descriptions with embedded standards enforcement.

## Features

- **Branch Analysis**: Automatically compares your feature branch against a base branch
- **Commit Aggregation**: Groups and normalizes commits by intent (feat, fix, refactor, chore)
- **AI-Powered Changelog**: Enhanced analysis using Groq AI (Mixtral 8x7b) to generate categorized changelogs with:
  - 🎨 UI/UX Improvements
  - 🔧 Build & Configuration Fixes
  - 🐛 Bug Fixes
  - 📦 Dependencies
  - 🚀 Performance Improvements
  - 📝 Documentation
  - 🧪 Testing
  - ⚙️ Refactoring
  - 🎯 Breaking Changes
- **Smart Rules Engine**: Detects missing tests, breaking changes, config modifications, and poor commit quality
- **Professional Output**: Generates structured PR descriptions with:
  - Executive summary
  - AI-generated categorized changelog (when enabled)
  - Key changes grouped by type
  - Files and areas touched
  - Commits analyzed
  - Impact assessment
  - Risks & notes from rule violations
  - Reviewer checklist
- **Multiple Formats**: Output as Markdown (default), plain text, or JSON
- **Graceful Fallback**: Falls back to rule-based analysis if AI is unavailable or disabled
- **Rate Limited**: Built-in rate limiting for API calls to prevent quota exhaustion

## Installation

### From Source

```bash
git clone <repository>
cd pr-forge
cargo build --release
./target/release/pr-forge --help
```

### AI Integration Setup

PR-Forge includes optional AI-powered changelog generation using Groq's API. To enable AI features:

1. **Get a Groq API Key**: Sign up at [console.groq.com](https://console.groq.com) to get your free API key
2. **Set Environment Variable**:
   ```bash
   export GROQ_API_KEY="gsk_your_api_key_here"
   ```
   
   Or create a `.env` file in your project root:
   ```
   GROQ_API_KEY=gsk_your_api_key_here
   ```

3. **AI is enabled by default**. Use `--disable-ai` flag to use rule-based analysis only.

**Note**: AI features are optional. The tool works perfectly fine without an API key, falling back to rule-based analysis.

## Quick Start

### Basic Usage

```bash
# Analyze current branch against default base (main/master/develop)
# AI analysis enabled by default (requires GROQ_API_KEY)
pr-forge feature/auth-refactor

# Specify explicit base branch
pr-forge feature/auth-refactor --base develop

# Disable AI and use rule-based analysis only
pr-forge feature/auth-refactor --disable-ai

# Output as JSON for CI integration
pr-forge feature/auth-refactor --format json

# Short mode (concise output)
pr-forge feature/auth-refactor --short
```

### Example Output

```markdown
## Summary
Added new authentication middleware and improved auth flow maintainability.

## Changelog

🔧 Build & Configuration Fixes
**JWT Middleware Configuration** - Centralized JWT validation logic with improved error handling
  - `src/auth/jwt.rs`
  - `src/middleware/mod.rs`

🐛 Bug Fixes
**Token Validation Edge Cases** - Fixed missing token edge case handling in authentication flow
  - `src/auth/jwt.rs`

⚙️ Refactoring
**Auth Utilities Extraction** - Removed duplicate middleware checks and extracted reusable auth utilities
  - `src/middleware/mod.rs`

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
        --disable-ai          Disable AI-powered analysis (use rule-based only)
    -h, --help                Print help
    -v, --version             Print version
```

## AI Integration

PR-Forge uses Groq's Mixtral 8x7b model to generate intelligent, categorized changelogs. The AI analyzes your commits and files to:

- **Categorize changes** into logical groups (UI/UX, Bug Fixes, Performance, etc.)
- **Generate detailed descriptions** for each change with context
- **Identify breaking changes** automatically
- **Create professional summaries** that go beyond commit messages

### How It Works

1. The tool analyzes your commits and file changes locally
2. Sends structured context to Groq API (commits, file paths, change types)
3. Receives categorized changelog with detailed entries
4. Integrates AI output seamlessly with rule-based analysis
5. Falls back gracefully if API is unavailable (no API key, network issues, etc.)

### Rate Limiting

Built-in rate limiting prevents API quota exhaustion:
- Configurable request limits per time window
- Automatic retry with exponential backoff
- Graceful degradation to rule-based analysis

### Privacy & Security

- Only commit messages and file paths are sent to the API (no code content)
- All analysis happens locally first
- API key is never logged or exposed
- Works completely offline with `--disable-ai` flag

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
│   ├── ai/
│   │   ├── mod.rs           # AI configuration & types
│   │   ├── groq_client.rs   # Groq API client
│   │   └── rate_limiter.rs  # Rate limiting for API calls
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
[AI Client] → Generate categorized changelog (optional)
    ↓
[PR Builder] → Generate narrative sections
    ↓
[Output Formatter] → Markdown/Plain/JSON
    ↓
Console Output
```

## Design Principles

1. **No Guessing**: The tool detects signals, it doesn't guess intent
2. **Deterministic**: Core analysis is local and reproducible; AI enhances but doesn't replace
3. **Graceful Degradation**: Works perfectly without AI; enhances when available
4. **Pluggable**: Rules can be easily extended (V2+)
5. **Senior-First**: Output matches senior-team standards
6. **Safe**: Never modifies branches, commits, or working tree
7. **Privacy-Focused**: Only sends minimal metadata to AI (no code content)

## Contributing

This is a solo project, but future contributions may be accepted. See the module architecture above for where to add features.

**Built by engineers, for engineers.**
Forge better pull requests.
