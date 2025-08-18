# Linting Configuration Guide

This document explains the different linting levels available for the wait-for project and how to use them.

## Overview

We provide several linting configurations to balance code quality with development productivity:

- **Quick**: Fast pre-commit checks (< 10 seconds)
- **Reasonable**: Good balance for daily development (< 30 seconds)
- **Strict**: Comprehensive checks before pushing (< 60 seconds)
- **CI**: Matches what runs in continuous integration
- **Super-strict**: All possible lints including restriction (may have false positives)

## Quick Reference

```bash
# Using the lint script (recommended)
./scripts/lint.sh quick      # Fast pre-commit checks
./scripts/lint.sh reasonable # Daily development checks
./scripts/lint.sh strict     # Pre-push comprehensive checks
./scripts/lint.sh ci         # Match CI exactly

# Using make commands
make lint-quick              # Same as above
make lint-reasonable         # Same as above
make lint-strict            # Same as above
make lint-ci                # Same as above

# Manual clippy commands
cargo clippy --all-targets --all-features -- -D warnings -D clippy::correctness
```

## Lint Levels Explained

### Quick Lints (Pre-commit)
**Purpose**: Catch obvious issues before committing
**Time**: < 10 seconds
**Includes**:
- Format check (`cargo fmt`)
- Basic compile check (`cargo check`)
- Essential clippy lints (correctness, suspicious)
- Quick unit tests

**Clippy flags**:
```bash
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::correctness \
  -D clippy::suspicious \
  -W clippy::complexity \
  -W clippy::perf \
  -W clippy::style
```

### Reasonable Lints (Development)
**Purpose**: Good balance of quality vs development speed
**Time**: < 30 seconds
**Includes**:
- Everything from Quick
- Pedantic and nursery lints (with sensible allows)
- Full test suite
- Documentation build check

**Clippy flags**:
```bash
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::correctness \
  -D clippy::suspicious \
  -D clippy::complexity \
  -D clippy::perf \
  -D clippy::style \
  -W clippy::pedantic \
  -W clippy::nursery \
  -A clippy::missing_errors_doc \
  -A clippy::missing_panics_doc \
  -A clippy::module_name_repetitions \
  -A clippy::similar_names \
  -A clippy::too_many_lines \
  -A clippy::cast_precision_loss \
  -A clippy::cast_possible_truncation \
  -A clippy::cast_sign_loss \
  -A clippy::must_use_candidate
```

### Strict Lints (Pre-push)
**Purpose**: Comprehensive checks before sharing code
**Time**: < 60 seconds
**Includes**:
- Everything from Reasonable
- Stricter unwrap/expect/panic checking
- Documentation examples testing
- Security audit (if tools installed)
- License compliance check

**Clippy flags**:
```bash
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::correctness \
  -D clippy::suspicious \
  -D clippy::complexity \
  -D clippy::perf \
  -D clippy::style \
  -D clippy::pedantic \
  -D clippy::nursery \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -W clippy::todo \
  -W clippy::unimplemented \
  -W clippy::dbg_macro \
  -A clippy::missing_errors_doc \
  -A clippy::missing_panics_doc \
  -A clippy::module_name_repetitions \
  -A clippy::similar_names
```

### CI Lints
**Purpose**: Match exactly what runs in GitHub Actions
**Includes**:
- Same as Strict but without optional tools
- Focuses on what can be reliably run in CI environment

### Super-strict Lints
**Purpose**: Catch every possible issue (may have false positives)
**Warning**: Includes `clippy::restriction` which can be overly pedantic
**Use case**: Code quality audits, learning about potential improvements

## Configuration Files

### clippy.toml
Contains reasonable thresholds for clippy lints:
- Cognitive complexity: 20 (instead of default 25)
- Type complexity: 75 (instead of default 100)
- Function arguments: 6 (instead of default 7)
- Function lines: 100 (default)

### lefthook.yml
Pre-commit and pre-push hooks using reasonable lints:
- **Pre-commit**: Quick checks for immediate feedback
- **Pre-push**: Comprehensive checks before sharing

### .github/workflows/lint.yml
CI configuration with:
- **Required checks**: Format, reasonable clippy, docs, security, licenses
- **Optional checks**: Strict clippy (warnings only), unused deps

## Why This Approach?

### Problems with `clippy::restriction`
The restriction lint group contains contradictory and overly pedantic lints:
- Some lints conflict with others
- Many are opinion-based rather than objectively improving quality
- Can slow down development without meaningful benefits

### Our Solution
1. **Essential lints always enforced**: correctness, suspicious, complexity, perf, style
2. **Quality lints with allowances**: pedantic, nursery (with sensible exceptions)
3. **Optional strict checking**: restriction lints available but not required
4. **Incremental quality**: Stricter checks at push/CI time vs commit time

## Usage Examples

### Daily Development Workflow
```bash
# Before committing
make lint-quick

# Before pushing feature branch
make lint-reasonable

# Before merging to main
make lint-strict
```

### Integration with Editors
Many editors can be configured to run reasonable lints on save:

**VS Code (rust-analyzer)**:
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [
    "--all-targets", "--all-features", "--",
    "-D", "warnings",
    "-D", "clippy::correctness",
    "-W", "clippy::pedantic"
  ]
}
```

### Custom Lint Runs
```bash
# Just format checking
./scripts/lint.sh format

# Auto-fix what's possible
./scripts/lint.sh fix

# See all available commands
./scripts/lint.sh help
```

## Installing Development Tools

The lint scripts can use additional tools if installed:
```bash
# Install all development tools
make install

# Or manually
cargo install cargo-audit cargo-deny cargo-machete
```

## Troubleshooting

### Rustfmt Warnings
The project uses some nightly-only rustfmt features. On stable toolchain, you'll see warnings but formatting still works.

### Tool Not Found
If optional tools (cargo-audit, cargo-deny, cargo-machete) aren't installed, the scripts will skip those checks with a warning.

### Performance
- Quick lints: ~5-10 seconds
- Reasonable lints: ~20-30 seconds
- Strict lints: ~45-60 seconds
- Times depend on system performance and cache state

## Contributing

When contributing to this project:
1. Ensure `make lint-reasonable` passes before creating PRs
2. CI runs `make lint-ci` which must pass
3. Consider running `make lint-strict` before important changes

The goal is productive development with high code quality, not perfectionism.
