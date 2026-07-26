use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// 工具 ID（唯一标识，通过 ID 引用时无需全名）
    pub id: Uuid,
    /// 工具名称
    pub name: String,
    /// 工具功能描述
    pub description: String,
    /// 参数 JSON Schema
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// 返回值说明
    #[serde(default)]
    pub returns: String,
}

impl Default for McpTool {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: String::new(),
            parameters: serde_json::Value::Null,
            returns: String::new(),
        }
    }
}

/// MCP 连接方式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionConfig {
    /// stdio 子进程方式连接
    #[serde(rename = "stdio")]
    Stdio {
        /// 启动命令（如 "npx", "uvx", "python" 等）
        command: String,
        /// 命令参数
        #[serde(default)]
        args: Vec<String>,
    },
    /// Streamable HTTP 方式连接
    #[serde(rename = "streamable_http")]
    StreamableHttp {
        /// MCP 服务端 HTTP URL（如 "http://localhost:8000/mcp"）
        url: String,
    },
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        ConnectionConfig::Stdio {
            command: String::new(),
            args: Vec::new(),
        }
    }
}

/// MCP 注册条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEntry {
    /// MCP ID（唯一标识，通过 ID 引用时无需全名）
    pub id: Uuid,
    /// MCP 名称
    pub name: String,
    /// MCP 描述
    pub description: String,
    /// 连接方式配置
    #[serde(default)]
    pub connection: ConnectionConfig,
    /// 该 MCP 提供的工具列表（连接后自动获取）
    #[serde(default)]
    pub tools: Vec<McpTool>,
    /// 连接状态
    #[serde(default)]
    pub connected: bool,
}

impl Default for McpEntry {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: String::new(),
            connection: ConnectionConfig::default(),
            tools: Vec::new(),
            connected: false,
        }
    }
}

/// 使用统计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// MCP 名称
    pub mcp_name: String,
    /// 工具名称
    pub tool_name: String,
    /// 使用次数
    pub count: u64,
    /// 最近使用时间（ISO 8601）
    pub last_used: String,
}

/// 服务端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP 服务端口
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    /// 启动时自动重连已注册的 MCP
    #[serde(default = "default_auto_connect")]
    pub auto_connect: bool,
}

fn default_http_port() -> u16 {
    9277
}

fn default_auto_connect() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 9277,
            auto_connect: true,
        }
    }
}

/// 工具查找结果
#[derive(Debug, Clone, Serialize)]
pub struct LookupResult {
    /// 所属 MCP ID
    pub mcp_id: Uuid,
    /// 所属 MCP 名称
    pub mcp_name: String,
    /// 所属 MCP 描述
    pub mcp_description: String,
    /// 工具 ID
    pub tool_id: Uuid,
    /// 工具名称
    pub tool_name: String,
    /// 工具描述
    pub tool_description: String,
    /// 参数 JSON Schema
    pub parameters: serde_json::Value,
    /// 返回值说明
    pub returns: String,
}

/// 工具调用结果（增强错误透传）
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallOutcome {
    /// 是否成功（传输层无异常）
    pub success: bool,
    /// 子 MCP 是否返回了错误（is_error）
    pub is_error: bool,
    /// 错误类型：none / transport / execution
    pub error_type: String,
    /// 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// 结果文本内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// 缓存条目元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 缓存 ID
    pub cache_id: Uuid,
    /// MCP ID
    pub mcp_id: Uuid,
    /// MCP 名称
    pub mcp_name: String,
    /// 工具 ID
    pub tool_id: Uuid,
    /// 工具名称
    pub tool_name: String,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 内容大小（字节）
    pub size: u64,
    /// 内容行数
    pub lines: usize,
}

/// 应用全局状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    /// MCP 注册表
    #[serde(default)]
    pub registry: Vec<McpEntry>,
    /// 使用统计
    #[serde(default)]
    pub usage: Vec<UsageRecord>,
    /// 服务端配置
    #[serde(default)]
    pub server_config: ServerConfig,
}

impl AppState {
    /// 记录工具使用，自动递增计数
    pub fn record_usage(&mut self, mcp_name: &str, tool_name: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(record) = self
            .usage
            .iter_mut()
            .find(|r| r.mcp_name == mcp_name && r.tool_name == tool_name)
        {
            record.count += 1;
            record.last_used = now;
        } else {
            self.usage.push(UsageRecord {
                mcp_name: mcp_name.to_string(),
                tool_name: tool_name.to_string(),
                count: 1,
                last_used: now,
            });
        }
    }

    /// 按使用次数降序获取最近使用列表
    pub fn get_recent_usage(&self, limit: Option<usize>) -> Vec<&UsageRecord> {
        let mut records: Vec<&UsageRecord> = self.usage.iter().collect();
        records.sort_by(|a, b| b.count.cmp(&a.count));
        if let Some(limit) = limit {
            records.truncate(limit);
        }
        records
    }

    /// 搜索 MCP（按名称或工具描述匹配，带相关性评分排序）
    pub fn search(&self, query: &str, limit: Option<usize>) -> Vec<&McpEntry> {
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(&McpEntry, u32)> = self
            .registry
            .iter()
            .filter_map(|entry| {
                let mut score = 0u32;
                // 5分：MCP 名称完全匹配
                if entry.name == query {
                    score = 5;
                // 4分：MCP 名称前缀匹配
                } else if entry.name.to_lowercase().starts_with(&query_lower) {
                    score = 4;
                // 3分：MCP 名称包含
                } else if entry.name.to_lowercase().contains(&query_lower) {
                    score = 3;
                // 2分：MCP 描述包含
                } else if entry.description.to_lowercase().contains(&query_lower) {
                    score = 2;
                }
                // 1分：工具名称或描述包含
                if score == 0 {
                    let tool_match = entry.tools.iter().any(|tool| {
                        tool.name.to_lowercase().contains(&query_lower)
                            || tool.description.to_lowercase().contains(&query_lower)
                    });
                    if tool_match {
                        score = 1;
                    }
                }
                if score > 0 {
                    Some((entry, score))
                } else {
                    None
                }
            })
            .collect();
        // 按分数降序排序
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        let mut results: Vec<&McpEntry> = scored.into_iter().map(|(entry, _)| entry).collect();
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        results
    }

    /// 按工具名精确查找，无结果时模糊匹配。返回匹配的工具及其所属 MCP 信息
    pub fn lookup_tool(&self, name: &str, limit: Option<usize>) -> Vec<LookupResult> {
        let name_lower = name.to_lowercase();
        let mut results: Vec<LookupResult> = Vec::new();

        // 先精确匹配
        for entry in &self.registry {
            for tool in &entry.tools {
                if tool.name == name {
                    results.push(LookupResult {
                        mcp_id: entry.id,
                        mcp_name: entry.name.clone(),
                        mcp_description: entry.description.clone(),
                        tool_id: tool.id,
                        tool_name: tool.name.clone(),
                        tool_description: tool.description.clone(),
                        parameters: tool.parameters.clone(),
                        returns: tool.returns.clone(),
                    });
                }
            }
        }

        // 无精确匹配时，降级为模糊匹配
        if results.is_empty() {
            for entry in &self.registry {
                for tool in &entry.tools {
                    if tool.name.to_lowercase().contains(&name_lower)
                        || tool.description.to_lowercase().contains(&name_lower)
                    {
                        results.push(LookupResult {
                            mcp_id: entry.id,
                            mcp_name: entry.name.clone(),
                            mcp_description: entry.description.clone(),
                            tool_id: tool.id,
                            tool_name: tool.name.clone(),
                            tool_description: tool.description.clone(),
                            parameters: tool.parameters.clone(),
                            returns: tool.returns.clone(),
                        });
                    }
                }
            }
        }

        if let Some(limit) = limit {
            results.truncate(limit);
        }
        results
    }

    /// 按工具名/描述搜索，返回匹配的工具及其所属 MCP 信息
    /// 与 lookup_tool 的区别：精确匹配优先，模糊匹配补充，按匹配质量排序
    pub fn search_tools(&self, query: &str, limit: Option<usize>) -> Vec<LookupResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<LookupResult> = Vec::new();

        // 精确匹配优先
        for entry in &self.registry {
            for tool in &entry.tools {
                if tool.name.to_lowercase() == query_lower {
                    results.push(LookupResult {
                        mcp_id: entry.id,
                        mcp_name: entry.name.clone(),
                        mcp_description: entry.description.clone(),
                        tool_id: tool.id,
                        tool_name: tool.name.clone(),
                        tool_description: tool.description.clone(),
                        parameters: tool.parameters.clone(),
                        returns: tool.returns.clone(),
                    });
                }
            }
        }

        // 模糊匹配补充
        for entry in &self.registry {
            for tool in &entry.tools {
                let name_lower = tool.name.to_lowercase();
                let desc_lower = tool.description.to_lowercase();
                // 跳过已经精确匹配的
                if name_lower == query_lower {
                    continue;
                }
                if name_lower.contains(&query_lower) || desc_lower.contains(&query_lower) {
                    results.push(LookupResult {
                        mcp_id: entry.id,
                        mcp_name: entry.name.clone(),
                        mcp_description: entry.description.clone(),
                        tool_id: tool.id,
                        tool_name: tool.name.clone(),
                        tool_description: tool.description.clone(),
                        parameters: tool.parameters.clone(),
                        returns: tool.returns.clone(),
                    });
                }
            }
        }

        if let Some(limit) = limit {
            results.truncate(limit);
        }
        results
    }
}