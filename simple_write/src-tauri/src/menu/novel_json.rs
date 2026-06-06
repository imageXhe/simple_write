use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Value};
use std::fs;
use std::path::{Path, PathBuf};

const NOVEL_CONFIG_PATH: &str = ".simple_write/novel-config.json";
const TXT_META_PATH: &str = ".simple_write/txt-meta.json";

// ---- 小说写作配置 ----

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InsertType {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub text_color: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkType {
    pub id: String,
    pub name: String,
    pub bg_color: String,
    pub text_color: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportDefaults {
    pub include_file_name_as_chapter: bool,
    pub include_folder_name_as_volume: bool,
    pub blank_line_between_sections: bool,
    pub output_encoding: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GraphStyle {
    pub direction: String,
    pub folder_node_color: String,
    pub folder_node_border: String,
    pub folder_node_font_size: u32,
    pub file_node_color: String,
    pub file_node_border: String,
    pub file_node_font_size: u32,
    pub edge_color: String,
    pub edge_width: u32,
    pub show_arrows: bool,
    pub node_gap: u32,
    pub level_gap: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NovelConfig {
    pub insert_types: Vec<InsertType>,
    pub mark_types: Vec<MarkType>,
    pub export_defaults: ExportDefaults,
    pub graph_style: GraphStyle,
}

// ---- txt 元数据（已导出给前端使用，Rust 端通过 JSON Value 间接操作） ----

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
    pub utf16_offset: usize,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InsertItem {
    pub id: String,
    pub type_id: String,
    pub anchor: TextPosition,
    pub payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_file_path: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkItem {
    pub id: String,
    pub type_id: String,
    pub range: TextRange,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileMeta {
    pub inserts: Vec<InsertItem>,
    pub marks: Vec<MarkItem>,
}

// ---- 文件路径工具 ----

fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

// ---- novel-config.json ----

fn novel_config_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(NOVEL_CONFIG_PATH)
}

fn ensure_novel_config_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = novel_config_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(json_path)
}

fn default_novel_config() -> NovelConfig {
    NovelConfig {
        insert_types: vec![
            InsertType {
                id: "hidden-link".to_string(),
                name: "隐藏链接".to_string(),
                icon: "link".to_string(),
                color: "#1890ff".to_string(),
                text_color: "#ffffff".to_string(),
                enabled: true,
            },
            InsertType {
                id: "annotation".to_string(),
                name: "批注".to_string(),
                icon: "comment".to_string(),
                color: "#faad14".to_string(),
                text_color: "#ffffff".to_string(),
                enabled: true,
            },
            InsertType {
                id: "reference".to_string(),
                name: "引用提示".to_string(),
                icon: "quote".to_string(),
                color: "#52c41a".to_string(),
                text_color: "#ffffff".to_string(),
                enabled: true,
            },
        ],
        mark_types: vec![
            MarkType {
                id: "highlight".to_string(),
                name: "重点".to_string(),
                bg_color: "#ffd666".to_string(),
                text_color: "#262626".to_string(),
                enabled: true,
            },
            MarkType {
                id: "foreshadow".to_string(),
                name: "伏笔".to_string(),
                bg_color: "#b37feb".to_string(),
                text_color: "#ffffff".to_string(),
                enabled: true,
            },
            MarkType {
                id: "character".to_string(),
                name: "人物".to_string(),
                bg_color: "#69c0ff".to_string(),
                text_color: "#262626".to_string(),
                enabled: true,
            },
            MarkType {
                id: "location".to_string(),
                name: "地点".to_string(),
                bg_color: "#95de64".to_string(),
                text_color: "#262626".to_string(),
                enabled: true,
            },
        ],
        export_defaults: ExportDefaults {
            include_file_name_as_chapter: true,
            include_folder_name_as_volume: true,
            blank_line_between_sections: true,
            output_encoding: "utf-8".to_string(),
        },
        graph_style: GraphStyle {
            direction: "top-to-bottom".to_string(),
            folder_node_color: "#69c0ff".to_string(),
            folder_node_border: "#1890ff".to_string(),
            folder_node_font_size: 14,
            file_node_color: "#ffffff".to_string(),
            file_node_border: "#d9d9d9".to_string(),
            file_node_font_size: 12,
            edge_color: "#bfbfbf".to_string(),
            edge_width: 1,
            show_arrows: true,
            node_gap: 40,
            level_gap: 80,
        },
    }
}

#[tauri::command]
pub fn read_novel_config(warehouse_path: String) -> Result<Value, String> {
    let json_path = ensure_novel_config_storage(&warehouse_path)?;

    if !json_path.exists() {
        let default_cfg = default_novel_config();
        let json = to_string(&default_cfg).map_err(|e| e.to_string())?;
        fs::write(&json_path, json).map_err(|e| e.to_string())?;
        return Ok(serde_json::to_value(default_cfg).unwrap_or(serde_json::json!({})));
    }

    let json_string = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
    if json_string.trim().is_empty() {
        let default_cfg = default_novel_config();
        return Ok(serde_json::to_value(default_cfg).unwrap_or(serde_json::json!({})));
    }
    from_str(&json_string).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_novel_config(warehouse_path: String, config: Value) -> Result<(), String> {
    let json_path = ensure_novel_config_storage(&warehouse_path)?;
    let json_string = to_string(&config).map_err(|e| e.to_string())?;
    fs::write(json_path, json_string).map_err(|e| e.to_string())
}

// ---- txt-meta.json ----

fn txt_meta_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(TXT_META_PATH)
}

fn ensure_txt_meta_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = txt_meta_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(json_path)
}

fn read_txt_meta_raw(warehouse_path: &str) -> Result<Value, String> {
    let json_path = ensure_txt_meta_storage(warehouse_path)?;

    if !json_path.exists() {
        fs::write(&json_path, "{}").map_err(|e| e.to_string())?;
        return Ok(serde_json::json!({}));
    }

    let json_string = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
    if json_string.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    from_str(&json_string).map_err(|e| e.to_string())
}

fn save_txt_meta_raw(warehouse_path: &str, meta: &Value) -> Result<(), String> {
    let json_path = ensure_txt_meta_storage(warehouse_path)?;
    let json_string = to_string(meta).map_err(|e| e.to_string())?;
    fs::write(json_path, json_string).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_txt_meta(warehouse_path: String) -> Result<Value, String> {
    read_txt_meta_raw(&warehouse_path)
}

#[tauri::command]
pub fn write_txt_meta(warehouse_path: String, meta: Value) -> Result<(), String> {
    save_txt_meta_raw(&warehouse_path, &meta)
}

// ---- 文件操作同步 ----

/// 文件重命名时同步 txt-meta.json 中的 key
pub fn sync_txt_meta_rename(
    warehouse_path: &str,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let mut meta = read_txt_meta_raw(warehouse_path)?;
    let old_key = normalize_path(old_path);
    let new_key = normalize_path(new_path);

    if let Some(obj) = meta.as_object_mut() {
        if let Some(value) = obj.remove(&old_key) {
            obj.insert(new_key, value);
        }
    }

    save_txt_meta_raw(warehouse_path, &meta)
}

/// 文件/文件夹删除时清理 txt-meta.json 中的对应条目
pub fn sync_txt_meta_remove(
    warehouse_path: &str,
    removed_path: &str,
    is_folder: bool,
) -> Result<(), String> {
    let mut meta = read_txt_meta_raw(warehouse_path)?;
    let removed_key = normalize_path(removed_path);

    if let Some(obj) = meta.as_object_mut() {
        if is_folder {
            // 删除以该文件夹路径开头的所有 key
            let prefix = format!("{}/", removed_key);
            let keys_to_remove: Vec<String> = obj
                .keys()
                .filter(|k| **k == removed_key || k.starts_with(&prefix))
                .cloned()
                .collect();
            for k in keys_to_remove {
                obj.remove(&k);
            }
        } else {
            obj.remove(&removed_key);
        }
    }

    save_txt_meta_raw(warehouse_path, &meta)
}

/// 获取 txt-meta 中所有包含 linkedFilePath 的插入项（用于合并链接扫描）
pub fn get_txt_hidden_links(warehouse_path: &str) -> Result<Vec<(String, Vec<String>)>, String> {
    let meta = read_txt_meta_raw(warehouse_path)?;
    let mut result: Vec<(String, Vec<String>)> = Vec::new();

    if let Some(obj) = meta.as_object() {
        for (file_path, file_meta) in obj {
            let mut targets: Vec<String> = Vec::new();
            if let Some(inserts) = file_meta.get("inserts").and_then(|v| v.as_array()) {
                for insert in inserts {
                    if let Some(linked) = insert
                        .get("linkedFilePath")
                        .and_then(|v| v.as_str())
                    {
                        let normalized = normalize_path(linked);
                        if !targets.contains(&normalized) {
                            targets.push(normalized);
                        }
                    }
                }
            }
            if !targets.is_empty() {
                result.push((normalize_path(file_path), targets));
            }
        }
    }

    Ok(result)
}

// ---- 自定义链接 ----

const CUSTOM_LINKS_PATH: &str = ".simple_write/custom-links.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomLink {
    pub id: String,
    pub name: String,
    pub target_path: String,
    pub target_text: String,
    pub source_path: String,
}

fn custom_links_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(CUSTOM_LINKS_PATH)
}

fn read_custom_links_raw(warehouse_path: &str) -> Result<Vec<CustomLink>, String> {
    let json_path = custom_links_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !json_path.exists() {
        fs::write(&json_path, "[]").map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }
    let json_string = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
    if json_string.trim().is_empty() { return Ok(vec![]); }
    from_str(&json_string).map_err(|e| e.to_string())
}

fn save_custom_links_raw(warehouse_path: &str, links: &[CustomLink]) -> Result<(), String> {
    let json_path = custom_links_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json_string = to_string(links).map_err(|e| e.to_string())?;
    fs::write(json_path, json_string).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_custom_links(warehouse_path: String) -> Result<Vec<CustomLink>, String> {
    read_custom_links_raw(&warehouse_path)
}

#[tauri::command]
pub fn add_custom_link(warehouse_path: String, link: CustomLink) -> Result<Vec<CustomLink>, String> {
    let mut links = read_custom_links_raw(&warehouse_path)?;
    links.push(link);
    save_custom_links_raw(&warehouse_path, &links)?;
    Ok(links)
}

#[tauri::command]
pub fn remove_custom_link(warehouse_path: String, id: String) -> Result<Vec<CustomLink>, String> {
    let mut links = read_custom_links_raw(&warehouse_path)?;
    links.retain(|l| l.id != id);
    save_custom_links_raw(&warehouse_path, &links)?;
    Ok(links)
}

pub fn sync_custom_links_rename(
    warehouse_path: &str, old_path: &str, new_path: &str,
) -> Result<(), String> {
    let mut links = read_custom_links_raw(warehouse_path)?;
    let old = normalize_path(old_path);
    let new = normalize_path(new_path);
    for link in &mut links {
        if link.target_path == old { link.target_path = new.clone(); }
        if link.source_path == old { link.source_path = new.clone(); }
    }
    save_custom_links_raw(warehouse_path, &links)
}

pub fn sync_custom_links_remove(warehouse_path: &str, removed_path: &str) -> Result<(), String> {
    let mut links = read_custom_links_raw(warehouse_path)?;
    let removed = normalize_path(removed_path);
    links.retain(|l| l.target_path != removed && l.source_path != removed);
    save_custom_links_raw(warehouse_path, &links)
}

// ---- 批量导出 ----

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub warehouse_path: String,
    pub file_paths: Vec<String>,
    pub output_path: String,
    pub include_file_name_as_chapter: bool,
    pub include_folder_name_as_volume: bool,
    pub blank_line_between_sections: bool,
}

fn is_hidden_dir(name: &str) -> bool {
    name == ".simple_write" || name.starts_with('.')
}

/// 递归读取目录中所有 .txt 文件（按文件树顺序），返回 (路径, 内容)
fn read_txt_files_sorted(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by(|a, b| {
        a.file_name()
            .to_string_lossy()
            .cmp(&b.file_name().to_string_lossy())
    });

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if is_hidden_dir(&name) {
            continue;
        }

        if path.is_dir() {
            files.extend(read_txt_files_sorted(&path)?);
        } else if name.ends_with(".txt") {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let full_path = normalize_path(&path.to_string_lossy());
            files.push((full_path, content));
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn export_story_bundle(request: ExportRequest) -> Result<(), String> {
    let root = Path::new(&request.warehouse_path);
    if !root.is_dir() {
        return Err("仓库路径不存在".into());
    }

    let selected_set: std::collections::HashSet<String> = request
        .file_paths
        .iter()
        .map(|p| normalize_path(p))
        .collect();

    let all_txt_files = read_txt_files_sorted(root)?;

    // 按文件树顺序过滤，只保留选中的
    let selected_files: Vec<&(String, String)> = all_txt_files
        .iter()
        .filter(|(p, _)| selected_set.contains(p))
        .collect();

    if selected_files.is_empty() {
        return Err("没有选中任何 .txt 文件".into());
    }

    let mut output = String::new();
    let mut last_volume: Option<String> = None;

    for (i, (path, content)) in selected_files.iter().enumerate() {
        // 卷名：取文件所在的一级子文件夹名
        let rel = if path.starts_with(&normalize_path(&request.warehouse_path)) {
            let base = normalize_path(&request.warehouse_path);
            let rel = path.strip_prefix(&base).unwrap_or(path);
            rel.trim_start_matches('/').to_string()
        } else {
            path.clone()
        };

        if request.include_folder_name_as_volume {
            let parts: Vec<&str> = rel.split('/').collect();
            if parts.len() >= 2 {
                let volume = parts[0].to_string();
                if last_volume.as_ref() != Some(&volume) {
                    if !output.is_empty() {
                        output.push_str("\n\n");
                    }
                    output.push_str(&format!("# {}\n", volume));
                    last_volume = Some(volume);
                }
            }
        }

        if request.include_file_name_as_chapter {
            let name = Path::new(path)
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if request.blank_line_between_sections && !output.is_empty() {
                output.push_str("\n\n");
            }
            output.push_str(&format!("## {}\n\n", name));
        } else if request.blank_line_between_sections && i > 0 {
            output.push_str("\n\n");
        }

        output.push_str(content.trim());
    }

    // 确保输出文件以 .txt 结尾
    let output_path = if request.output_path.ends_with(".txt") {
        request.output_path.clone()
    } else {
        format!("{}.txt", request.output_path)
    };

    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&output_path, output).map_err(|e| e.to_string())
}
