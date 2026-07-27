mod commands;
mod mcp_client;
mod mcp_server;
pub mod models;
mod storage;

use std::sync::Arc;
use tokio::sync::RwLock;

use models::AppState;
use models::ConnectionConfig;

/// 全局应用状态，Tauri 和 MCP Server 共享
pub struct SharedState {
    pub state: Arc<RwLock<AppState>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 tracing（全局只需一次）
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // 从文件加载持久化状态
    let app_state = storage::load_state();
    let shared = Arc::new(RwLock::new(app_state));

    // 启动时自动重连已注册的 MCP
    let auto_connect = {
        let app_state_guard = shared.blocking_read();
        app_state_guard.server_config.auto_connect
    };
    if auto_connect {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app_state_guard = shared.blocking_write();
        let entries: Vec<(String, ConnectionConfig)> = app_state_guard
            .registry
            .iter()
            .map(|e| (e.name.clone(), e.connection.clone()))
            .collect();
        for (name, config) in entries {
            match rt.block_on(mcp_client::discover_tools(&config)) {
                Ok(tools) => {
                    if let Some(entry) = app_state_guard.registry.iter_mut().find(|e| e.name == name) {
                        entry.tools = tools;
                        entry.connected = true;
                    }
                }
                Err(e) => {
                    tracing::warn!("自动重连 MCP '{}' 失败: {}", name, e);
                }
            }
        }
        storage::save_state(&app_state_guard);
    }

    // 克隆一份引用给 MCP Server 线程使用
    let mcp_state = shared.clone();

    // 启动 MCP Server（在独立线程中运行，通过 stdio 通信）
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            mcp_server::run_stdio(mcp_state).await;
        });
    });

    // 启动 HTTP MCP Server
    let http_state = shared.clone();
    let http_port = {
        let app_state_guard = shared.blocking_read();
        app_state_guard.server_config.http_port
    };
    let bind_address = {
        let app_state_guard = shared.blocking_read();
        app_state_guard.server_config.bind_address.clone()
    };
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = mcp_server::run_http(http_state, &bind_address, http_port).await {
                tracing::error!("HTTP MCP Server 启动失败: {:?}", e);
            }
        });
    });

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    // 桌面端：注册 autostart 插件
    #[cfg(not(target_os = "android"))]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ));
    }

    builder
        .manage(SharedState { state: shared })
        .invoke_handler(tauri::generate_handler![
            commands::add_mcp,
            commands::connect_mcp,
            commands::disconnect_mcp,
            commands::remove_mcp,
            commands::list_mcp,
            commands::get_usage_stats,
            commands::get_server_config,
            commands::set_http_port,
            commands::set_bind_address,
            commands::get_local_ips,
            commands::open_data_dir,
            commands::get_platform,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}