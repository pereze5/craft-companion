use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub craft_type: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Counter {
    pub id: i64,
    pub project_id: i64,
    pub label: String,
    pub value: i64,
    pub counter_type: String,
    pub sort_order: i64,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i64,
    pub project_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub notes: String,
    pub current_page: i64,
    pub position_note: String,
    pub counter_snapshot_json: String,
    pub pattern_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pattern {
    pub id: i64,
    pub project_id: i64,
    pub display_name: String,
    pub file_path: String,
    pub file_type: String,
    pub current_page: i64,
    pub last_position_note: String,
    pub last_viewed_at: String,
    pub metadata_json: String,
}

#[tauri::command]
pub async fn create_project(
    pool: tauri::State<'_, SqlitePool>,
    name: String,
    craft_type: String,
) -> Result<Project, String> {
    let result = sqlx::query(
        r#"
        INSERT INTO projects (name, craft_type)
        VALUES (?1, ?2)
        "#,
    )
    .bind(name)
    .bind(craft_type)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    sqlx::query_as::<_, Project>(
        r#"
        SELECT id, name, craft_type, status, created_at
        FROM projects
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_projects(pool: tauri::State<'_, SqlitePool>) -> Result<Vec<Project>, String> {
    sqlx::query_as::<_, Project>(
        r#"
        SELECT id, name, craft_type, status, created_at
        FROM projects
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_project(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE project_state
        SET active_project_id = ?1,
            last_opened_at = CURRENT_TIMESTAMP
        WHERE id = 1
        "#,
    )
    .bind(project_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_active_project(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Option<Project>, String> {
    sqlx::query_as::<_, Project>(
        r#"
        SELECT p.id, p.name, p.craft_type, p.status, p.created_at
        FROM projects p
        JOIN project_state s ON s.active_project_id = p.id
        WHERE s.id = 1
        "#,
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_counter(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
    label: String,
    counter_type: String,
) -> Result<Counter, String> {
    let result = sqlx::query(
        r#"
        INSERT INTO counters (project_id, label, counter_type)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(project_id)
    .bind(label)
    .bind(counter_type)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    sqlx::query_as::<_, Counter>(
        r#"
        SELECT id, project_id, label, value, counter_type, sort_order, updated_at
        FROM counters
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_counters(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<Vec<Counter>, String> {
    sqlx::query_as::<_, Counter>(
        r#"
        SELECT id, project_id, label, value, counter_type, sort_order, updated_at
        FROM counters
        WHERE project_id = ?1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn increment_counter(
    pool: tauri::State<'_, SqlitePool>,
    counter_id: i64,
    delta: i64,
) -> Result<Counter, String> {
    sqlx::query(
        r#"
        UPDATE counters
        SET value = value + ?1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?2
        "#,
    )
    .bind(delta)
    .bind(counter_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Counter>(
        r#"
        SELECT id, project_id, label, value, counter_type, sort_order, updated_at
        FROM counters
        WHERE id = ?1
        "#,
    )
    .bind(counter_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_session(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<Session, String> {
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (project_id)
        VALUES (?1)
        "#,
    )
    .bind(project_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, project_id, started_at, ended_at, notes,
               current_page, position_note,
               counter_snapshot_json, pattern_id
        FROM sessions
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn end_session(
    pool: tauri::State<'_, SqlitePool>,
    session_id: i64,
    notes: String,
    current_page: i64,
    position_note: String,
    pattern_id: Option<i64>,
    counter_snapshot_json: String,
) -> Result<Session, String> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET ended_at = CURRENT_TIMESTAMP,
            notes = ?1,
            current_page = ?2,
            position_note = ?3,
            pattern_id = ?4,
            counter_snapshot_json = ?5
        WHERE id = ?6
        "#,
    )
    .bind(&notes)
    .bind(current_page)
    .bind(&position_note)
    .bind(pattern_id)
    .bind(&counter_snapshot_json)
    .bind(session_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if let Some(pattern_id_value) = pattern_id {
        sqlx::query(
            r#"
            UPDATE patterns
            SET current_page = ?1,
                last_position_note = ?2,
                last_viewed_at = CURRENT_TIMESTAMP
            WHERE id = ?3
            "#,
        )
        .bind(current_page)
        .bind(&position_note)
        .bind(pattern_id_value)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    }

    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, project_id, started_at, ended_at, notes,
               current_page, position_note,
               counter_snapshot_json, pattern_id
        FROM sessions
        WHERE id = ?1
        "#,
    )
    .bind(session_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<Vec<Session>, String> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, project_id, started_at, ended_at, notes,
               current_page, position_note,
               counter_snapshot_json, pattern_id
        FROM sessions
        WHERE project_id = ?1
        ORDER BY started_at DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_pattern(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
    display_name: String,
    file_path: String,
    file_type: String,
) -> Result<Pattern, String> {
    let result = sqlx::query(
        r#"
        INSERT INTO patterns (project_id, display_name, file_path, file_type)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(project_id)
    .bind(display_name)
    .bind(file_path)
    .bind(file_type)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    sqlx::query_as::<_, Pattern>(
        r#"
        SELECT id, project_id, display_name, file_path, file_type,
               current_page, last_position_note, last_viewed_at, metadata_json
        FROM patterns
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_patterns(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<Vec<Pattern>, String> {
    sqlx::query_as::<_, Pattern>(
        r#"
        SELECT id, project_id, display_name, file_path, file_type,
               current_page, last_position_note, last_viewed_at, metadata_json
        FROM patterns
        WHERE project_id = ?1
        ORDER BY last_viewed_at DESC, id DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_pattern(
    pool: tauri::State<'_, SqlitePool>,
    pattern_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM patterns
        WHERE id = ?1
        "#,
    )
    .bind(pattern_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_pattern_position(
    pool: tauri::State<'_, SqlitePool>,
    pattern_id: i64,
    current_page: i64,
    last_position_note: String,
) -> Result<Pattern, String> {
    sqlx::query(
        r#"
        UPDATE patterns
        SET current_page = ?1,
            last_position_note = ?2,
            last_viewed_at = CURRENT_TIMESTAMP
        WHERE id = ?3
        "#,
    )
    .bind(current_page)
    .bind(last_position_note)
    .bind(pattern_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Pattern>(
        r#"
        SELECT id, project_id, display_name, file_path, file_type,
               current_page, last_position_note, last_viewed_at, metadata_json
        FROM patterns
        WHERE id = ?1
        "#,
    )
    .bind(pattern_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: i64,
    pub name: String,
    pub supply_type: String,
    pub brand: String,
    pub color_name: String,
    pub color_code: String,
    pub quantity: f64,
    pub unit: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn add_inventory_item(
    pool: tauri::State<'_, SqlitePool>,
    name: String,
    supply_type: String,
    brand: String,
    color_name: String,
    color_code: String,
    quantity: f64,
    unit: String,
    notes: String,
) -> Result<InventoryItem, String> {
    let result = sqlx::query(
        r#"
        INSERT INTO inventory_items
        (name, supply_type, brand, color_name, color_code, quantity, unit, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(name)
    .bind(supply_type)
    .bind(brand)
    .bind(color_name)
    .bind(color_code)
    .bind(quantity)
    .bind(unit)
    .bind(notes)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT id, name, supply_type, brand, color_name, color_code,
               quantity, unit, notes, created_at, updated_at
        FROM inventory_items
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_inventory_items(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<InventoryItem>, String> {
    sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT id, name, supply_type, brand, color_name, color_code,
               quantity, unit, notes, created_at, updated_at
        FROM inventory_items
        ORDER BY supply_type ASC, name ASC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_inventory_item(
    pool: tauri::State<'_, SqlitePool>,
    item_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM inventory_items
        WHERE id = ?1
        "#,
    )
    .bind(item_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProjectInventoryItem {
    pub id: i64,
    pub project_id: i64,
    pub inventory_item_id: i64,
    pub quantity_allocated: f64,
    pub notes: String,
    pub item_name: String,
    pub supply_type: String,
    pub brand: String,
    pub color_name: String,
    pub color_code: String,
    pub unit: String,
}

#[tauri::command]
pub async fn link_inventory_to_project(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
    inventory_item_id: i64,
    quantity_allocated: f64,
    notes: String,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO project_inventory
        (project_id, inventory_item_id, quantity_allocated, notes)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(project_id)
    .bind(inventory_item_id)
    .bind(quantity_allocated)
    .bind(notes)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_project_inventory(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<Vec<ProjectInventoryItem>, String> {
    sqlx::query_as::<_, ProjectInventoryItem>(
        r#"
        SELECT
            pi.id,
            pi.project_id,
            pi.inventory_item_id,
            pi.quantity_allocated,
            pi.notes,
            ii.name AS item_name,
            ii.supply_type,
            ii.brand,
            ii.color_name,
            ii.color_code,
            ii.unit
        FROM project_inventory pi
        JOIN inventory_items ii ON ii.id = pi.inventory_item_id
        WHERE pi.project_id = ?1
        ORDER BY ii.supply_type ASC, ii.name ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_project_inventory_link(
    pool: tauri::State<'_, SqlitePool>,
    project_inventory_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM project_inventory
        WHERE id = ?1
        "#,
    )
    .bind(project_inventory_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_project(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
    name: String,
    craft_type: String,
    status: String,
) -> Result<Project, String> {
    sqlx::query(
        r#"
        UPDATE projects
        SET name = ?1,
            craft_type = ?2,
            status = ?3
        WHERE id = ?4
        "#,
    )
    .bind(name)
    .bind(craft_type)
    .bind(status)
    .bind(project_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Project>(
        r#"
        SELECT id, name, craft_type, status, created_at
        FROM projects
        WHERE id = ?1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(
    pool: tauri::State<'_, SqlitePool>,
    project_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM projects
        WHERE id = ?1
        "#,
    )
    .bind(project_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE project_state
        SET active_project_id = NULL
        WHERE active_project_id = ?1
        "#,
    )
    .bind(project_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_counter(
    pool: tauri::State<'_, SqlitePool>,
    counter_id: i64,
    label: String,
    value: i64,
    counter_type: String,
) -> Result<Counter, String> {
    sqlx::query(
        r#"
        UPDATE counters
        SET label = ?1,
            value = ?2,
            counter_type = ?3,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?4
        "#,
    )
    .bind(label)
    .bind(value)
    .bind(counter_type)
    .bind(counter_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Counter>(
        r#"
        SELECT id, project_id, label, value, counter_type, sort_order, updated_at
        FROM counters
        WHERE id = ?1
        "#,
    )
    .bind(counter_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_counter(
    pool: tauri::State<'_, SqlitePool>,
    counter_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM counters
        WHERE id = ?1
        "#,
    )
    .bind(counter_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_inventory_item(
    pool: tauri::State<'_, SqlitePool>,
    item_id: i64,
    name: String,
    supply_type: String,
    brand: String,
    color_name: String,
    color_code: String,
    quantity: f64,
    unit: String,
    notes: String,
) -> Result<InventoryItem, String> {
    sqlx::query(
        r#"
        UPDATE inventory_items
        SET name = ?1,
            supply_type = ?2,
            brand = ?3,
            color_name = ?4,
            color_code = ?5,
            quantity = ?6,
            unit = ?7,
            notes = ?8,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?9
        "#,
    )
    .bind(name)
    .bind(supply_type)
    .bind(brand)
    .bind(color_name)
    .bind(color_code)
    .bind(quantity)
    .bind(unit)
    .bind(notes)
    .bind(item_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT id, name, supply_type, brand, color_name, color_code,
               quantity, unit, notes, created_at, updated_at
        FROM inventory_items
        WHERE id = ?1
        "#,
    )
    .bind(item_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_project_inventory_link(
    pool: tauri::State<'_, SqlitePool>,
    project_inventory_id: i64,
    quantity_allocated: f64,
    notes: String,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE project_inventory
        SET quantity_allocated = ?1,
            notes = ?2
        WHERE id = ?3
        "#,
    )
    .bind(quantity_allocated)
    .bind(notes)
    .bind(project_inventory_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ShoppingListItem {
    pub inventory_item_id: i64,
    pub item_name: String,
    pub supply_type: String,
    pub brand: String,
    pub color_name: String,
    pub color_code: String,
    pub inventory_quantity: f64,
    pub allocated_quantity: f64,
    pub shortage_quantity: f64,
    pub unit: String,
}

#[tauri::command]
pub async fn get_shopping_list(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<ShoppingListItem>, String> {
    sqlx::query_as::<_, ShoppingListItem>(
        r#"
        SELECT
            ii.id AS inventory_item_id,
            ii.name AS item_name,
            ii.supply_type,
            ii.brand,
            ii.color_name,
            ii.color_code,
            ii.quantity AS inventory_quantity,
            COALESCE(SUM(pi.quantity_allocated), 0) AS allocated_quantity,
            COALESCE(SUM(pi.quantity_allocated), 0) - ii.quantity AS shortage_quantity,
            ii.unit
        FROM inventory_items ii
        LEFT JOIN project_inventory pi ON pi.inventory_item_id = ii.id
        GROUP BY ii.id
        HAVING shortage_quantity > 0
        ORDER BY ii.supply_type ASC, ii.name ASC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_pattern_metadata(
    pool: tauri::State<'_, SqlitePool>,
    pattern_id: i64,
    metadata_json: String,
) -> Result<Pattern, String> {
    sqlx::query(
        r#"
        UPDATE patterns
        SET metadata_json = ?1,
            last_viewed_at = CURRENT_TIMESTAMP
        WHERE id = ?2
        "#,
    )
    .bind(metadata_json)
    .bind(pattern_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Pattern>(
        r#"
        SELECT id, project_id, display_name, file_path, file_type,
               current_page, last_position_note, last_viewed_at, metadata_json
        FROM patterns
        WHERE id = ?1
        "#,
    )
    .bind(pattern_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_session_notes(
    pool: tauri::State<'_, SqlitePool>,
    session_id: i64,
    notes: String,
) -> Result<Session, String> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET notes = ?1
        WHERE id = ?2
        "#,
    )
    .bind(notes)
    .bind(session_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, project_id, started_at, ended_at, notes,
               current_page, position_note,
               counter_snapshot_json, pattern_id
        FROM sessions
        WHERE id = ?1
        "#,
    )
    .bind(session_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_mobile_data(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<String, String> {
    let export_dir = PathBuf::from("mobile-export");
    let patterns_dir = export_dir.join("patterns");

    fs::create_dir_all(&patterns_dir)
        .map_err(|e| e.to_string())?;

    let projects = sqlx::query_as::<_, Project>(
        r#"
        SELECT id, name, craft_type, status, created_at
        FROM projects
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let counters = sqlx::query_as::<_, Counter>(
        r#"
        SELECT id, project_id, label, value, counter_type, sort_order, updated_at
        FROM counters
        ORDER BY project_id, sort_order, id
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let patterns = sqlx::query_as::<_, Pattern>(
        r#"
        SELECT id, project_id, display_name, file_path, file_type,
               current_page, last_position_note, last_viewed_at, metadata_json
        FROM patterns
        ORDER BY project_id, last_viewed_at DESC, id DESC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let sessions = sqlx::query_as::<_, Session>(
        r#"
        SELECT id, project_id, started_at, ended_at, notes,
               current_page, position_note, counter_snapshot_json, pattern_id
        FROM sessions
        ORDER BY project_id, started_at DESC, id DESC
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut mobile_patterns = Vec::new();

    for pattern in patterns {
        let mut mobile_file = serde_json::Value::Null;

        if pattern.file_type == "pdf" || pattern.file_type == "image" {
            let source_path = PathBuf::from(&pattern.file_path);

            if source_path.exists() {
                let extension = source_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("file");

                let mobile_filename =
                    format!("pattern-{}.{}", pattern.id, extension);

                let destination_path =
                    patterns_dir.join(&mobile_filename);

                fs::copy(&source_path, &destination_path)
                    .map_err(|e| e.to_string())?;

                mobile_file = json!(
                    format!("patterns/{}", mobile_filename)
                );
            }
        }

        mobile_patterns.push(json!({
            "id": pattern.id,
            "project_id": pattern.project_id,
            "display_name": pattern.display_name,
            "file_path": pattern.file_path,
            "file_type": pattern.file_type,
            "current_page": pattern.current_page,
            "last_position_note": pattern.last_position_note,
            "last_viewed_at": pattern.last_viewed_at,
            "metadata_json": pattern.metadata_json,
            "mobile_file": mobile_file
        }));
    }

    let export_data = json!({
        "export_version": 1,
        "projects": projects,
        "patterns": mobile_patterns,
        "sessions": sessions,
        "counters": counters
    });

    let json_path = export_dir.join("data.json");

        fs::write(
        &json_path,
        serde_json::to_string_pretty(&export_data)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let zip_path = std::env::current_dir()
    .map_err(|e| e.to_string())?
    .join("mobile-export.zip");

    let output = Command::new("powershell")
        .args([
            "-Command",
            "Compress-Archive -Path .\\mobile-export\\* -DestinationPath .\\mobile-export.zip -Force",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(format!(
    "Mobile export created at {}",
    zip_path.display()
))
}

#[tauri::command]
pub async fn import_mobile_data(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::fs::File;
    use std::io::copy;
    use zip::ZipArchive;

    let zip_path =
        PathBuf::from("/sdcard/Download/mobile-export.zip");

    if !zip_path.exists() {
        return Err("mobile-export.zip not found".into());
    }

    let extract_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("mobile-import");

    fs::create_dir_all(&extract_dir)
        .map_err(|e| e.to_string())?;

    let file =
        File::open(&zip_path)
            .map_err(|e| e.to_string())?;

    let mut archive =
        ZipArchive::new(file)
            .map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut zipped =
            archive.by_index(i)
                .map_err(|e| e.to_string())?;

        let outpath =
            extract_dir.join(zipped.name());

        if zipped.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| e.to_string())?;
            }

            let mut outfile =
                File::create(&outpath)
                    .map_err(|e| e.to_string())?;

            copy(&mut zipped, &mut outfile)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(format!(
        "Imported to {}",
        extract_dir.display()
    ))
}