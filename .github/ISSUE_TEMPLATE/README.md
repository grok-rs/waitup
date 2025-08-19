# GitHub Issues Best Practices

This document outlines best practices for creating effective GitHub issues for the waitup project.

## 🎯 Before Creating an Issue

1. **Search existing issues** - Check if your issue has already been reported
2. **Read the documentation** - Review the README and [project wiki](https://github.com/grok-rs/waitup/wiki)
3. **Try the latest version** - Ensure you're using the most recent release
4. **Use the right template** - Choose the appropriate issue template for your needs

## 📋 Issue Templates

We provide several issue templates to help you provide the right information:

### 🐛 Bug Reports
Use this template when waitup isn't working as expected.

**Best practices:**
- Provide exact reproduction steps
- Include the complete command and output
- Specify your environment details
- Attach relevant logs with `--verbose` flag

### ✨ Feature Requests
Use this template to suggest new functionality.

**Best practices:**
- Clearly explain the problem you're trying to solve
- Describe your proposed solution in detail
- Consider how it would benefit other users
- Include example usage syntax if applicable

### 📚 Documentation Issues
Use this template for documentation problems.

**Best practices:**
- Specify the exact location of the issue
- Quote the current content that's problematic
- Suggest specific improvements
- Consider your target audience (beginners vs advanced)

### ⚡ Performance Issues
Use this template for performance problems.

**Best practices:**
- Provide specific performance measurements
- Include your system specifications
- Compare with expected performance
- Suggest potential optimizations if you have ideas

### ❓ Questions
Use this template when you need help understanding waitup.

**Best practices:**
- Be specific about what you're trying to accomplish
- Include your current approach and what you've tried
- Provide context about your use case
- Check documentation first before asking

## ✅ Writing Effective Issues

### Title Guidelines
- **Be specific and descriptive**
  - ✅ Good: "HTTP health check fails with custom headers containing colons"
  - ❌ Bad: "doesn't work"

- **Include key information**
  - ✅ Good: "Docker container exits with code 1 on Alpine Linux"
  - ❌ Bad: "Error in Docker"

### Description Best Practices

#### For Bug Reports
```markdown
## Expected Behavior
waitup should successfully connect to localhost:8080

## Actual Behavior
Connection times out after 30 seconds with error: "connection refused"

## Steps to Reproduce
1. Start a local server: `python -m http.server 8080`
2. Run waitup: `waitup localhost:8080 --timeout 60s`
3. Observe timeout error

## Environment
- waitup version: v1.0.2
- OS: Ubuntu 22.04
- Installation: cargo install waitup
```

#### For Feature Requests
```markdown
## Problem
When waiting for multiple HTTP endpoints, each endpoint might expect different status codes (200, 204, 404, etc.), but waitup only supports one global `--expect-status`.

## Proposed Solution
Allow per-target status code specification:
```bash
waitup http://api.example.com/health:200 http://admin.example.com/ready:204
```

## Use Cases
- Health checks with different success criteria
- API endpoints that return different status codes when ready
- Legacy services with non-standard success responses
```

### Code and Output Formatting
- Use code blocks with appropriate language tags:
  ```bash
  # Commands
  waitup localhost:8080 --verbose
  ```

  ```rust
  // Rust code
  let target = Target::tcp("localhost", 8080)?;
  ```

- Include complete error messages and stack traces

### Environment Information
Always include relevant environment details:
- waitup version (`waitup --version`)
- Operating system and version
- Installation method (cargo, docker, binary)
- Container runtime if applicable
- Network configuration if relevant

## 🏷️ Labels and Assignment

Our maintainers will add appropriate labels to categorize and prioritize issues:

- **Type labels**: `bug`, `enhancement`, `documentation`, `question`, `performance`
- **Priority labels**: `critical`, `high`, `medium`, `low`
- **Status labels**: `needs-triage`, `confirmed`, `in-progress`, `blocked`
- **Area labels**: `cli`, `http`, `tcp`, `dns`, `docker`, `library`

## 🤝 Community Guidelines

### Be Respectful
- Use inclusive language
- Be patient with responses
- Help others when you can
- Follow our Code of Conduct

### Provide Context
- Explain your use case and requirements
- Share relevant background information
- Include links to related issues or discussions

### Follow Up
- Respond to requests for additional information
- Test proposed solutions when available
- Close issues when resolved
- Update issues if circumstances change

## 🚀 Contributing

If you're willing to help implement a feature or fix a bug:

1. Comment on the issue expressing your interest
2. Wait for maintainer feedback on approach
3. Fork the repository and create a feature branch
4. Submit a pull request referencing the issue

## 📞 Getting Help

- **General questions**: Use GitHub Discussions
- **Bug reports**: Use the bug report template
- **Security issues**: Use our private security reporting
- **Documentation**: Check the GitHub repository docs first

## Examples of Great Issues

### Excellent Bug Report
> **Title**: "waitup fails to resolve IPv6 addresses on macOS Monterey"
>
> Provides exact reproduction steps, complete error output, system details, and explains the impact on the user's workflow.

### Excellent Feature Request
> **Title**: "Add support for custom HTTP methods in health checks"
>
> Clearly explains the problem, provides concrete use cases, shows example syntax, and demonstrates understanding of the broader impact.

### Excellent Documentation Issue
> **Title**: "README installation section missing Windows-specific instructions"
>
> Points to specific documentation location, explains what's missing, suggests concrete improvements, and offers to help with the fix.

---

Thank you for helping make waitup better! Your detailed issue reports and thoughtful feature requests help us build a tool that works well for everyone.
