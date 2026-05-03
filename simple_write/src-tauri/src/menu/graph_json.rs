use serde_json::{from_str, to_string};
use std::fs;
use std::path::{Path, PathBuf};

const JSON_PATH: &str = ".simple_write/graph.json";

fn graph_json_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(JSON_PATH)
}

fn ensure_graph_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = graph_json_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !json_path.exists() {
        fs::write(&json_path, "{}").map_err(|e| e.to_string())?;
    }
    Ok(json_path)
}

// 读取图谱配置（颜色组 + 节点颜色映射）
#[tauri::command]
pub fn read_graph_config(warehouse_path: String) -> Result<serde_json::Value, String> {
    let json_path = ensure_graph_storage(&warehouse_path)?;
    let json_string = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
    if json_string.trim().is_empty() {
        return Ok(serde_json::json!({ "colorGroups": [], "nodeColors": {} }));
    }
    from_str(&json_string).map_err(|e| e.to_string())
}

// 保存图谱配置
#[tauri::command]
pub fn write_graph_config(warehouse_path: String, config: String) -> Result<(), String> {
    let json_path = ensure_graph_storage(&warehouse_path)?;
    // 验证 JSON 格式
    let parsed: serde_json::Value = from_str(&config).map_err(|e| format!("无效的 JSON: {}", e))?;
    let formatted = to_string(&parsed).map_err(|e| e.to_string())?;
    fs::write(json_path, formatted).map_err(|e| e.to_string())
}
