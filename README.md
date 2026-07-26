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

MCP 集市与路由中心 —— AI 通过连接 McpStudio 即可发现和使用所有已注册的 MCP 工具，无需逐个连接每个 MCP 服务器。

## 功能特性

- **MCP 注册管理**：支持 stdio 和 Streamable HTTP 两种连接方式，批量导入 JSON 配置
- **工具发现与路由**：自动发现已注册 MCP 的工具列表，提供字典式搜索和精确查找
- **使用统计**：记录每个工具的调用次数和最近使用时间
- **HTTP 服务**：内置 HTTP MCP Server（默认端口 9277），支持远程 AI 连接
- **市场搜索**：从 GitHub 搜索和发现社区 MCP 工具（话题：mcp-server、modelcontextprotocol）
- **国际化**：支持中文和英文界面切换
- **结果缓存**：工具调用结果自动缓存，支持增量读取

## 技术栈

| 类型 | 技术 |
|------|------|
| 桌面框架 | [Tauri 2](https://tauri.app/) |
| 前端 | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) |
| 后端 | [Rust](https://www.rust-lang.org/) + [rmcp](https://github.com/modelcontextprotocol/rust-sdk)（MCP Rust SDK） |
| HTTP 服务 | [Axum](https://github.com/tokio-rs/axum) |

## 依赖与致谢

本项目基于以下开源项目构建：

| 项目 | 用途 | 许可证 |
|------|------|--------|
| [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) | MCP 协议 Rust 实现 | Apache-2.0 / MIT |
| [Tauri](https://github.com/tauri-apps/tauri) | 跨平台桌面框架 | Apache-2.0 / MIT |
| [Vue.js](https://github.com/vuejs/core) | 前端 UI 框架 | MIT |
| [Axum](https://github.com/tokio-rs/axum) | Rust Web 框架 | MIT |
| [Tokio](https://github.com/tokio-rs/tokio) | Rust 异步运行时 | MIT |

## 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- Windows 需要 [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### 克隆仓库

```bash
git clone --recurse-submodules git@github.com:ZeroFxc/McpStudio.git
cd McpStudio
```

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布版

```bash
npm run tauri build
```

构建产物在 `src-tauri/target/release/` 目录下。

## 目录结构

```
McpStudio/
├── src/                    # Vue 前端源码
│   ├── components/         # UI 组件
│   ├── i18n/               # 国际化字典
│   └── App.vue             # 主应用
├── src-tauri/              # Tauri / Rust 后端
│   ├── src/                # Rust 源码
│   │   ├── commands.rs     # Tauri 命令
│   │   ├── mcp_client.rs   # MCP 客户端
│   │   ├── mcp_server.rs   # MCP 服务端
│   │   ├── models.rs       # 数据模型
│   │   └── storage.rs      # 持久化存储
│   ├── rust-sdk-rmcp/      # rmcp 子模块
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .github/workflows/      # CI/CD 工作流
├── package.json
└── README.md
```

## 文档

- [贡献指南](CONTRIBUTING.md)
- [行为准则](CODE_OF_CONDUCT.md)
- [变更日志](CHANGELOG.md)
- [安全策略](SECURITY.md)

## 许可证

本项目代码基于 [Apache License 2.0](LICENSE) 开源。

本项目使用的 rmcp 子模块（[modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)）采用 Apache-2.0 / MIT 双许可证。

---

[English](README-en.md)