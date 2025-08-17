# Security Policy

## Supported Versions

We provide security updates for the following versions of wait-for:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in wait-for, please report it responsibly.

### How to Report

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them privately by:

1. **Email**: Send details to <kalyuzhni.sergei@gmail.com>
2. **Subject**: Include "SECURITY" in the subject line
3. **Details**: Provide as much information as possible

### What to Include

Please include the following information in your report:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** assessment
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up

### Response Timeline

- **24 hours**: Acknowledgment of your report
- **72 hours**: Initial assessment and triage
- **7 days**: Detailed response with timeline for fix
- **30 days**: Security patch release (if applicable)

### Disclosure Policy

- We will coordinate disclosure timing with you
- We prefer responsible disclosure after a fix is available
- We will acknowledge your contribution (unless you prefer anonymity)
- We may offer a bug bounty for significant vulnerabilities

## Security Considerations

### Network Security

wait-for makes network connections to test connectivity. Consider these security aspects:

#### DNS Resolution

- Uses system DNS resolver
- Subject to DNS spoofing attacks
- Consider using specific DNS servers in secure environments

#### TCP Connections

- Creates outbound TCP connections
- Does not transmit sensitive data
- Uses standard socket connections

#### HTTP/HTTPS Requests

- Makes HTTP requests to specified endpoints
- Follows redirects (security consideration)
- Does not validate TLS certificates by default
- Consider network policies in production

### Input Validation

wait-for validates:

- Host/port formats
- URL formats
- Time duration formats
- Command line arguments

### Command Execution

When using the `-- command` feature:

- Executes arbitrary commands after successful connections
- Commands run with the same privileges as wait-for
- Consider principle of least privilege
- Validate command arguments in scripts

### Environment Variables

wait-for reads environment variables:

- `WAIT_FOR_TIMEOUT`
- `WAIT_FOR_INTERVAL`

These are not security-sensitive but could affect behavior.

## Best Practices

### Production Usage

1. **Least Privilege**: Run with minimal required permissions
2. **Network Policies**: Use firewall rules to restrict connections
3. **Input Validation**: Validate hostnames and URLs in scripts
4. **Logging**: Monitor connection attempts in security-sensitive environments
5. **Timeouts**: Use reasonable timeouts to prevent resource exhaustion

### Docker Security

When using in containers:

```dockerfile
# Run as non-root user
USER 1000:1000

# Minimal image
FROM scratch
COPY wait-for /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/wait-for"]
```

### Kubernetes Security

In Kubernetes environments:

```yaml
# Security context
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
      - ALL
```

## Known Security Considerations

### 1. DNS Spoofing

- **Risk**: DNS responses could be spoofed
- **Mitigation**: Use trusted DNS servers, validate endpoints

### 2. Command Injection

- **Risk**: Malicious commands in `-- command` syntax
- **Mitigation**: Validate input, use absolute paths

### 3. Resource Exhaustion

- **Risk**: Long timeouts could consume resources
- **Mitigation**: Set reasonable timeout limits

### 4. Network Scanning

- **Risk**: Could be used for network reconnaissance
- **Mitigation**: Monitor usage, apply network policies

## Updates and Notifications

- Security updates will be released as patch versions
- Critical vulnerabilities will be announced in release notes
- Subscribe to GitHub releases for notifications

## Questions?

If you have questions about security but not a vulnerability to report, please:

- Open a GitHub issue with the `security` label
- Email <kalyuzhni.sergei@gmail.com> with "SECURITY QUESTION" in subject

Thank you for helping keep wait-for secure!
