use crate::menu::bookmark::{remove_bookmarks_by_path_prefix, replace_bookmark_path_prefix};
use crate::menu::favorite::{remove_favorites_by_path_prefix, replace_favorite_path_prefix};
use crate::menu::novel_json::{sync_txt_meta_rename, sync_txt_meta_remove, sync_custom_links_rename, sync_custom_links_remove};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::hash_map::DefaultHasher;
use base64::Engine;
use std::ffi::OsStr;
use regex::Regex;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNode {
    pub is_folder: bool,
    pub info: FileNodeInfo,
    pub id: u64,
    pub key: Vec<String>,
    pub children: Vec<FileNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNodeInfo {
    pub name: String,
    pub create_time: String,
    pub modify_time: String,
    pub note_num: u64,
    pub folder_num: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileRequest {
    pub is_folder: bool,
    pub name: String,
    #[serde(default)]
    pub key_parent: Vec<String>,
    pub warehouse_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFileRequest {
    #[serde(default)]
    pub key: Vec<String>,
    pub copy_suffix: String,
    pub warehouse_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileRequest {
    #[serde(default)]
    pub key: Vec<String>,
    pub new_name: String,
    pub warehouse_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MoveFileRequest {
    #[serde(default)]
    pub key: Vec<String>,
    pub new_parent_path: String,
    pub warehouse_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileRequest {
    #[serde(default)]
    pub key: Vec<String>,
    pub warehouse_path: String,
}

const JSON_PATH: &str = ".simple_write/file_list.json";
const INTERNAL_DIR_NAME: &str = ".simple_write";

fn file_json_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(JSON_PATH)
}

fn ensure_file_tree_storage(warehouse_path: &str) -> Result<(), String> {
    let json_path = file_json_path(warehouse_path);

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if !json_path.exists() {
        fs::write(&json_path, "[]").map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn save_file_tree(warehouse_path: &str, node_list: &[FileNode]) -> Result<(), String> {
    ensure_file_tree_storage(warehouse_path)?;

    let json_string = to_string(node_list).map_err(|error| error.to_string())?;
    fs::write(file_json_path(warehouse_path), json_string).map_err(|error| error.to_string())
}

fn current_time_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// 快速粘贴功能专用时间戳格式：2026.01.23 15:52:12
fn quick_paste_timestamp() -> String {
    Local::now().format("%Y.%m.%d %H:%M:%S").to_string()
}

// 快速粘贴错误类型码（前端根据类型码做 i18n 翻译）
fn quick_paste_err(code: &str, detail: Option<&str>) -> String {
    match detail {
        Some(d) => format!("{}|{}", code, d),
        None => code.to_string(),
    }
}

fn format_system_time(time: SystemTime) -> String {
    let utc_time: DateTime<Utc> = DateTime::<Utc>::from(time);
    utc_time.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

fn metadata_time_string(metadata: &fs::Metadata) -> String {
    metadata
        .created()
        .or_else(|_| metadata.modified())
        .map(format_system_time)
        .unwrap_or_else(|_| current_time_string())
}

fn hash_path(segments: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    segments.join("/").hash(&mut hasher);
    hasher.finish()
}

fn strip_md_suffix(name: &str) -> String {
    if name.to_lowercase().ends_with(".md") {
        name[..name.len() - 3].to_string()
    } else {
        name.to_string()
    }
}

fn normalize_entry_base_name(base_name: &str, is_folder: bool) -> String {
    let trimmed = base_name.trim();

    if is_folder {
        trimmed.to_string()
    } else {
        strip_md_suffix(trimmed)
    }
}

fn create_unique_entry_name(parent_path: &Path, base_name: &str, is_folder: bool) -> Result<String, String> {
    let normalized_base_name = normalize_entry_base_name(base_name, is_folder);
    let mut suffix_index = 0;

    loop {
        let unique_base = if suffix_index == 0 {
            normalized_base_name.clone()
        } else {
            format!("{}({})", normalized_base_name, suffix_index)
        };

        let entry_name = if is_folder {
            unique_base
        } else {
            format!("{}.md", unique_base)
        };

        let entry_path = parent_path.join(&entry_name);

        if !entry_path.exists() {
            if is_folder {
                fs::create_dir_all(&entry_path).map_err(|error| error.to_string())?;
            } else {
                fs::File::create(&entry_path).map_err(|error| error.to_string())?;
            }

            return Ok(entry_name);
        }

        suffix_index += 1;
    }
}

fn resolve_parent_path(warehouse_path: &Path, key_parent: &[String]) -> Result<PathBuf, String> {
    let mut current_path = warehouse_path.to_path_buf();

    for segment in key_parent {
        current_path = current_path.join(segment);
    }

    if current_path.exists() && current_path.is_dir() {
        Ok(current_path)
    } else {
        Err("未找到目标父级目录".to_string())
    }
}

fn resolve_entry_path(warehouse_path: &Path, key: &[String]) -> Result<PathBuf, String> {
    let mut current_path = warehouse_path.to_path_buf();

    for segment in key {
        current_path = current_path.join(segment);
    }

    if current_path.exists() {
        Ok(current_path)
    } else {
        Err("未找到目标文件或文件夹".to_string())
    }
}

fn canonicalize_existing_path(path: &Path, error_message: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|_| error_message.to_string())
}

fn normalize_requested_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        Err("名称不能为空".to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn path_file_name_string(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(|name| name.to_string())
        .ok_or_else(|| "无法识别目标名称".to_string())
}

fn split_name_parts(name: &str) -> (String, String) {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(name)
        .to_string();
    let extension = path
        .extension()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .map(|value| format!(".{}", value))
        .unwrap_or_default();

    (stem, extension)
}

fn create_duplicate_path(
    parent_path: &Path,
    entry_name: &str,
    is_folder: bool,
    copy_suffix: &str,
) -> Result<PathBuf, String> {
    let normalized_name = normalize_requested_name(entry_name)?;
    let suffix = copy_suffix.trim();
    let (stem, extension) = if is_folder {
        (normalized_name, String::new())
    } else {
        split_name_parts(&normalized_name)
    };
    let duplicated_stem = format!("{}{}", stem, suffix);
    let mut suffix_index = 0;

    loop {
        let candidate_name = if suffix_index == 0 {
            if is_folder {
                duplicated_stem.clone()
            } else {
                format!("{}{}", duplicated_stem, extension)
            }
        } else if is_folder {
            format!("{}({})", duplicated_stem, suffix_index)
        } else {
            format!("{}({}){}", duplicated_stem, suffix_index, extension)
        };

        let candidate_path = parent_path.join(&candidate_name);

        if !candidate_path.exists() {
            return Ok(candidate_path);
        }

        suffix_index += 1;
    }
}

fn build_renamed_entry_name(source_path: &Path, requested_name: &str) -> Result<String, String> {
    let normalized_name = normalize_requested_name(requested_name)?;

    if source_path.is_dir() {
        return Ok(normalized_name);
    }

    if Path::new(&normalized_name).extension().is_some() {
        return Ok(normalized_name);
    }

    let current_extension = source_path
        .extension()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty());

    Ok(match current_extension {
        Some(extension) => format!("{}.{}", normalized_name, extension),
        None => normalized_name,
    })
}

fn copy_entry_recursive(source_path: &Path, destination_path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(source_path).map_err(|error| error.to_string())?;

    if metadata.is_dir() {
        fs::create_dir_all(destination_path).map_err(|error| error.to_string())?;

        for entry in read_directory_entries(source_path)? {
            let file_name = entry.file_name();
            copy_entry_recursive(&entry.path(), &destination_path.join(file_name))?;
        }

        Ok(())
    } else {
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        fs::copy(source_path, destination_path)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn read_directory_entries(directory: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    entries.sort_by(|left, right| {
        left.file_name()
            .to_string_lossy()
            .cmp(&right.file_name().to_string_lossy())
    });

    Ok(entries)
}

fn build_file_nodes(directory: &Path, relative_segments: &[String]) -> Result<Vec<FileNode>, String> {
    let mut nodes = Vec::new();

    for entry in read_directory_entries(directory)? {
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| "文件名包含无法识别的字符".to_string())?;

        if file_name == INTERNAL_DIR_NAME {
            continue;
        }

        let entry_path = entry.path();
        let metadata = fs::metadata(&entry_path).map_err(|error| error.to_string())?;
        let is_folder = metadata.is_dir();

        let mut entry_segments = relative_segments.to_vec();
        entry_segments.push(file_name.clone());

        let children = if is_folder {
            build_file_nodes(&entry_path, &entry_segments)?
        } else {
            vec![]
        };

        let note_num = children.iter().filter(|node| !node.is_folder).count() as u64;
        let folder_num = children.iter().filter(|node| node.is_folder).count() as u64;
        let time_string = metadata_time_string(&metadata);

        nodes.push(FileNode {
            is_folder,
            info: FileNodeInfo {
                name: file_name,
                create_time: time_string.clone(),
                modify_time: time_string,
                note_num,
                folder_num,
            },
            id: hash_path(&entry_segments),
            key: entry_segments,
            children,
        });
    }

    Ok(nodes)
}

fn refresh_file_tree(warehouse_path: &str) -> Result<Vec<FileNode>, String> {
    ensure_file_tree_storage(warehouse_path)?;

    let warehouse_root = Path::new(warehouse_path);

    if !warehouse_root.exists() || !warehouse_root.is_dir() {
        save_file_tree(warehouse_path, &[])?;
        return Ok(vec![]);
    }

    let node_list = build_file_nodes(warehouse_root, &[])?;
    save_file_tree(warehouse_path, &node_list)?;

    Ok(node_list)
}

fn get_file_tree_or_empty(warehouse_path: &str) -> Vec<FileNode> {
    match refresh_file_tree(warehouse_path) {
        Ok(node_list) => node_list,
        Err(error) => {
            println!("get_file_json {}", error);
            vec![]
        }
    }
}

#[tauri::command]
pub fn get_file_json(warehouse_path: String) -> Vec<FileNode> {
    get_file_tree_or_empty(&warehouse_path)
}

#[tauri::command]
pub fn get_file_content(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    if path.is_dir() {
        return Err("目标是文件夹，无法读取内容".to_string());
    }

    fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_file_as_base64(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    if path.is_dir() {
        return Err("目标是文件夹，无法读取内容".to_string());
    }

    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let mime = mime_guess_from_path(path);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, base64_data))
}

fn mime_guess_from_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[tauri::command]
pub fn save_file_content(file_path: String, content: String) -> Result<(), String> {
    let path = Path::new(&file_path);

    if path.is_dir() {
        return Err("目标是文件夹，无法写入内容".to_string());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    fs::write(path, content).map_err(|error| error.to_string())
}

// 快速粘贴：读取剪贴板内容并追加到仓库根目录下的文件
#[tauri::command]
pub fn quick_paste(warehouse_path: String, file_name: String) -> Result<String, String> {
    let err = |code: &str| quick_paste_err(code, None);
    let err_detail = |code: &str, detail: &str| quick_paste_err(code, Some(detail));

    // 1. 读取剪贴板内容
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| err_detail("clipboard_access", &e.to_string()))?;

    let content = clipboard
        .get_text()
        .map_err(|_| err("clipboard_no_text"))?;

    // 2. 检查内容是否为空或全是空白字符
    if content.trim().is_empty() {
        return Err(err("clipboard_empty"));
    }

    // 3. 构建文件路径
    let file_path = Path::new(&warehouse_path).join(format!("{}.txt", file_name));

    // 4. 检查是否与上次粘贴内容重复
    if file_path.exists() {
        if let Ok(existing) = fs::read_to_string(&file_path) {
            let re = Regex::new(r"\d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2}\n")
                .map_err(|_| err("file_open"))?;
            if let Some(last_match) = re.find_iter(&existing).last() {
                let after_ts = &existing[last_match.end()..];
                let last_content = after_ts.trim_end_matches(|c| c == '\n' || c == '\r');
                if last_content == content {
                    return Err(err("duplicate_content"));
                }
            }
        }
    }

    // 5. 以追加模式打开文件（不存在则创建）
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| err_detail("file_open", &e.to_string()))?;

    // 6. 写入时间戳、内容和空行分隔
    let timestamp = quick_paste_timestamp();
    writeln!(file, "{}", timestamp).map_err(|e| err_detail("write_file", &e.to_string()))?;
    writeln!(file, "{}", content).map_err(|e| err_detail("write_file", &e.to_string()))?;
    writeln!(file).map_err(|e| err_detail("write_file", &e.to_string()))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn test(warehouse_path: String) -> String {
    let path_total: String = format!("{}{}{}", warehouse_path, "/", JSON_PATH);
    println!("{}", path_total);
    path_total
}

pub fn create_file_json(warehouse_path: String) {
    match refresh_file_tree(&warehouse_path) {
        Ok(_) => println!("create_file_json SUCCESS"),
        Err(error) => println!("create_file_json {}", error),
    }
}

#[tauri::command]
pub fn create_file(request: CreateFileRequest) -> Result<Vec<FileNode>, String> {
    let warehouse_root = Path::new(&request.warehouse_path);
    let parent_path = resolve_parent_path(warehouse_root, &request.key_parent)?;
    let _entry_name = create_unique_entry_name(&parent_path, &request.name, request.is_folder)?;

    refresh_file_tree(&request.warehouse_path)
}

#[tauri::command]
pub fn duplicate_file_entry(request: DuplicateFileRequest) -> Result<Vec<FileNode>, String> {
    let warehouse_root = Path::new(&request.warehouse_path);
    let source_path = resolve_entry_path(warehouse_root, &request.key)?;
    let parent_path = source_path
        .parent()
        .ok_or_else(|| "未找到目标父级目录".to_string())?;
    let source_name = path_file_name_string(&source_path)?;
    let destination_path = create_duplicate_path(
        parent_path,
        &source_name,
        source_path.is_dir(),
        &request.copy_suffix,
    )?;

    copy_entry_recursive(&source_path, &destination_path)?;
    refresh_file_tree(&request.warehouse_path)
}

#[tauri::command]
pub fn rename_file_entry(request: RenameFileRequest) -> Result<Vec<FileNode>, String> {
    let warehouse_root = Path::new(&request.warehouse_path);
    let source_path = resolve_entry_path(warehouse_root, &request.key)?;
    let parent_path = source_path
        .parent()
        .ok_or_else(|| "未找到目标父级目录".to_string())?;
    let target_name = build_renamed_entry_name(&source_path, &request.new_name)?;
    let current_name = path_file_name_string(&source_path)?;

    if current_name == target_name {
        return Err("新旧名称相同".to_string());
    }

    let target_path = parent_path.join(&target_name);

    if target_path.exists() {
        return Err("目标名称已存在".to_string());
    }

    let source_path_string = source_path.to_string_lossy().replace("\\", "/");
    let target_path_string = target_path.to_string_lossy().replace("\\", "/");

    fs::rename(&source_path, &target_path).map_err(|error| error.to_string())?;
    replace_favorite_path_prefix(&request.warehouse_path, &source_path_string, &target_path_string)?;
    replace_bookmark_path_prefix(&request.warehouse_path, &source_path_string, &target_path_string)?;
    let _ = sync_txt_meta_rename(&request.warehouse_path, &source_path_string, &target_path_string);
    let _ = sync_custom_links_rename(&request.warehouse_path, &source_path_string, &target_path_string);
    refresh_file_tree(&request.warehouse_path)
}

#[tauri::command]
pub fn move_file_entry(request: MoveFileRequest) -> Result<Vec<FileNode>, String> {
    let warehouse_root = Path::new(&request.warehouse_path);
    let source_path = resolve_entry_path(warehouse_root, &request.key)?;
    let source_canonical_path = canonicalize_existing_path(&source_path, "未找到目标文件或文件夹")?;
    let warehouse_canonical_path = canonicalize_existing_path(warehouse_root, "未找到仓库目录")?;
    let target_parent_path = PathBuf::from(request.new_parent_path.trim());

    if !target_parent_path.exists() || !target_parent_path.is_dir() {
        return Err("请选择有效的目标目录".to_string());
    }

    let target_parent_canonical_path = canonicalize_existing_path(&target_parent_path, "请选择有效的目标目录")?;

    if !target_parent_canonical_path.starts_with(&warehouse_canonical_path) {
        return Err("目标路径必须位于当前仓库内".to_string());
    }

    if source_path.is_dir() && target_parent_canonical_path.starts_with(&source_canonical_path) {
        return Err("不能移动到当前文件夹内部".to_string());
    }

    let source_name = path_file_name_string(&source_path)?;
    let target_path = target_parent_path.join(&source_name);

    if source_path == target_path {
        return Err("新旧路径相同".to_string());
    }

    if target_path.exists() {
        return Err("目标路径已存在同名文件或文件夹".to_string());
    }

    let source_path_string = source_path.to_string_lossy().replace("\\", "/");
    let target_path_string = target_path.to_string_lossy().replace("\\", "/");

    fs::rename(&source_path, &target_path).map_err(|error| error.to_string())?;
    replace_favorite_path_prefix(&request.warehouse_path, &source_path_string, &target_path_string)?;
    replace_bookmark_path_prefix(&request.warehouse_path, &source_path_string, &target_path_string)?;
    let _ = sync_txt_meta_rename(&request.warehouse_path, &source_path_string, &target_path_string);
    let _ = sync_custom_links_rename(&request.warehouse_path, &source_path_string, &target_path_string);
    refresh_file_tree(&request.warehouse_path)
}

use crate::readme_template::README_TEMPLATE;

#[tauri::command]
pub fn generate_readme(warehouse_path: String) -> Result<String, String> {
    let readme_path = std::path::Path::new(&warehouse_path).join("README.md");

    // 如果 README.md 已存在，则跳过生成
    if readme_path.exists() {
        return Err("README.md 已存在，请手动删除后再生成".to_string());
    }

    std::fs::write(&readme_path, README_TEMPLATE).map_err(|e| e.to_string())?;
    Ok("README.md 生成成功".to_string())
}

#[tauri::command]
pub fn delete_file_entry(request: DeleteFileRequest) -> Result<Vec<FileNode>, String> {
    let warehouse_root = Path::new(&request.warehouse_path);
    let source_path = resolve_entry_path(warehouse_root, &request.key)?;
    let source_path_string = source_path.to_string_lossy().replace("\\", "/");
    let is_folder = source_path.is_dir();

    if is_folder {
        fs::remove_dir_all(&source_path).map_err(|error| error.to_string())?;
    } else {
        fs::remove_file(&source_path).map_err(|error| error.to_string())?;
    }

    remove_favorites_by_path_prefix(&request.warehouse_path, &source_path_string)?;
    remove_bookmarks_by_path_prefix(&request.warehouse_path, &source_path_string)?;
    let _ = sync_txt_meta_remove(&request.warehouse_path, &source_path_string, is_folder);
    let _ = sync_custom_links_remove(&request.warehouse_path, &source_path_string);
    refresh_file_tree(&request.warehouse_path)
}
