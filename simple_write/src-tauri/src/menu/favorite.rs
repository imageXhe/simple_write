use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::fs;
use std::path::{Path, PathBuf};

const JSON_PATH: &str = ".simple_write/favorite.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub create_time: String,
    pub path: String,
}

fn favorite_json_path(warehouse_path: &str) -> PathBuf {
    Path::new(warehouse_path).join(JSON_PATH)
}

fn ensure_favorite_storage(warehouse_path: &str) -> Result<PathBuf, String> {
    let json_path = favorite_json_path(warehouse_path);

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if !json_path.exists() {
        fs::write(&json_path, "[]").map_err(|error| error.to_string())?;
    }

    Ok(json_path)
}

fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

fn read_favorites(warehouse_path: &str) -> Result<Vec<FavoriteItem>, String> {
    let json_path = ensure_favorite_storage(warehouse_path)?;
    let json_string = fs::read_to_string(json_path).map_err(|error| error.to_string())?;

    if json_string.trim().is_empty() {
        return Ok(vec![]);
    }

    from_str(&json_string).map_err(|error| error.to_string())
}

fn save_favorites(warehouse_path: &str, favorites: &[FavoriteItem]) -> Result<Vec<FavoriteItem>, String> {
    let json_path = ensure_favorite_storage(warehouse_path)?;
    let json_string = to_string(favorites).map_err(|error| error.to_string())?;

    fs::write(json_path, json_string).map_err(|error| error.to_string())?;
    Ok(favorites.to_vec())
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

pub fn create_favorites_json(warehouse_path: String) {
    match ensure_favorite_storage(&warehouse_path) {
        Ok(_) => println!("create_favorites_json SUCCESS"),
        Err(error) => println!("create_favorites_json {}", error),
    }
}

pub fn replace_favorite_path_prefix(
    warehouse_path: &str,
    old_prefix: &str,
    new_prefix: &str,
) -> Result<Vec<FavoriteItem>, String> {
    let normalized_old_prefix = normalize_path(old_prefix);
    let normalized_new_prefix = normalize_path(new_prefix);
    let mut favorites = read_favorites(warehouse_path)?;

    for favorite in &mut favorites {
        if is_same_or_child_path(&favorite.path, &normalized_old_prefix) {
            favorite.path = replace_path_prefix(
                &favorite.path,
                &normalized_old_prefix,
                &normalized_new_prefix,
            );
        }
    }

    save_favorites(warehouse_path, &favorites)
}

pub fn remove_favorites_by_path_prefix(
    warehouse_path: &str,
    target_prefix: &str,
) -> Result<Vec<FavoriteItem>, String> {
    let normalized_target_prefix = normalize_path(target_prefix);
    let favorites = read_favorites(warehouse_path)?;
    let filtered_favorites: Vec<FavoriteItem> = favorites
        .into_iter()
        .filter(|favorite| !is_same_or_child_path(&favorite.path, &normalized_target_prefix))
        .collect();

    save_favorites(warehouse_path, &filtered_favorites)
}

#[tauri::command]
pub fn get_favorites_json(warehouse_path: String) -> Result<Vec<FavoriteItem>, String> {
    read_favorites(&warehouse_path)
}

#[tauri::command]
pub fn add_favorite(
    warehouse_path: String,
    file_path: String,
    item_type: String,
) -> Result<Vec<FavoriteItem>, String> {
    let normalized_path = normalize_path(&file_path);
    let mut favorites = read_favorites(&warehouse_path)?;

    if favorites.iter().any(|favorite| favorite.path == normalized_path) {
        return Ok(favorites);
    }

    favorites.push(FavoriteItem {
        item_type,
        create_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        path: normalized_path,
    });

    save_favorites(&warehouse_path, &favorites)
}

#[tauri::command]
pub fn remove_favorite(warehouse_path: String, file_path: String) -> Result<Vec<FavoriteItem>, String> {
    let normalized_path = normalize_path(&file_path);
    let favorites = read_favorites(&warehouse_path)?;
    let filtered_favorites: Vec<FavoriteItem> = favorites
        .into_iter()
        .filter(|favorite| favorite.path != normalized_path)
        .collect();

    save_favorites(&warehouse_path, &filtered_favorites)
}
