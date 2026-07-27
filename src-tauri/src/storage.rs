use std::fs;
use std::path::PathBuf;

use crate::models::{AppState, CacheEntry};
use uuid::Uuid;

/// 获取数据目录路径（用于获取配置文件目录）
fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "android")]
    {
        // Android 使用 app 私有内部存储目录
        // HOME 环境变量在 Android 上通常指向 /data/data/<package>
        if let Ok(home) = std::env::var("HOME") {
            let mut p = PathBuf::from(&home);
            p.push("files");
            return Some(p);
        }
        // 兜底：硬编码包名路径
        Some(PathBuf::from("/data/data/com.nirithy.mcpstudio/files"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        std::env::var("HOME").ok().map(|h| {
            let mut p = PathBuf::from(h);
            p.push(".local");
            p.push("share");
            p
        })
    }
}

/// 获取数据目录路径
pub fn data_dir() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| dirs_next().unwrap_or_else(|| PathBuf::from(".")));
    path.push("McpStudio");
    path
}

/// 获取数据文件路径
fn data_path() -> PathBuf {
    let mut path = data_dir();
    fs::create_dir_all(&path).ok();
    path.push("data.json");
    path
}

/// 从文件加载应用状态
pub fn load_state() -> AppState {
    let path = data_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AppState::default(),
        }
    } else {
        AppState::default()
    }
}

/// 保存应用状态到文件
pub fn save_state(state: &AppState) {
    let path = data_path();
    if let Ok(json) = serde_json::to_string_pretty(state) {
        fs::write(&path, json).ok();
    }
}

/// 获取缓存目录路径
fn cache_dir() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push("McpStudio");
    path.push("cache");
    fs::create_dir_all(&path).ok();
    path
}

/// 获取缓存元数据索引文件路径
fn cache_index_path() -> PathBuf {
    let mut path = cache_dir();
    path.push("_index.json");
    path
}

/// 加载缓存索引
fn load_cache_index() -> Vec<CacheEntry> {
    let path = cache_index_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

/// 保存缓存索引
fn save_cache_index(index: &[CacheEntry]) {
    let path = cache_index_path();
    if let Ok(json) = serde_json::to_string_pretty(index) {
        fs::write(&path, json).ok();
    }
}

/// 将工具调用结果保存到缓存，返回缓存条目
pub fn save_cache(
    mcp_id: Uuid,
    mcp_name: &str,
    tool_id: Uuid,
    tool_name: &str,
    content: &str,
) -> CacheEntry {
    let cache_id = Uuid::new_v4();
    let dir = cache_dir();
    let file_path = dir.join(format!("{}.json", cache_id));

    let lines = content.lines().count();
    let size = content.len() as u64;
    let created_at = chrono::Utc::now().to_rfc3339();

    // 写入内容文件
    fs::write(&file_path, content).ok();

    // 更新索引
    let entry = CacheEntry {
        cache_id,
        mcp_id,
        mcp_name: mcp_name.to_string(),
        tool_id,
        tool_name: tool_name.to_string(),
        created_at: created_at.clone(),
        size,
        lines,
    };

    let mut index = load_cache_index();
    index.push(entry.clone());
    // 只保留最近 200 条缓存
    if index.len() > 200 {
        let remove_count = index.len() - 200;
        // 删除最旧的缓存文件
        for old in index.iter().take(remove_count) {
            let old_path = dir.join(format!("{}.json", old.cache_id));
            fs::remove_file(&old_path).ok();
        }
        index.drain(0..remove_count);
    }
    save_cache_index(&index);

    entry
}

/// 列出所有缓存条目（按时间倒序）
pub fn list_cache_entries() -> Vec<CacheEntry> {
    let mut index = load_cache_index();
    index.reverse(); // 最新的在前
    index
}

/// 读取缓存内容（支持分页，按行）
/// 返回 (内容字符串, 总行数, 起始行, 返回行数)
pub fn read_cache_content(
    cache_id: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<(String, usize, usize, usize), String> {
    let dir = cache_dir();
    let file_path = dir.join(format!("{}.json", cache_id));

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取缓存文件失败: {}", e))?;

    let all_lines: Vec<&str> = content.lines().collect();
    let total = all_lines.len();
    let start = offset.unwrap_or(0);
    let page_size = limit.unwrap_or(100).min(500);
    let end = (start + page_size).min(total);
    let sliced: Vec<&str> = all_lines[start..end].to_vec();
    let returned = sliced.len();

    Ok((sliced.join("\n"), total, start, returned))
}