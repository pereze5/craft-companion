use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::fs;
use tauri::Manager;

pub async fn init_db(app: &tauri::AppHandle) -> SqlitePool {
    println!("INIT DB RUNNING");

    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("craft_companion.db");
    println!("DB PATH: {:?}", db_path);

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to connect to SQLite");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            craft_type TEXT NOT NULL,
            status TEXT DEFAULT 'active',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS project_state (
            id INTEGER PRIMARY KEY,
            active_project_id INTEGER,
            last_opened_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(active_project_id) REFERENCES projects(id)
        );

        INSERT OR IGNORE INTO project_state (id, active_project_id)
        VALUES (1, NULL);

        CREATE TABLE IF NOT EXISTS counters (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id INTEGER NOT NULL,
        label TEXT NOT NULL,
        value INTEGER NOT NULL DEFAULT 0,
        counter_type TEXT DEFAULT 'row',
        sort_order INTEGER DEFAULT 0,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_counters_project_id
        ON counters(project_id);

        CREATE TABLE IF NOT EXISTS sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id INTEGER NOT NULL,
        started_at TEXT DEFAULT CURRENT_TIMESTAMP,
        ended_at TEXT,
        notes TEXT DEFAULT '',
        current_page INTEGER DEFAULT 1,
        position_note TEXT DEFAULT '',
        counter_snapshot_json TEXT DEFAULT '{}',
        pattern_id INTEGER,
        FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_project_id
        ON sessions(project_id);

        CREATE TABLE IF NOT EXISTS patterns (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id INTEGER NOT NULL,
        display_name TEXT NOT NULL,
        file_path TEXT NOT NULL,
        file_type TEXT NOT NULL,
        current_page INTEGER DEFAULT 1,
        last_position_note TEXT DEFAULT '',
        last_viewed_at TEXT DEFAULT CURRENT_TIMESTAMP,
        metadata_json TEXT DEFAULT '{}',
        FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_patterns_project_id
        ON patterns(project_id);

        CREATE TABLE IF NOT EXISTS inventory_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        supply_type TEXT NOT NULL,
        brand TEXT DEFAULT '',
        color_name TEXT DEFAULT '',
        color_code TEXT DEFAULT '',
        quantity REAL DEFAULT 1,
        unit TEXT DEFAULT '',
        notes TEXT DEFAULT '',
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE INDEX IF NOT EXISTS idx_inventory_items_supply_type
        ON inventory_items(supply_type);

        CREATE TABLE IF NOT EXISTS project_inventory (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id INTEGER NOT NULL,
        inventory_item_id INTEGER NOT NULL,
        quantity_allocated REAL DEFAULT 1,
        notes TEXT DEFAULT '',
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
        FOREIGN KEY(inventory_item_id) REFERENCES inventory_items(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_project_inventory_project_id
        ON project_inventory(project_id);

        CREATE INDEX IF NOT EXISTS idx_project_inventory_inventory_item_id
        ON project_inventory(inventory_item_id);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize database schema");

    pool
}
