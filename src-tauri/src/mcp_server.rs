use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    transport::streamable_http_server::{StreamableHttpService, StreamableHttpServerConfig, session::local::LocalSessionManager},
    ServiceExt,
};
use tokio::sync::RwLock;

use crate::mcp_client;
use crate::models::{AppState, McpEntry, McpTool};
use crate::storage;

/// search_mcp 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    #[schemars(description = "搜索关键词，匹配 MCP 名称、描述或工具名称/描述")]
    pub query: String,
    #[schemars(description = "可选，返回结果的最大数量")]
    pub limit: Option<usize>,
}

/// query_mcp 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryMcpRequest {
    #[schemars(description = "要查询的 MCP ID 或名称")]
    pub mcp_id: String,
    #[schemars(description = "可选，要查询的工具 ID 或名称，不填则返回所有工具")]
    pub tool_id: Option<String>,
    #[schemars(description = "可选，分页偏移量（从 0 开始），仅在未指定 tool_id 时生效")]
    pub offset: Option<usize>,
    #[schemars(description = "可选，每页返回的工具数量，默认 10，仅在未指定 tool_id 时生效")]
    pub limit: Option<usize>,
}

/// list_recent_usage 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RecentUsageRequest {
    #[schemars(description = "可选，返回结果的最大数量")]
    pub limit: Option<usize>,
}

/// lookup_tool 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LookupToolRequest {
    #[schemars(description = "要查找的工具名称")]
    pub name: String,
    #[schemars(description = "可选，返回结果的最大数量")]
    pub limit: Option<usize>,
}

/// search_tools 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchToolsRequest {
    #[schemars(description = "搜索关键词，匹配工具名称或描述")]
    pub query: String,
    #[schemars(description = "可选，返回结果的最大数量")]
    pub limit: Option<usize>,
}

/// call_tool 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CallToolRequest {
    #[schemars(description = "要调用工具所属的 MCP ID 或名称")]
    pub mcp_id: String,
    #[schemars(description = "要调用的工具 ID 或名称")]
    pub tool_id: String,
    #[schemars(description = "调用工具的参数，JSON 对象格式，如 {\"key\": \"value\"}")]
    #[serde(default)]
    pub args: Option<serde_json::Value>,
    #[schemars(description = "可选，输出文件路径（绝对路径）。指定后结果写入文件而非内联返回，避免长结果被截断")]
    pub output_file: Option<String>,
}

/// batch_call_tools 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BatchCallToolRequest {
    #[schemars(description = "批量调用列表，每项包含 mcp_id、tool_id 和可选的 args、output_file")]
    pub calls: Vec<CallToolItem>,
}

/// 批量调用中的单个调用项
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CallToolItem {
    #[schemars(description = "MCP ID 或名称")]
    pub mcp_id: String,
    #[schemars(description = "工具 ID 或名称")]
    pub tool_id: String,
    #[schemars(description = "调用参数，JSON 对象格式")]
    #[serde(default)]
    pub args: Option<serde_json::Value>,
    #[schemars(description = "可选，输出文件路径（绝对路径）。指定后结果写入文件而非内联返回")]
    pub output_file: Option<String>,
}

/// read_cache 工具参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadCacheRequest {
    #[schemars(description = "缓存 ID（call_tool 返回的 cache_id）")]
    pub cache_id: String,
    #[schemars(description = "可选，分页起始行号（从 0 开始），默认 0")]
    pub offset: Option<usize>,
    #[schemars(description = "可选，每页行数，默认 100，最大 500")]
    pub limit: Option<usize>,
}

/// McpStudio MCP Server
#[derive(Clone)]
pub struct McpStudioServer {
    state: Arc<RwLock<AppState>>,
    tool_router: ToolRouter<McpStudioServer>,
}

/// 通过 ID 或名称查找 MCP 条目
fn find_mcp<'a>(registry: &'a [McpEntry], mcp_id: &str) -> Option<&'a McpEntry> {
    registry
        .iter()
        .find(|e| e.id.to_string() == mcp_id || e.name == mcp_id)
}

/// 通过 ID 或名称查找工具
fn find_tool<'a>(entry: &'a McpEntry, tool_id: &str) -> Option<&'a McpTool> {
    entry
        .tools
        .iter()
        .find(|t| t.id.to_string() == tool_id || t.name == tool_id)
}

/// 将结果文本写入文件，返回 (文件路径, 字节数)
fn write_result_to_file(path: &str, text: &str) -> Result<(String, u64), String> {
    let file_path = std::path::Path::new(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    std::fs::write(file_path, text)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    let size = std::fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);
    Ok((path.to_string(), size))
}

/// 智能解析 result 文本：若是合法 JSON 则解析为对象，否则保持字符串
fn smart_result_value(text: &Option<String>) -> serde_json::Value {
    match text {
        Some(t) => {
            match serde_json::from_str::<serde_json::Value>(t) {
                Ok(parsed) => parsed,
                Err(_) => serde_json::Value::String(t.clone()),
            }
        }
        None => serde_json::Value::Null,
    }
}

/// 构建 call_tool 返回的 JSON，处理缓存、output_file 和智能 result 解析
fn build_call_result_json(
    mcp_id: &str,
    mcp_name: &str,
    tool_id: &str,
    tool_name: &str,
    outcome: &crate::models::ToolCallOutcome,
    output_file: &Option<String>,
    cache_id: Option<uuid::Uuid>,
) -> serde_json::Value {
    let mut result = serde_json::json!({
        "mcp_id": mcp_id,
        "mcp_name": mcp_name,
        "tool_id": tool_id,
        "tool_name": tool_name,
        "success": outcome.success,
        "is_error": outcome.is_error,
        "error_type": outcome.error_type,
        "error_message": outcome.error_message,
    });

    // 写入缓存后的 ID
    if let Some(cid) = cache_id {
        result["cache_id"] = serde_json::json!(cid);
    }

    // 如果指定了 output_file，写入磁盘
    if let Some(ref path) = output_file {
        if let Some(ref text) = outcome.result {
            match write_result_to_file(path, text) {
                Ok((p, size)) => {
                    result["result"] = serde_json::json!(null);
                    result["output_file"] = serde_json::json!(p);
                    result["output_size"] = serde_json::json!(size);
                }
                Err(e) => {
                    result["result"] = smart_result_value(&outcome.result);
                    result["output_file_error"] = serde_json::json!(e);
                }
            }
        } else {
            result["result"] = serde_json::Value::Null;
        }
    } else {
        result["result"] = smart_result_value(&outcome.result);
    }

    result
}

#[tool_router]
impl McpStudioServer {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    /// 列出当前所有已连接的 MCP 工具列表，按 MCP 名字作为菜单组织
    #[tool(description = "列出当前所有已连接的 MCP 工具列表，按 MCP 名字作为菜单组织。返回每个 MCP 的名称、描述、工具数量和连接状态。")]
    async fn list_mcp_menu(&self) -> Result<CallToolResult, McpError> {
        let menu = {
            let app_state = self.state.read().await;
            app_state
                .registry
                .iter()
                .map(|entry| {
                    serde_json::json!({
                        "id": entry.id,
                        "name": entry.name,
                        "description": entry.description,
                        "tool_count": entry.tools.len(),
                        "connected": entry.connected
                    })
                })
                .collect::<Vec<_>>()
        };
        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage("McpStudio", "list_mcp_menu");
            storage::save_state(&app_state);
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&menu).unwrap_or_default(),
        )]))
    }

    /// 搜索 MCP 工具，可以通过名字搜索有关的 MCP，也可以直接搜索具体功能
    #[tool(description = "搜索 MCP 工具。可以通过名字搜索有关的 MCP，也可以直接搜索具体功能。支持可选的 limit 参数控制返回结果数量。")]
    async fn search_mcp(
        &self,
        Parameters(SearchRequest { query, limit }): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result_json = {
            let app_state = self.state.read().await;
            let results = app_state.search(&query, limit);
            results
                .iter()
                .map(|entry| {
                    serde_json::json!({
                        "id": entry.id,
                        "name": entry.name,
                        "description": entry.description,
                        "tool_count": entry.tools.len(),
                        "connected": entry.connected,
                        "tools": entry.tools.iter().map(|t| {
                            serde_json::json!({
                                "id": t.id,
                                "name": t.name,
                                "description": t.description,
                                "parameters": t.parameters,
                                "returns": t.returns
                            })
                        }).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>()
        };
        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage("McpStudio", "search_mcp");
            storage::save_state(&app_state);
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 查询指定 MCP 的详细信息
    #[tool(description = "查询指定 MCP 的详细信息，包括需要传入什么参数、怎么使用等。支持按 MCP ID 或名称查找，可选按工具 ID 或名称过滤单个工具。支持 offset/limit 分页避免大结果截断。")]
    async fn query_mcp(
        &self,
        Parameters(QueryMcpRequest { mcp_id, tool_id, offset, limit }): Parameters<QueryMcpRequest>,
    ) -> Result<CallToolResult, McpError> {
        let detail = {
            let app_state = self.state.read().await;
            let entry = find_mcp(&app_state.registry, &mcp_id)
                .ok_or_else(|| McpError::resource_not_found(
                    "mcp_not_found",
                    Some(serde_json::json!({"mcp_id": mcp_id})),
                ))?;

            let total_tools = entry.tools.len();

            // 如果指定了 tool_id，只返回匹配的工具，忽略分页
            let tools: Vec<&McpTool> = if let Some(ref tid) = tool_id {
                find_tool(entry, tid).into_iter().collect()
            } else {
                // 分页
                let page_size = limit.unwrap_or(10).min(50);
                let start = offset.unwrap_or(0);
                entry.tools.iter().skip(start).take(page_size).collect()
            };

            serde_json::json!({
                "id": entry.id,
                "name": entry.name,
                "description": entry.description,
                "connection": serde_json::to_value(&entry.connection).unwrap_or_default(),
                "connected": entry.connected,
                "total_tools": total_tools,
                "offset": offset.unwrap_or(0),
                "returned": tools.len(),
                "tools": tools.iter().map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                        "returns": t.returns
                    })
                }).collect::<Vec<_>>()
            })
        };
        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage("McpStudio", "query_mcp");
            storage::save_state(&app_state);
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&detail).unwrap_or_default(),
        )]))
    }

    /// 列出最近使用的 MCP 工具，按使用次数降序排序
    #[tool(description = "列出最近使用的 MCP 工具，按使用次数降序排序。支持可选的 limit 参数控制返回结果数量。")]
    async fn list_recent_usage(
        &self,
        Parameters(RecentUsageRequest { limit }): Parameters<RecentUsageRequest>,
    ) -> Result<CallToolResult, McpError> {
        let app_state = self.state.read().await;
        let records = app_state.get_recent_usage(limit);

        let result_json: Vec<serde_json::Value> = records
            .iter()
            .map(|r| {
                serde_json::json!({
                    "mcp_name": r.mcp_name,
                    "tool_name": r.tool_name,
                    "count": r.count,
                    "last_used": r.last_used
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 像字典查词一样，在所有已注册 MCP 中查找指定工具
    #[tool(description = "像字典查词一样，在所有已注册 MCP 中查找指定工具。先精确匹配工具名，无结果时模糊匹配。返回工具所属 MCP、描述、参数schema、返回值说明。")]
    async fn lookup_tool(
        &self,
        Parameters(LookupToolRequest { name, limit }): Parameters<LookupToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result_json = {
            let app_state = self.state.read().await;
            let results = app_state.lookup_tool(&name, limit);
            results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "mcp_id": r.mcp_id,
                        "mcp_name": r.mcp_name,
                        "mcp_description": r.mcp_description,
                        "tool_id": r.tool_id,
                        "tool_name": r.tool_name,
                        "tool_description": r.tool_description,
                        "parameters": r.parameters,
                        "returns": r.returns,
                    })
                })
                .collect::<Vec<_>>()
        };
        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage("McpStudio", "lookup_tool");
            storage::save_state(&app_state);
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 按工具名称或描述搜索工具，跨所有已注册 MCP
    #[tool(description = "按工具名称或描述搜索工具，跨所有已注册 MCP。精确匹配优先，模糊匹配补充。返回匹配的工具列表，每项包含所属 MCP 信息、工具 ID、描述、参数 schema。")]
    async fn search_tools(
        &self,
        Parameters(SearchToolsRequest { query, limit }): Parameters<SearchToolsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result_json = {
            let app_state = self.state.read().await;
            let results = app_state.search_tools(&query, limit);
            results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "mcp_id": r.mcp_id,
                        "mcp_name": r.mcp_name,
                        "mcp_description": r.mcp_description,
                        "tool_id": r.tool_id,
                        "tool_name": r.tool_name,
                        "tool_description": r.tool_description,
                        "parameters": r.parameters,
                        "returns": r.returns,
                    })
                })
                .collect::<Vec<_>>()
        };
        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage("McpStudio", "search_tools");
            storage::save_state(&app_state);
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 调用指定 MCP 的指定工具
    #[tool(description = "调用指定 MCP 的指定工具，传入参数并获取执行结果。结果自动缓存到磁盘，返回 cache_id 供后续分页读取。支持通过 ID 或名称指定 MCP 和工具。可指定 output_file 将长结果额外写入磁盘。")]
    async fn call_tool(
        &self,
        Parameters(CallToolRequest { mcp_id, tool_id, args, output_file }): Parameters<CallToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        // 获取连接配置和工具名称
        let (config, tool_name, mcp_name, entry_id, tool_uuid) = {
            let app_state = self.state.read().await;
            let entry = find_mcp(&app_state.registry, &mcp_id)
                .ok_or_else(|| McpError::resource_not_found(
                    "mcp_not_found",
                    Some(serde_json::json!({"mcp_id": mcp_id})),
                ))?;
            let tool = find_tool(entry, &tool_id)
                .ok_or_else(|| McpError::resource_not_found(
                    "tool_not_found",
                    Some(serde_json::json!({"tool_id": tool_id})),
                ))?;
            (entry.connection.clone(), tool.name.clone(), entry.name.clone(), entry.id, tool.id)
        };

        // 将 args 转为 Map
        let args_map = args.and_then(|v| {
            if let serde_json::Value::Object(map) = v {
                if map.is_empty() { None } else { Some(map) }
            } else {
                None
            }
        });

        // 调用工具（返回增强的错误信息）
        let outcome = mcp_client::call_tool(&config, &tool_name, args_map).await;

        // 自动缓存结果
        let cache_id = outcome.result.as_ref().map(|text| {
            let entry = storage::save_cache(entry_id, &mcp_name, tool_uuid, &tool_name, text);
            entry.cache_id
        });

        // 记录使用统计并持久化
        {
            let mut app_state = self.state.write().await;
            app_state.record_usage(&mcp_name, &tool_name);
            storage::save_state(&app_state);
        }

        let result_json = build_call_result_json(
            &mcp_id, &mcp_name, &tool_id, &tool_name, &outcome, &output_file, cache_id,
        );

        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 批量调用多个 MCP 的多个工具
    #[tool(description = "批量调用多个 MCP 的多个工具。传入 calls 数组，每项包含 mcp_id（MCP ID 或名称）、tool_id（工具 ID 或名称）和可选的 args。返回每个调用的结果列表。")]
    async fn batch_call_tools(
        &self,
        Parameters(BatchCallToolRequest { calls }): Parameters<BatchCallToolRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut results: Vec<serde_json::Value> = Vec::new();

        for call in &calls {
            // 获取连接配置和工具名称
            let (config, tool_name, mcp_name, entry_id, tool_uuid) = {
                let app_state = self.state.read().await;
                match find_mcp(&app_state.registry, &call.mcp_id) {
                    Some(entry) => {
                        match find_tool(entry, &call.tool_id) {
                            Some(tool) => (entry.connection.clone(), tool.name.clone(), entry.name.clone(), entry.id, tool.id),
                            None => {
                                results.push(serde_json::json!({
                                    "mcp_id": call.mcp_id,
                                    "tool_id": call.tool_id,
                                    "success": false,
                                    "error": format!("工具 '{}' 在 MCP '{}' 中未找到", call.tool_id, entry.name)
                                }));
                                continue;
                            }
                        }
                    }
                    None => {
                        results.push(serde_json::json!({
                            "mcp_id": call.mcp_id,
                            "tool_id": call.tool_id,
                            "success": false,
                            "error": format!("MCP '{}' 未找到", call.mcp_id)
                        }));
                        continue;
                    }
                }
            };

            let args_map = call.args.as_ref().and_then(|v| {
                if let serde_json::Value::Object(map) = v {
                    if map.is_empty() { None } else { Some(map.clone()) }
                } else {
                    None
                }
            });

            let outcome = mcp_client::call_tool(&config, &tool_name, args_map).await;

            // 自动缓存结果
            let cache_id = outcome.result.as_ref().map(|text| {
                let entry = storage::save_cache(entry_id, &mcp_name, tool_uuid, &tool_name, text);
                entry.cache_id
            });

            results.push(build_call_result_json(
                &call.mcp_id, &mcp_name, &call.tool_id, &tool_name, &outcome, &call.output_file, cache_id,
            ));

            // 记录使用统计
            {
                let mut app_state = self.state.write().await;
                app_state.record_usage(&mcp_name, &tool_name);
            }
        }

        // 持久化
        {
            let app_state = self.state.read().await;
            storage::save_state(&app_state);
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&results).unwrap_or_default(),
        )]))
    }

    /// 列出所有缓存条目，按时间倒序
    #[tool(description = "列出所有缓存条目，按创建时间倒序排列。返回缓存 ID、关联的 MCP/工具信息、大小、行数、创建时间。")]
    async fn list_cache(&self) -> Result<CallToolResult, McpError> {
        let entries = storage::list_cache_entries();
        let result_json: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "cache_id": e.cache_id,
                    "mcp_id": e.mcp_id,
                    "mcp_name": e.mcp_name,
                    "tool_id": e.tool_id,
                    "tool_name": e.tool_name,
                    "created_at": e.created_at,
                    "size": e.size,
                    "lines": e.lines,
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&result_json).unwrap_or_default(),
        )]))
    }

    /// 读取缓存内容，支持分页
    #[tool(description = "读取缓存内容，支持按行分页。传入 cache_id（由 call_tool 返回），可选 offset（起始行）和 limit（每页行数）。返回分页内容、总行数等元数据。")]
    async fn read_cache(
        &self,
        Parameters(ReadCacheRequest { cache_id, offset, limit }): Parameters<ReadCacheRequest>,
    ) -> Result<CallToolResult, McpError> {
        match storage::read_cache_content(&cache_id, offset, limit) {
            Ok((content, total, start, returned)) => {
                let result_json = serde_json::json!({
                    "cache_id": cache_id,
                    "offset": start,
                    "limit": limit.unwrap_or(100),
                    "returned": returned,
                    "total_lines": total,
                    "has_more": start + returned < total,
                    "content": content,
                });

                Ok(CallToolResult::success(vec![ContentBlock::text(
                    serde_json::to_string_pretty(&result_json).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::resource_not_found(
                "cache_not_found",
                Some(serde_json::json!({"cache_id": cache_id, "error": e})),
            )),
        }
    }
}

#[tool_handler]
impl ServerHandler for McpStudioServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_protocol_version(ProtocolVersion::V_2024_11_05)
        .with_instructions(
            "McpStudio - MCP 集市与路由中心。提供以下工具：list_mcp_menu（列出已连接 MCP）、search_mcp（搜索 MCP 及工具）、search_tools（跨 MCP 搜索工具）、query_mcp（查询 MCP 详情，支持分页和单工具过滤）、lookup_tool（按工具名查找工具）、call_tool（调用工具，自动缓存结果返回 cache_id）、batch_call_tools（批量调用多个工具）、list_cache（列出缓存条目）、read_cache（分页读取缓存内容）、list_recent_usage（最近使用统计）。".to_string(),
        )
    }
}

/// 启动 MCP Server，通过 stdio 与 AI 客户端通信
pub async fn run_stdio(state: Arc<RwLock<AppState>>) {
    let server = McpStudioServer::new(state);

    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("MCP Server 启动失败: {:?}", e);
        });

    if let Ok(service) = service {
        let _ = service.waiting().await;
    }
}

/// 启动 MCP Server，通过 HTTP 对外提供服务
pub async fn run_http(
    state: Arc<RwLock<AppState>>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ct = tokio_util::sync::CancellationToken::new();
    let state_clone = state.clone();

    let service = StreamableHttpService::new(
        move || {
            let server = McpStudioServer::new(state_clone.clone());
            Ok(server)
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );

    let addr = format!("127.0.0.1:{}", port);
    let router = axum::Router::new().nest_service("/mcp", service);

    let tcp_listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("HTTP MCP Server 已启动，监听地址: http://{}/mcp", addr);

    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            ct.cancelled().await;
        })
        .await?;

    Ok(())
}