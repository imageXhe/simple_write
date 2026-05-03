use serde::{Serialize, Deserialize};
use serde_json::{from_str, to_string};
use std::path::Path;
use std::fs;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectJson {
    num: u64,
    project_list: Vec<Project>
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    path: String,
    id: u64
}

const APPDATA_PATH: &str = concat!(env!("APPDATA"), "/simple_write");
const JSON_PATH: &str = concat!(env!("APPDATA"), "/simple_write/project.json");

#[tauri::command]
pub fn init_warehouse_json() -> String {
    let path: &Path = Path::new(JSON_PATH);
    if !path.exists() {
        fs::create_dir(&APPDATA_PATH).unwrap();
        match fs::File::create(path) {
            Ok(_) => {
                let json: ProjectJson = ProjectJson {
                    num: 0,
                    project_list: Vec::new()
                };
                let json_string = to_string(&json).unwrap();
                fs::write(JSON_PATH, json_string).unwrap();
                return "init_warehouse_json SUCCESS".to_string();
            }
            Err(e) => {
                println!(" {}", path.display());
                println!("init_warehouse_json {}", e);
                return "ERROR!!!   init_warehouse_json ERROR".to_string();
            }
        }
    }else {
        return "init_warehouse_json SUCCESS".to_string();
    }
}

#[tauri::command]
pub fn get_warehouse_json() -> ProjectJson {
    let project_json: String = fs::read_to_string(JSON_PATH).unwrap();
    let parsed_json: ProjectJson = from_str(&project_json).unwrap();
    return parsed_json;
}

pub fn update_warehouse_json(project_id: u64, project_name: String, project_path: String, operation: String) {
    //operation: create, delete, rename, move
    let mut project_json = get_warehouse_json();
    if operation == "create" {
        project_json.project_list.push(Project {
            name: project_name,
            path: project_path,
            id: project_id
        });
        project_json.num += 1;
    }else if operation == "delete" {
        project_json.project_list.retain(|x: &Project| x.id != project_id);
        project_json.num -= 1;
    }else if operation == "rename" {
        for project in &mut project_json.project_list {
            if project.id == project_id {
                project.name = project_name;
                break;
            }
        }
    }else if operation == "move" {
        for project in &mut project_json.project_list {
            if project.id == project_id {
                project.path = project_path;
                break;
            }
        }
    }
    let json_string = to_string(&project_json).unwrap();
    fs::write(JSON_PATH, json_string).unwrap();
    println!("{}_warehouse_json SUCCESS", operation);
}

