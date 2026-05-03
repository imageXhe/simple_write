use std::fs::File;

const JSON_PATH: &str = ".simple_write/setting.json";

pub fn create_setting_json(path: String) {
    let path_total: String = format!("{}{}{}", path, "/", JSON_PATH);
    match File::create(&path_total) {
        Ok(_) => {
            // TODO
        }
        Err(e) => {
            println!("create_setting_json {}", e)
        }
    }
}

// fn write_setting_json(path: String) {
    
// }

