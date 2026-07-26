# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Workflow cache optimization: npm dependency caching and Rust cache sharing

## [0.1.0] - 2026-07-27

### Added
- MCP Registration Management: Supports stdio and Streamable HTTP connections
- Tool Discovery & Routing: Auto-discovers tool lists from registered MCPs with dictionary-style search
- Usage Statistics: Records call count and last-used time for each tool
- HTTP Service: Built-in HTTP MCP Server (default port 9277)
- Marketplace Search: Search community MCP tools from GitHub
- Internationalization: Chinese and English UI switching
- Result Caching: Automatic caching of tool call results with incremental read support
- Settings Page: Language switching, storage directory management
- CI/CD Workflows: Automatic build checks and Release publishing
- Workflow Cache: npm and Rust dependency caching

[Unreleased]: https://github.com/ZeroFxc/McpStudio/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.0

---

[中文文档](CHANGELOG.md)