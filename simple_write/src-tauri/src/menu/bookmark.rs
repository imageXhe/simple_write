use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::path::{Path, PathBuf};

const JSON_PATH: &str = ".simple_write/bookmarks.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
}

fn bookmark_json_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(JSON_PATH)
}

fn ensure_bookmark_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = bookmark_json_path(warehouse_path);

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if !json_path.exists() {
        fs::write(&json_path, "[]").map_err(|error| error.to_string())?;
    }

    Ok(json_path)
}

fn read_bookmarks(warehouse_path: &str) -> Result<Vec<Bookmark>, String> {
    let json_path = ensure_bookmark_storage(warehouse_path)?;
    let json_string = fs::read_to_string(json_path).map_err(|error| error.to_string())?;

    if json_string.trim().is_empty() {
        return Ok(vec![]);
    }

    from_str(&json_string).map_err(|error| error.to_string())
}

fn save_bookmarks(warehouse_path: &str, bookmarks: &[Bookmark]) -> Result<Vec<Bookmark>, String> {
    let json_path = ensure_bookmark_storage(warehouse_path)?;
    let json_string = to_string(bookmarks).map_err(|error| error.to_string())?;

    fs::write(json_path, json_string).map_err(|error| error.to_string())?;
    Ok(bookmarks.to_vec())
}

fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

fn is_same_or_child_path(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

fn replace_path_prefix(path: &str, old_prefix: &str, new_prefix: &str) -> String {
    if path == old_prefix {
        return new_prefix.to_string();
    }

    format!("{new_prefix}{}", &path[old_prefix.len()..])
}

pub fn replace_bookmark_path_prefix(
    warehouse_path: &str,
    old_prefix: &str,
    new_prefix: &str,
) -> Result<Vec<Bookmark>, String> {
    let normalized_old_prefix = normalize_path(old_prefix);
    let normalized_new_prefix = normalize_path(new_prefix);
    let mut bookmarks = read_bookmarks(warehouse_path)?;

    for bookmark in &mut bookmarks {
        if is_same_or_child_path(&bookmark.file_path, &normalized_old_prefix) {
            let next_path = replace_path_prefix(
                &bookmark.file_path,
                &normalized_old_prefix,
                &normalized_new_prefix,
            );
            bookmark.file_path = next_path.clone();
            bookmark.id = next_path;
        }
    }

    save_bookmarks(warehouse_path, &bookmarks)
}

pub fn remove_bookmarks_by_path_prefix(
    warehouse_path: &str,
    target_prefix: &str,
) -> Result<Vec<Bookmark>, String> {
    let normalized_target_prefix = normalize_path(target_prefix);
    let bookmarks = read_bookmarks(warehouse_path)?;

    let filtered_bookmarks: Vec<Bookmark> = bookmarks
        .into_iter()
        .filter(|bookmark| !is_same_or_child_path(&bookmark.file_path, &normalized_target_prefix))
        .collect();

    save_bookmarks(warehouse_path, &filtered_bookmarks)
}

pub fn create_bookmarks_json(warehouse_path: String) {
    match ensure_bookmark_storage(&warehouse_path) {
        Ok(_) => println!("create_bookmarks_json SUCCESS"),
        Err(error) => println!("create_bookmarks_json {}", error),
    }
}

#[tauri::command]
pub fn get_bookmarks_json(warehouse_path: String) -> Result<Vec<Bookmark>, String> {
    read_bookmarks(&warehouse_path)
}

#[tauri::command]
pub fn add_bookmark(warehouse_path: String, file_path: String, name: String) -> Result<Vec<Bookmark>, String> {
    let normalized_path = file_path.replace("\\", "/");
    let mut bookmarks = read_bookmarks(&warehouse_path)?;

    if bookmarks.iter().any(|bookmark| bookmark.file_path == normalized_path) {
        return Ok(bookmarks);
    }

    bookmarks.push(Bookmark {
        id: normalized_path.clone(),
        name,
        file_path: normalized_path,
        created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    });

    save_bookmarks(&warehouse_path, &bookmarks)
}

#[tauri::command]
pub fn remove_bookmark(warehouse_path: String, file_path: String) -> Result<Vec<Bookmark>, String> {
    let normalized_path = file_path.replace("\\", "/");
    let bookmarks = read_bookmarks(&warehouse_path)?;
    let filtered_bookmarks: Vec<Bookmark> = bookmarks
        .into_iter()
        .filter(|bookmark| bookmark.file_path != normalized_path)
        .collect();

    save_bookmarks(&warehouse_path, &filtered_bookmarks)
}
