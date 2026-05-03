
use super::super::file_list::note::{create_file_json, generate_readme};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use serde::{Deserialize, Serialize};
use super::setting::create_setting_json;
use super::warehouse_json::update_warehouse_json;
use super::bookmark::create_bookmarks_json;
use super::favorite::create_favorites_json;


fn success() -> String {
    return "SUCCESS".to_string()
}
fn error() -> String {
    return "ERROR".to_string()
}

#[derive(Debug , Serialize , Deserialize)]
pub struct ResponseInfo {
    status: String,
    msg: String
}

#[tauri::command]
pub fn open_warehouse(folder_path: String) -> ResponseInfo {
    let warehouse_path: &Path = Path::new(&folder_path);
    if !warehouse_path.exists() {
        return ResponseInfo {
            status: error(),
            msg: "文件夹不存在".to_string(),
        };
    }
    if !warehouse_path.is_dir() {
        return ResponseInfo {
            status: error(),
            msg: "所选路径不是文件夹".to_string(),
        };
    }

    // 从完整路径中提取文件夹名称和父级路径
    let name = String::from(warehouse_path.file_name().unwrap().to_str().unwrap());
    let parent_path = String::from(warehouse_path.parent().unwrap().to_str().unwrap());

    // 如果 .simple_write 目录不存在，则创建
    let json_dir = format!("{}/.simple_write", folder_path);
    if !Path::new(&json_dir).exists() {
        create_json_dir(folder_path.clone());
        create_setting_json(folder_path.clone());
        create_file_json(folder_path.clone());
        create_bookmarks_json(folder_path.clone());
        create_favorites_json(folder_path.clone());
        // 自动生成 README.md（如果不存在）
        let _ = generate_readme(folder_path.clone());
    }

    let time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    update_warehouse_json(time, name, parent_path, "create".to_string());

    ResponseInfo {
        status: success(),
        msg: folder_path,
    }
}

#[tauri::command]
pub fn create_warehouse(path: String, name: String) -> ResponseInfo {
    let path_total: String = format!("{}{}{}", path, "/", name);
    let warehouse_path: &Path = Path::new(&path_total);
    if warehouse_path.exists() {
        let response: ResponseInfo = ResponseInfo { 
            status: error(), 
            msg: "路径下已有同名文件夹".to_string() };
        return response
    }else {
        match fs::create_dir(&path_total) {
            Ok(_) => {
                let time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                create_json_dir(path_total.clone());
                create_setting_json(path_total.clone());
                create_file_json(path_total.clone());
                create_bookmarks_json(path_total.clone());
                create_favorites_json(path_total.clone());
                // 自动生成 README.md
                let _ = generate_readme(path_total.clone());

                update_warehouse_json(time, name, path, "create".to_string());

                let response: ResponseInfo = ResponseInfo { 
                    status: success(), 
                    msg: path_total 
                };
                return response
            }
            Err(e) => {
                let response: ResponseInfo = ResponseInfo { 
                    status: error(), 
                    msg: e.to_string() 
                };
                return response
            }
        }
    }
}


#[tauri::command]
pub fn delete_warehouse(path: String, name: String, id: u64) -> ResponseInfo {
    let path_total: String = format!("{}{}{}", path, "/", name);
    match fs::remove_dir_all(&path_total) {
        Ok(_) => {
            update_warehouse_json(id, name, path.clone(), "delete".to_string());
            let response: ResponseInfo = ResponseInfo { 
                status: success(), 
                msg: path
            };
            return response
        }
        Err(e) => {
            let response: ResponseInfo = ResponseInfo {
                status: error(),
                msg: e.to_string()
            };
            return response
        }
    }
}

#[tauri::command]
pub fn rename_warehouse(path: String, name: String, id: u64, new_name: String) -> ResponseInfo {
    let path_total: String = format!("{}{}{}", path, "/", name);
    let path_new: String = format!("{}{}{}", path, "/", new_name);
    let warehouse_path: &Path = Path::new(&path_new);
    if warehouse_path.exists() {
        let response: ResponseInfo = ResponseInfo { 
            status: error(), 
            msg: "路径下已有同名文件夹".to_string() };
        return response
    }
    match fs::rename(&path_total, &path_new) {
        Ok(_) => {
            update_warehouse_json(id, new_name, path.clone(), "rename".to_string());
            let response: ResponseInfo = ResponseInfo { 
                status: success(), 
                msg: path
            };
            return response
        }
        Err(e) => {
            let response: ResponseInfo = ResponseInfo {
                status: error(),
                msg: e.to_string()
            };
            return response
        }
    }
}

#[tauri::command]
pub fn move_warehouse(path: String, name: String, id: u64, new_path: String) -> ResponseInfo {
    let path_total: String = format!("{}{}{}", path, "/", name);
    let path_new: String = format!("{}{}{}", new_path, "/", name);
    let warehouse_path: &Path = Path::new(&path_new);
    if warehouse_path.exists() {
        let response: ResponseInfo = ResponseInfo { 
            status: error(), 
            msg: "路径下已有同名文件夹".to_string() };
        return response
    }
    match fs::rename(&path_total, &path_new) {
        Ok(_) => {
            update_warehouse_json(id, name, new_path.clone(), "move".to_string());
            let response: ResponseInfo = ResponseInfo { 
                status: success(), 
                msg: new_path 
            };
            return response
        }
        Err(e) => {
            let response: ResponseInfo = ResponseInfo {
                status: error(),
                msg: e.to_string()
            };
            return response
        }
    }
}


pub fn create_json_dir(path: String) {
    let name: String = ".simple_write".to_string();
    let path_total: String = format!("{}{}{}", path, "/", name);
    match fs::create_dir(&path_total) {
        Ok(_) => {
            // TODO
        }
        Err(e) => {
            println!("create_json_dir {}", e)
        }
    }
}
