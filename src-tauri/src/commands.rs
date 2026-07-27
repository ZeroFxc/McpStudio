use tauri::State;

use crate::mcp_client;
use crate::models::{ConnectionConfig, McpEntry, ServerConfig};
use crate::storage;
use crate::SharedState;

/// 添加 MCP 连接（注册到系统）
#[tauri::command]
pub async fn add_mcp(
    state: State<'_, SharedState>,
    name: String,
    description: String,
    connection: ConnectionConfig,
) -> Result<McpEntry, String> {
    let mut app_state = state.state.write().await;

    // 检查是否已存在同名 MCP
    if app_state.registry.iter().any(|e| e.name == name) {
        return Err(format!("MCP '{}' 已存在", name));
    }

    let entry = McpEntry {
        id: uuid::Uuid::new_v4(),
        name: name.clone(),
        description,
        connection,
        tools: Vec::new(),
        connected: false,
    };
    app_state.registry.push(entry.clone());

    // 持久化
    storage::save_state(&app_state);

    Ok(entry)
}

/// 连接到 MCP 服务器并自动发现工具
#[tauri::command]
pub async fn connect_mcp(
    state: State<'_, SharedState>,
    name: String,
) -> Result<McpEntry, String> {
    // 先获取连接配置（释放读锁后再获取写锁）
    let config = {
        let app_state = state.state.read().await;
        let entry = app_state
            .registry
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| format!("MCP '{}' 未找到", name))?;
        entry.connection.clone()
    };

    // 连接并发现工具
    let tools = mcp_client::discover_tools(&config).await?;

    // 更新注册表中的工具列表和连接状态
    let entry = {
        let mut app_state = state.state.write().await;
        let entry = app_state
            .registry
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| format!("MCP '{}' 未找到", name))?;

        entry.tools = tools;
        entry.connected = true;
        entry.clone()
    };

    // 持久化
    {
        let app_state = state.state.read().await;
        storage::save_state(&app_state);
    }

    Ok(entry)
}

/// 断开 MCP 连接
#[tauri::command]
pub async fn disconnect_mcp(
    state: State<'_, SharedState>,
    name: String,
) -> Result<(), String> {
    let mut app_state = state.state.write().await;

    let entry = app_state
        .registry
        .iter_mut()
        .find(|e| e.name == name)
        .ok_or_else(|| format!("MCP '{}' 未找到", name))?;

    entry.connected = false;

    storage::save_state(&app_state);

    Ok(())
}

/// 删除 MCP 连接
#[tauri::command]
pub async fn remove_mcp(
    state: State<'_, SharedState>,
    name: String,
) -> Result<(), String> {
    let mut app_state = state.state.write().await;

    let index = app_state
        .registry
        .iter()
        .position(|e| e.name == name)
        .ok_or_else(|| format!("MCP '{}' 未找到", name))?;

    app_state.registry.remove(index);

    // 持久化
    storage::save_state(&app_state);

    Ok(())
}

/// 列出所有已注册 MCP
#[tauri::command]
pub async fn list_mcp(state: State<'_, SharedState>) -> Result<Vec<McpEntry>, String> {
    let app_state = state.state.read().await;
    Ok(app_state.registry.clone())
}

/// 获取使用统计
#[tauri::command]
pub async fn get_usage_stats(
    state: State<'_, SharedState>,
    limit: Option<usize>,
) -> Result<Vec<crate::models::UsageRecord>, String> {
    let app_state = state.state.read().await;
    let records: Vec<_> = app_state
        .get_recent_usage(limit)
        .into_iter()
        .cloned()
        .collect();
    Ok(records)
}

/// 获取服务端配置
#[tauri::command]
pub async fn get_server_config(
    state: State<'_, SharedState>,
) -> Result<ServerConfig, String> {
    let app_state = state.state.read().await;
    Ok(app_state.server_config.clone())
}

/// 设置 HTTP 端口
#[tauri::command]
pub async fn set_http_port(
    state: State<'_, SharedState>,
    port: u16,
) -> Result<(), String> {
    let mut app_state = state.state.write().await;
    app_state.server_config.http_port = port;
    storage::save_state(&app_state);
    Ok(())
}

/// 设置 HTTP 绑定地址
#[tauri::command]
pub async fn set_bind_address(
    state: State<'_, SharedState>,
    address: String,
) -> Result<(), String> {
    let mut app_state = state.state.write().await;
    app_state.server_config.bind_address = address;
    storage::save_state(&app_state);
    Ok(())
}

/// 在系统文件管理器中打开数据目录
#[tauri::command]
pub async fn open_data_dir() -> Result<(), String> {
    let dir = storage::data_dir();
    // 确保目录存在
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建目录: {}", e))?;
    opener::open(&dir).map_err(|e| format!("无法打开目录: {}", e))?;
    Ok(())
}

/// 获取本机局域网 IP 地址列表（非回环 IPv4）
#[tauri::command]
pub async fn get_local_ips() -> Result<Vec<String>, String> {
    let ifaces = get_if_addrs::get_if_addrs()
        .map_err(|e| format!("获取网络接口失败: {}", e))?;
    let mut ips: Vec<String> = ifaces
        .iter()
        .filter_map(|iface| {
            if iface.is_loopback() {
                return None;
            }
            match iface.addr {
                get_if_addrs::IfAddr::V4(ref v4) => Some(v4.ip.to_string()),
                _ => None,
            }
        })
        .collect();
    // 排序保证稳定输出
    ips.sort();
    ips.dedup();
    Ok(ips)
}

/// 环境检测结果
#[derive(serde::Serialize)]
pub struct EnvCheckResult {
    pub installed: bool,
    pub version: String,
}

/// 检测指定命令是否存在并获取版本号
/// 优先使用 `--version`，失败时回退到 `-v` 或 `-V`
#[tauri::command]
pub async fn check_env(command: String) -> Result<EnvCheckResult, String> {
    // 先检查命令是否存在（Windows 用 where，Unix 用 which）
    #[cfg(target_os = "windows")]
    let check_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let check_cmd = "which";

    let exists = std::process::Command::new(check_cmd)
        .arg(&command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !exists {
        return Ok(EnvCheckResult {
            installed: false,
            version: String::new(),
        });
    }

    // 获取版本号，依次尝试 --version、-v、-V
    let version_flags = ["--version", "-v", "-V"];
    let mut version = String::new();
    for flag in &version_flags {
        if let Ok(output) = std::process::Command::new(&command)
            .arg(flag)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr).trim().to_string();
            if !combined.is_empty() {
                // 取第一行作为版本号
                version = combined.lines().next().unwrap_or("").to_string();
                break;
            }
        }
    }

    Ok(EnvCheckResult {
        installed: true,
        version,
    })
}