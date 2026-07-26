# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: Supported |

## Reporting a Vulnerability

If you discover a security vulnerability, please do **NOT** submit it via a public GitHub Issue.

Please report privately through one of the following channels:

1. **GitHub Security Advisory**: Go to the repository's [Security](https://github.com/ZeroFxc/McpStudio/security) tab and select "Report a vulnerability"
2. **Email**: Contact the project maintainer (if configured)

Please include the following information in your report:

- Detailed description of the vulnerability
- Steps to reproduce
- Affected versions
- Possible fix suggestions (if any)

## Response Time

- We will **acknowledge** your report within **48 hours**
- We will provide an initial assessment within **7 days**
- Fix time depends on the severity and complexity of the vulnerability

## Disclosure Process

1. Reporter privately submits the vulnerability
2. Project maintainers confirm and assess the vulnerability
3. Develop and test a fix patch
4. Release a new version with the fix
5. Publicly disclose vulnerability details within **30 days** of release

## Security Best Practices

### For Users

- Always use the latest version
- Do not trust MCP configurations from untrusted sources
- MCP tools can execute arbitrary code; be cautious when adding MCP servers from unknown sources

### For Developers

- Do not hardcode secrets or credentials in code
- Use environment variables or secure storage for sensitive information
- Regularly update dependencies to fix known vulnerabilities

---

[中文文档](SECURITY.md)