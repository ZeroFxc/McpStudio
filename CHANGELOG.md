# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2026-07-28

## [0.1.4] - 2026-07-28

## [0.1.3] - 2026-07-28

## [0.1.2] - 2026-07-27

### Added
- 环境检测页面：检测 Python、uv、Node.js、npm、Git、pip 等 MCP 依赖
- 市场预览功能：GitHub 仓库搜索、文件列表、README 渲染
- 主题系统：暗色/亮色主题切换，CSS 变量驱动全局配色
- 版本号过长自动截断，支持展开详情和 hover 浮动提示

### Fixed
- 修复复制按钮点击无响应（事件委托替代逐个绑定）
- 修复标签页语言切换不生效（computed 属性动态响应 i18n）
- 修复最大化按钮图标切换延迟（手动状态切换 + 防抖 onResized）

## [0.1.1] - 2026-07-27

### Added
- 工作流缓存优化：npm 依赖缓存和 Rust 缓存共享

### Fixed
- 修复 Tauri 2 窗口操作权限缺失

## [0.1.0] - 2026-07-27

### Added
- MCP 注册管理：支持 stdio 和 Streamable HTTP 两种连接方式
- 工具发现与路由：自动发现已注册 MCP 的工具列表，支持字典式搜索
- 使用统计：记录每个工具的调用次数和最近使用时间
- HTTP 服务：内置 HTTP MCP Server（默认端口 9277）
- 市场搜索：从 GitHub 搜索社区 MCP 工具
- 国际化：支持中文和英文界面切换
- 结果缓存：工具调用结果自动缓存，支持增量读取
- 设置页面：语言切换、存储目录管理
- CI/CD 工作流：自动编译检查和 Release 发布
- 工作流缓存：npm 和 Rust 依赖缓存

[Unreleased]: https://github.com/ZeroFxc/McpStudio/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.5
[0.1.4]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.4
[0.1.3]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.3
[0.1.2]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.2
[0.1.1]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.1
[0.1.0]: https://github.com/ZeroFxc/McpStudio/releases/tag/v0.1.0

---

[English](CHANGELOG-en.md)