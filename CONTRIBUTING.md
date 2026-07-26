# 贡献指南

感谢你对 McpStudio 的关注！我们欢迎任何形式的贡献。

## 行为准则

本项目遵循 [Contributor Covenant 行为准则](CODE_OF_CONDUCT.md)。参与即表示你同意遵守其条款。

## 如何贡献

### 报告 Bug

如果你发现了 Bug，请通过 GitHub Issues 提交，并包含以下信息：

- **标题**：简明描述问题
- **描述**：详细说明 Bug 的表现
- **复现步骤**：如何触发该 Bug
- **期望行为**：你期望的正常行为
- **实际行为**：实际发生的情况
- **环境信息**：操作系统、Node.js 版本、Rust 版本
- **截图/日志**（如有）：附上相关截图或错误日志

### 功能请求

如果你有功能建议，请通过 GitHub Issues 提交，并说明：

- **需求背景**：为什么需要这个功能
- **功能描述**：你期望的功能是什么样的
- **替代方案**：你考虑过的其他方案

### 提交 Pull Request

1. **Fork 本仓库**，然后克隆到本地
2. **创建功能分支**：`git checkout -b feat/your-feature-name`
3. **进行开发**，遵循代码风格指南
4. **提交代码**：遵循 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/) 规范
5. **推送到你的 Fork**：`git push origin feat/your-feature-name`
6. **创建 Pull Request**，描述你的变更

#### 分支命名规范

- `feat/xxx` — 新功能
- `fix/xxx` — Bug 修复
- `docs/xxx` — 文档更新
- `refactor/xxx` — 代码重构
- `chore/xxx` — 构建/工具变更

#### 提交信息规范

遵循 Conventional Commits 格式：

```
<type>(<scope>): <description>

[optional body]
```

类型（type）：
- `feat`：新功能
- `fix`：Bug 修复
- `docs`：文档变更
- `style`：代码格式（不影响代码逻辑）
- `refactor`：代码重构
- `perf`：性能优化
- `test`：测试相关
- `chore`：构建/工具变更

## 开发环境搭建

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

### 构建

```bash
npm run tauri build
```

## 代码风格指南

### Vue / TypeScript

- 使用 TypeScript 严格模式
- 组件使用 `<script setup lang="ts">` 语法
- UI 文本使用 i18n 字典，禁止硬编码
- 禁止使用 emoji，用 CSS 绘制替代
- JSON 配置输入框使用简洁占位符，不加示例文字
- 运行 `npx vue-tsc --noEmit` 确保类型检查通过

### Rust

- 遵循 `rustfmt` 默认格式
- 公开函数添加中文注释（功能描述、参数说明、返回值）
- 使用 `cargo check` 确保编译通过
- 使用 `cargo clippy` 进行 lint 检查

## 项目结构

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
└── package.json
```

## 许可证

贡献的代码将采用与项目相同的 [Apache License 2.0](LICENSE) 许可证。

---

[English](CONTRIBUTING-en.md)