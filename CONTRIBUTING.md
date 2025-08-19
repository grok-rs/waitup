# Contributing to waitup

Thank you for your interest in contributing to waitup! This document provides guidelines and information for contributors.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold professional and respectful behavior.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected vs actual behavior**
- **Environment information** (OS, Rust version, waitup version)
- **Minimal test case** if possible

### Suggesting Features

Feature suggestions are welcome! Please:

- **Search existing issues** first to avoid duplicates
- **Describe the use case** and motivation
- **Provide examples** of how the feature would be used
- **Consider backward compatibility**

### Development Setup

1. **Fork and clone** the repository:

   ```bash
   git clone https://github.com/grok-rs/waitup.git
   cd waitup
   ```

2. **Install Rust** (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**:

   ```bash
   cargo build
   ```

4. **Run tests**:

   ```bash
   cargo test
   cargo test --test integration_tests
   ```

5. **Run linting**:

   ```bash
   cargo clippy -- -D warnings
   cargo fmt --all -- --check
   ```

### Making Changes

1. **Create a feature branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following these guidelines:
   - Write clear, readable code
   - Add tests for new functionality
   - Update documentation as needed
   - Follow existing code style
   - Keep commits atomic and well-described

3. **Test thoroughly**:

   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Update documentation** if needed:
   - Update README.md for user-facing changes
   - Update CHANGELOG.md following the format
   - Add rustdoc comments for new public APIs

### Code Style

- **Use `cargo fmt`** for consistent formatting
- **Follow Rust conventions** and idioms
- **Write clear variable and function names**
- **Add documentation** for public APIs
- **Handle errors properly** using `Result` types
- **Prefer explicit over implicit** behavior

### Testing

- **Write unit tests** for individual functions
- **Write integration tests** for CLI behavior
- **Test error conditions** and edge cases
- **Ensure tests are deterministic** and don't depend on external services
- **Mock external dependencies** when necessary

### Documentation

- **Use rustdoc comments** (`///`) for public APIs
- **Provide examples** in documentation
- **Keep README.md updated** with new features
- **Update help text** in the CLI for new options

### Pull Request Process

1. **Update tests and documentation**
2. **Ensure CI passes** (tests, clippy, formatting)
3. **Update CHANGELOG.md** with your changes
4. **Create a clear PR description** explaining:
   - What changes were made
   - Why they were made
   - How to test them
5. **Link related issues** if applicable
6. **Be responsive** to review feedback

### Release Process

Releases follow semantic versioning:

- **Patch** (x.y.Z): Bug fixes, documentation updates
- **Minor** (x.Y.z): New features, backward compatible
- **Major** (X.y.z): Breaking changes

### Getting Help

- **Open an issue** for questions about contributing
- **Check existing documentation** and issues first
- **Be patient and respectful** in all interactions

## Development Tips

### Local Testing

Test against real services:

```bash
# Start a local server for testing
python3 -m http.server 8000 &

# Test waitup
cargo run -- localhost:8000 --timeout 10s
```

### Cross-platform Testing

Test on different platforms if possible:

- Linux (primary target)
- macOS
- Windows

### Performance Testing

Use the benchmarks to test performance:

```bash
cargo bench
```

### Integration Testing

Test with real-world scenarios:

- Docker containers
- Kubernetes pods
- CI/CD pipelines

## Thank You

Your contributions make waitup better for everyone. Thank you for taking the time to contribute!

## Questions?

Feel free to open an issue with the `question` label if you need help or clarification.
