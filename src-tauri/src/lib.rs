mod db;
mod fs_handler;
mod commands;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Use local file for prototype
            let db_path = "brain.db"; 
            let db = Database::init(db_path).expect("failed to init db");
            
            app.manage(db);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet, 
            commands::scan_vault, 
            commands::get_all_resources,
            commands::get_graph_data,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_note_content,
            commands::create_link,
            commands::create_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
