mod menu;
mod readme_template;
use menu::project::{create_warehouse, open_warehouse, delete_warehouse, rename_warehouse, move_warehouse};
use menu::warehouse_json::{init_warehouse_json, get_warehouse_json};
use menu::bookmark::{get_bookmarks_json, add_bookmark, remove_bookmark};
use menu::favorite::{get_favorites_json, add_favorite, remove_favorite};
use menu::link_json::{scan_links, get_outgoing_links, get_incoming_links};
use menu::graph_json::{read_graph_config, write_graph_config};

mod file_list;
use file_list::note::{
    get_file_json,
    create_file,
    get_file_content,
    read_file_as_base64,
    save_file_content,
    duplicate_file_entry,
    rename_file_entry,
    move_file_entry,
    delete_file_entry,
    generate_readme,
    test,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("{}1", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        
        .invoke_handler(tauri::generate_handler![ 
            greet,
            create_warehouse, open_warehouse, delete_warehouse, rename_warehouse, move_warehouse,
            init_warehouse_json, get_warehouse_json,
            get_bookmarks_json, add_bookmark, remove_bookmark,
            get_favorites_json, add_favorite, remove_favorite,
            scan_links, get_outgoing_links, get_incoming_links,
            read_graph_config, write_graph_config,
            get_file_json, create_file, get_file_content, read_file_as_base64, save_file_content,
            duplicate_file_entry, rename_file_entry, move_file_entry, delete_file_entry,
            generate_readme,
            test
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
