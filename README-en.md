# McpStudio

<p align="center">
  <img src="https://img.shields.io/badge/License-Apache%202.0-blue?style=flat" alt="License" />
  <img src="https://img.shields.io/github/v/release/ZeroFxc/McpStudio?style=flat" alt="Release" />
  <img src="https://img.shields.io/github/stars/ZeroFxc/McpStudio?style=flat" alt="Stars" />
  <img src="https://img.shields.io/github/actions/workflow/status/ZeroFxc/McpStudio/ci.yml?style=flat&label=CI" alt="CI" />
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6?style=flat&logo=windows" alt="Platform" />
  <img src="https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri&logoColor=white" alt="Tauri" />
  <img src="https://img.shields.io/badge/Vue-3-4FC08D?style=flat&logo=vuedotjs&logoColor=white" alt="Vue" />
  <img src="https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white" alt="TypeScript" />
</p>

MCP Hub & Router Center — Connect AI to McpStudio and discover and use all registered MCP tools without connecting to each MCP server individually.

## Features

- **MCP Registration Management**: Supports both stdio and Streamable HTTP connections, with batch import via JSON configuration
- **Tool Discovery & Routing**: Automatically discovers tool lists from registered MCPs, providing dictionary-style search and precise lookup
- **Usage Statistics**: Records call count and last-used time for each tool
- **HTTP Service**: Built-in HTTP MCP Server (default port 9277), supporting remote AI connections
- **Marketplace Search**: Search and discover community MCP tools from GitHub (topics: mcp-server, modelcontextprotocol)
- **Internationalization**: Supports Chinese and English UI switching
- **Result Caching**: Automatic caching of tool call results with incremental read support

## Tech Stack

- **Desktop Framework**: [Tauri 2](https://tauri.app/)
- **Frontend**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/)
- **Backend**: [Rust](https://www.rust-lang.org/) + [rmcp](https://github.com/modelcontextprotocol/rust-sdk) (MCP Rust SDK)
- **HTTP Service**: [Axum](https://github.com/tokio-rs/axum)

## Dependencies & Acknowledgments

This project is built on the following open-source projects:

| Project | Purpose | License |
|---------|---------|---------|
| [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) | MCP protocol Rust implementation | Apache-2.0 / MIT |
| [Tauri](https://github.com/tauri-apps/tauri) | Cross-platform desktop framework | Apache-2.0 / MIT |
| [Vue.js](https://github.com/vuejs/core) | Frontend UI framework | MIT |
| [Axum](https://github.com/tokio-rs/axum) | Rust web framework | MIT |
| [Tokio](https://github.com/tokio-rs/tokio) | Rust async runtime | MIT |

## Quick Start

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

### Build for Production

```bash
npm run tauri build
```

Build artifacts are located in the `src-tauri/target/release/` directory.

## Directory Structure

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
├── package.json
└── README.md
```

## Documentation

- [Contributing Guide](CONTRIBUTING-en.md)
- [Code of Conduct](CODE_OF_CONDUCT-en.md)
- [Changelog](CHANGELOG-en.md)
- [Security Policy](SECURITY-en.md)

## License

This project is open-sourced under the [Apache License 2.0](LICENSE).

The rmcp submodule ([modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)) used in this project is dual-licensed under Apache-2.0 / MIT.

---

[中文文档](README.md)