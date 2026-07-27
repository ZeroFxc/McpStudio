use std::sync::Arc;

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, CallToolResult, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
#[cfg(not(target_os = "android"))]
use rmcp::transport::TokioChildProcess;
#[cfg(not(target_os = "android"))]
use tokio::process::Command;

use crate::models::{ConnectionConfig, McpTool, ToolCallOutcome};

/// 连接到 MCP 服务器并获取其工具列表
/// 连接后立即获取工具列表，然后断开连接
pub async fn discover_tools(config: &ConnectionConfig) -> Result<Vec<McpTool>, String> {
    match config {
        #[cfg(not(target_os = "android"))]
        ConnectionConfig::Stdio { command, args } => discover_stdio(command, args).await,
        ConnectionConfig::StreamableHttp { url } => discover_http(url).await,
        #[cfg(target_os = "android")]
        ConnectionConfig::Stdio { .. } => Err("stdio 连接在 Android 上不可用，请使用 Streamable HTTP".to_string()),
    }
}

/// 将 rmcp 的 Tool 转换为 McpTool
fn convert_tool(t: rmcp::model::Tool) -> McpTool {
    McpTool {
        id: uuid::Uuid::new_v4(),
        name: t.name.into_owned(),
        description: t.description.unwrap_or_default().into_owned(),
        parameters: serde_json::Value::Object(
            Arc::unwrap_or_clone(t.input_schema)
        ),
        returns: String::new(),
    }
}
/// 通过 stdio 子进程连接并发现工具（仅桌面端）
#[cfg(not(target_os = "android"))]
async fn discover_stdio(command: &str, args: &[String]) -> Result<Vec<McpTool>, String> {
    let mut cmd = Command::new(command);
    for arg in args {
        cmd.arg(arg);
    }

    let transport = TokioChildProcess::new(cmd)
        .map_err(|e| format!("创建子进程传输失败: {}", e))?;

    let client = ()
        .serve(transport)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 获取工具列表（list_all_tools 返回 Vec<Tool>）
    let tools = client
        .list_all_tools()
        .await
        .map_err(|e| format!("获取工具列表失败: {}", e))?;

    // 转换为 McpTool
    let mcp_tools: Vec<McpTool> = tools
        .into_iter()
        .map(convert_tool)
        .collect();

    // 断开连接
    client.cancel().await.map_err(|e| format!("断开连接失败: {}", e))?;

    Ok(mcp_tools)
}

/// 通过 Streamable HTTP 连接并发现工具
async fn discover_http(url: &str) -> Result<Vec<McpTool>, String> {
    let transport = StreamableHttpClientTransport::from_uri(url);

    let client_info = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("McpStudio", "0.1.0"),
    );

    let client = client_info
        .serve(transport)
        .await
        .map_err(|e| format!("HTTP 连接失败: {}", e))?;

    // 获取工具列表
    let tools = client
        .list_all_tools()
        .await
        .map_err(|e| format!("获取工具列表失败: {}", e))?;

    // 转换为 McpTool
    let mcp_tools: Vec<McpTool> = tools
        .into_iter()
        .map(convert_tool)
        .collect();

    // 断开连接
    client.cancel().await.map_err(|e| format!("断开连接失败: {}", e))?;

    Ok(mcp_tools)
}

/// 调用指定 MCP 的工具并返回结果（增强错误透传）
pub async fn call_tool(
    config: &ConnectionConfig,
    tool_name: &str,
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> ToolCallOutcome {
    let raw_result = match config {
        #[cfg(not(target_os = "android"))]
        ConnectionConfig::Stdio { command, args } => {
            call_tool_stdio(command, args, tool_name, arguments).await
        }
        ConnectionConfig::StreamableHttp { url } => {
            call_tool_http(url, tool_name, arguments).await
        }
        #[cfg(target_os = "android")]
        ConnectionConfig::Stdio { .. } => {
            Err("stdio 连接在 Android 上不可用".to_string())
        }
    };

    match raw_result {
        Ok(call_result) => {
            // 提取文本内容
            let text = call_result.content.iter()
                .filter_map(|block| {
                    if let rmcp::model::ContentBlock::Text(text) = block {
                        Some(text.text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            if call_result.is_error.unwrap_or(false) {
                ToolCallOutcome {
                    success: false,
                    is_error: true,
                    error_type: "execution".to_string(),
                    error_message: Some(if text.is_empty() {
                        "子 MCP 工具返回了错误，但未提供错误详情".to_string()
                    } else {
                        text.clone()
                    }),
                    result: Some(text),
                }
            } else {
                ToolCallOutcome {
                    success: true,
                    is_error: false,
                    error_type: "none".to_string(),
                    error_message: None,
                    result: Some(text),
                }
            }
        }
        Err(e) => ToolCallOutcome {
            success: false,
            is_error: false,
            error_type: "transport".to_string(),
            error_message: Some(e),
            result: None,
        },
    }
}

/// 通过 stdio 连接调用工具（仅桌面端）
#[cfg(not(target_os = "android"))]
async fn call_tool_stdio(
    command: &str,
    args: &[String],
    tool_name: &str,
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, String> {
    let mut cmd = Command::new(command);
    for arg in args {
        cmd.arg(arg);
    }
    let transport = TokioChildProcess::new(cmd)
        .map_err(|e| format!("创建子进程失败: {}", e))?;
    let client = ()
        .serve(transport)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    let mut params = CallToolRequestParams::new(tool_name.to_string());
    if let Some(args) = arguments {
        params = params.with_arguments(args);
    }
    let result = client
        .call_tool(params)
        .await
        .map_err(|e| format!("工具调用失败: {}", e))?;
    client.cancel().await.ok();
    Ok(result)
}

/// 通过 HTTP 连接调用工具
async fn call_tool_http(
    url: &str,
    tool_name: &str,
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, String> {
    let transport = StreamableHttpClientTransport::from_uri(url);
    let client_info = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("McpStudio", "0.1.0"),
    );
    let client = client_info
        .serve(transport)
        .await
        .map_err(|e| format!("HTTP 连接失败: {}", e))?;
    let mut params = CallToolRequestParams::new(tool_name.to_string());
    if let Some(args) = arguments {
        params = params.with_arguments(args);
    }
    let result = client
        .call_tool(params)
        .await
        .map_err(|e| format!("工具调用失败: {}", e))?;
    client.cancel().await.ok();
    Ok(result)
}