# Contributing Guide

Thank you for your interest in McpStudio! We welcome all forms of contributions.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT-en.md). By participating, you agree to abide by its terms.

## How to Contribute

### Reporting Bugs

If you discover a bug, please submit it via GitHub Issues with the following information:

- **Title**: Briefly describe the issue
- **Description**: Detailed explanation of the bug's behavior
- **Steps to Reproduce**: How to trigger the bug
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: Operating system, Node.js version, Rust version
- **Screenshots/Logs** (if available): Attach relevant screenshots or error logs

### Feature Requests

If you have a feature suggestion, please submit it via GitHub Issues with:

- **Background**: Why this feature is needed
- **Feature Description**: What you expect the feature to look like
- **Alternatives**: Other solutions you've considered

### Submitting Pull Requests

1. **Fork the repository** and clone it locally
2. **Create a feature branch**: `git checkout -b feat/your-feature-name`
3. **Develop** following the code style guide
4. **Commit**: Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification
5. **Push to your fork**: `git push origin feat/your-feature-name`
6. **Create a Pull Request** describing your changes

#### Branch Naming Convention

- `feat/xxx` — New feature
- `fix/xxx` — Bug fix
- `docs/xxx` — Documentation updates
- `refactor/xxx` — Code refactoring
- `chore/xxx` — Build/tooling changes

#### Commit Message Convention

Follow the Conventional Commits format:

```
<type>(<scope>): <description>

[optional body]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no logic changes)
- `refactor`: Code refactoring
- `perf`: Performance optimization
- `test`: Test-related
- `chore`: Build/tooling changes

## Development Environment Setup

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- Windows requires [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Clone the Repository

```bash
git clone --recurse-submodules git@github.com:ZeroFxc/McpStudio.git
cd McpStudio
```

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Code Style Guide

### Vue / TypeScript

- Use TypeScript strict mode
- Use `<script setup lang="ts">` syntax for components
- Use i18n dictionary for UI text; no hardcoded text
- No emojis; use CSS-drawn alternatives
- JSON configuration input fields use concise placeholders without example text
- Run `npx vue-tsc --noEmit` to ensure type checking passes

### Rust

- Follow `rustfmt` default formatting
- Add Chinese comments for public functions (description, parameters, return values)
- Use `cargo check` to ensure compilation passes
- Use `cargo clippy` for linting

## Project Structure

```
McpStudio/
├── src/                    # Vue frontend source
│   ├── components/         # UI components
│   ├── i18n/               # i18n dictionaries
│   └── App.vue             # Main application
├── src-tauri/              # Tauri / Rust backend
│   ├── src/                # Rust source
│   │   ├── commands.rs     # Tauri commands
│   │   ├── mcp_client.rs   # MCP client
│   │   ├── mcp_server.rs   # MCP server
│   │   ├── models.rs       # Data models
│   │   └── storage.rs      # Persistent storage
│   ├── rust-sdk-rmcp/      # rmcp submodule
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .github/workflows/      # CI/CD workflows
└── package.json
```

## License

Contributed code will be licensed under the same [Apache License 2.0](LICENSE) as the project.

---

[中文文档](CONTRIBUTING.md)