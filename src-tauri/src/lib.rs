mod commands;
mod db;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                let pool = db::init_db(&app_handle).await;
                app_handle.manage(pool);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::set_active_project,
            commands::get_active_project,
            commands::create_counter,
            commands::list_counters,
            commands::increment_counter,
            commands::start_session,
            commands::end_session,
            commands::list_sessions,
            commands::add_pattern,
            commands::list_patterns,
            commands::delete_pattern,
            commands::add_inventory_item,
            commands::list_inventory_items,
            commands::delete_inventory_item,
            commands::link_inventory_to_project,
            commands::list_project_inventory,
            commands::remove_project_inventory_link,
            commands::update_project,
            commands::delete_project,
            commands::update_counter,
            commands::delete_counter,
            commands::update_inventory_item,
            commands::update_project_inventory_link,
            commands::get_shopping_list,
            commands::update_pattern_metadata,
            commands::update_session_notes,
            commands::export_mobile_data,
            commands::import_mobile_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}