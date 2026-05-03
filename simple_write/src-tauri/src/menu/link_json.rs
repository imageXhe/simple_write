use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::path::{Path, PathBuf};

const JSON_PATH: &str = ".simple_write/link.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkEntry {
    pub source: String,
    pub targets: Vec<String>,
}

fn link_json_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(JSON_PATH)
}

fn ensure_link_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = link_json_path(warehouse_path);
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !json_path.exists() {
        fs::write(&json_path, "[]").map_err(|e| e.to_string())?;
    }
    Ok(json_path)
}

fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

fn read_links(warehouse_path: &str) -> Result<Vec<LinkEntry>, String> {
    let json_path = ensure_link_storage(warehouse_path)?;
    let json_string = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
    if json_string.trim().is_empty() {
        return Ok(vec![]);
    }
    from_str(&json_string).map_err(|e| e.to_string())
}

fn save_links(warehouse_path: &str, links: &[LinkEntry]) -> Result<Vec<LinkEntry>, String> {
    let json_path = ensure_link_storage(warehouse_path)?;
    let json_string = to_string(links).map_err(|e| e.to_string())?;
    fs::write(json_path, json_string).map_err(|e| e.to_string())?;
    Ok(links.to_vec())
}

/// 递归读取目录下所有条目（文件和文件夹），返回路径列表
fn read_all_entries(dir: &Path) -> Result<Vec<String>, String> {
    let mut entries_paths = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".simple_write" || name.starts_with('.') {
            continue;
        }

        let full_path = normalize_path(&path.to_string_lossy());
        entries_paths.push(full_path);

        if path.is_dir() {
            entries_paths.extend(read_all_entries(&path)?);
        }
    }

    Ok(entries_paths)
}

/// 递归读取目录下可读文本文件（.md/.txt），返回 (路径, 内容)
fn read_text_files(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".simple_write" || name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            files.extend(read_text_files(&path)?);
        } else if name.ends_with(".md") || name.ends_with(".txt") {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let full_path = normalize_path(&path.to_string_lossy());
            files.push((full_path, content));
        }
    }

    Ok(files)
}

/// 根据 [[name]] 在条目列表中匹配路径
fn resolve_wiki_link(name: &str, all_entries: &[String]) -> Option<String> {
    let name_lower = name.to_lowercase();

    // 精确匹配（含扩展名）
    for path in all_entries {
        let entry_name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if entry_name == name {
            return Some(path.clone());
        }
    }

    // 补 .md 扩展名
    let with_md = format!("{}.md", name);
    for path in all_entries {
        let entry_name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if entry_name == with_md {
            return Some(path.clone());
        }
    }

    // 模糊匹配（忽略大小写，忽略扩展名）
    for path in all_entries {
        let entry_name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let entry_lower = entry_name.to_lowercase();
        if entry_lower == name_lower {
            return Some(path.clone());
        }
        // 忽略扩展名匹配
        if let Some(dot) = entry_lower.rfind('.') {
            if entry_lower[..dot] == name_lower {
                return Some(path.clone());
            }
        }
    }

    None
}

/// 扫描仓库中所有可读文件的 [[链接]]，生成 link.json
#[tauri::command]
pub fn scan_links(warehouse_path: String) -> Result<Vec<LinkEntry>, String> {
    let root = Path::new(&warehouse_path);
    if !root.is_dir() {
        return Err("仓库路径不存在".into());
    }

    let all_entries = read_all_entries(root)?;
    let text_files = read_text_files(root)?;
    let wiki_regex = Regex::new(r"\[\[([^\]]+)\]\]").map_err(|e| e.to_string())?;

    let mut entries: Vec<LinkEntry> = Vec::new();

    for (source_path, content) in &text_files {
        let mut targets: Vec<String> = Vec::new();
        for cap in wiki_regex.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            if let Some(resolved) = resolve_wiki_link(name, &all_entries) {
                let normalized = normalize_path(&resolved);
                if !targets.contains(&normalized) {
                    targets.push(normalized);
                }
            }
        }
        let normalized_source = normalize_path(source_path);
        entries.push(LinkEntry {
            source: normalized_source,
            targets,
        });
    }

    save_links(&warehouse_path, &entries)
}

/// 获取某文件的出链列表
#[tauri::command]
pub fn get_outgoing_links(warehouse_path: String, file_path: String) -> Result<Vec<String>, String> {
    let links = read_links(&warehouse_path)?;
    let normalized = normalize_path(&file_path);
    for entry in &links {
        if entry.source == normalized {
            return Ok(entry.targets.clone());
        }
    }
    Ok(vec![])
}

/// 获取某文件的反向链接列表（哪些文件链接了它）
#[tauri::command]
pub fn get_incoming_links(
    warehouse_path: String,
    file_path: String,
) -> Result<Vec<String>, String> {
    let links = read_links(&warehouse_path)?;
    let normalized = normalize_path(&file_path);
    let mut incoming: Vec<String> = Vec::new();
    for entry in &links {
        if entry.targets.contains(&normalized) {
            incoming.push(entry.source.clone());
        }
    }
    Ok(incoming)
}
