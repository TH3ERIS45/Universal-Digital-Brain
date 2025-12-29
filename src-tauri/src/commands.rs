use crate::db::Database;
use crate::fs_handler;
use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use serde::Serialize;
use rusqlite::{params, OptionalExtension};


#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn scan_vault(path: String, db: State<Database>) -> Result<String, String> {
    let files = fs_handler::scan_directory(&path);
    let conn = db.conn.lock().unwrap();
    
    let mut count = 0;
    for file in files {
        if file.is_dir { continue; } 
        
        let resource_type = if file.name.ends_with(".md") { "note" } else { "file" };
        
        // Check if resource exists by path
        let existing_id: Option<String> = conn.query_row(
            "SELECT id FROM resources WHERE path = ?",
            params![&file.path],
            |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;

        let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let created_at = Utc::now().to_rfc3339();
        
        // Read Content if it's a note
        let content = if resource_type == "note" {
             std::fs::read_to_string(&file.path).unwrap_or_default()
        } else {
             String::new()
        };

        conn.execute(
            "INSERT INTO resources (id, type, path, title, content, created_at, updated_at, extra_metadata) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
            params![
                &id, 
                resource_type, 
                &file.path, 
                &file.name,
                &content,
                &created_at,
                &created_at,
                Option::<String>::None
            ]
        ).map_err(|e| e.to_string())?;
        
        count += 1;
    }

    Ok(format!("Scanned {} files", count))
}

#[derive(Serialize)]
pub struct Resource {
    id: String,
    path: Option<String>,
    title: String,
    #[serde(rename = "type")]
    resource_type: String,
    extra_metadata: Option<String>,
}

#[tauri::command]
pub fn get_all_resources(db: State<Database>) -> Result<Vec<Resource>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, path, title, type, extra_metadata FROM resources ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    
    let iter = stmt.query_map([], |row| {
        Ok(Resource {
            id: row.get(0)?,
            path: row.get(1).ok(), 
            title: row.get(2)?,
            resource_type: row.get(3)?,
            extra_metadata: row.get(4).ok(),
        })
    }).map_err(|e| e.to_string())?;

    let mut resources = Vec::new();
    for res in iter {
        resources.push(res.map_err(|e| e.to_string())?);
    }
    
    Ok(resources)
}

#[tauri::command]
pub fn create_link(title: String, url: String, db: State<Database>) -> Result<String, String> {
    let conn = db.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    let metadata = serde_json::json!({ "url": url }).to_string();

    conn.execute(
        "INSERT INTO resources (id, type, title, created_at, updated_at, extra_metadata) VALUES (?, 'link', ?, ?, ?, ?)",
        params![&id, &title, &created_at, &created_at, &metadata],
    ).map_err(|e| e.to_string())?;
    
    Ok(id)
}

#[tauri::command]
pub fn create_task(title: String, db: State<Database>) -> Result<String, String> {
    let conn = db.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    let metadata = serde_json::json!({ "status": "todo" }).to_string();

    conn.execute(
        "INSERT INTO resources (id, type, title, created_at, updated_at, extra_metadata) VALUES (?, 'task', ?, ?, ?, ?)",
        params![&id, &title, &created_at, &created_at, &metadata],
    ).map_err(|e| e.to_string())?;
    
    Ok(id)
}

#[derive(Serialize)]
pub struct GraphNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    source: String,
    target: String,
}

#[derive(Serialize)]
pub struct GraphData {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[tauri::command]
pub fn get_graph_data(db: State<Database>) -> Result<GraphData, String> {
    let conn = db.conn.lock().unwrap();
    
    // Nodes
    let mut stmt = conn.prepare("SELECT id, title, type FROM resources").map_err(|e| e.to_string())?;
    let nodes = stmt.query_map([], |row| {
        Ok(GraphNode {
            id: row.get(0)?,
            label: row.get(1)?,
            node_type: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Edges
    let mut stmt = conn.prepare("SELECT source_id, target_id FROM links").map_err(|e| e.to_string())?;
    let edges = stmt.query_map([], |row| {
        Ok(GraphEdge {
            source: row.get(0)?,
            target: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(GraphData { nodes, edges })
}

#[tauri::command]
pub fn create_note(title: String, content: String, db: State<Database>) -> Result<String, String> {
    let conn = db.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    // 1. Determine Filename and Path (Hardcoded vault path for now, should be cleaner in v2)
    // Sanitize title for filename
    let sanitized_title: String = title.chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect();
    let filename = format!("{}.md", sanitized_title.trim());
    let path = format!("/home/thyeris/Documents/NewProject/vault/{}", filename); // TODO: Get root from config/state

    // 2. Write to File System
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;

    // 3. Insert to DB
    conn.execute(
        "INSERT INTO resources (id, type, path, title, content, created_at, updated_at) VALUES (?, 'note', ?, ?, ?, ?, ?)",
        params![&id, &path, &title, &content, &created_at, &created_at],
    ).map_err(|e| e.to_string())?;
    
    Ok(id)
}

#[tauri::command]
pub fn update_note(id: String, title: String, content: String, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    let updated_at = Utc::now().to_rfc3339();
    
    // 1. Get existing path
    let path: String = conn.query_row(
        "SELECT path FROM resources WHERE id = ?",
        params![&id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // 2. Write to File System
    if !path.is_empty() {
         std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    }

    // 3. Update DB
    conn.execute(
        "UPDATE resources SET title = ?, content = ?, updated_at = ? WHERE id = ?",
        params![&title, &content, &updated_at, &id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_note(id: String, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    
    // 1. Get path to delete file
    let path: Option<String> = conn.query_row(
        "SELECT path FROM resources WHERE id = ?",
        params![&id],
        |row| row.get(0),
    ).optional().map_err(|e| e.to_string())?;

    // 2. Delete from FS
    if let Some(p) = path {
        if !p.is_empty() {
            let _ = std::fs::remove_file(p); // Ignore error if file missing
        }
    }

    // 3. Delete from DB
    conn.execute(
        "DELETE FROM resources WHERE id = ?",
        params![&id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[derive(Serialize)]
pub struct NoteContent {
    id: String,
    title: String,
    content: String,
}

#[tauri::command]
pub fn get_note_content(id: String, db: State<Database>) -> Result<NoteContent, String> {
    let conn = db.conn.lock().unwrap();
    
    // We prioritize the DB content since we sync it on write/scan.
    let mut stmt = conn.prepare("SELECT title, content FROM resources WHERE id = ?").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![&id]).map_err(|e| e.to_string())?;
    
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(NoteContent {
            id,
            title: row.get(0).unwrap_or_default(),
            content: row.get(1).unwrap_or_default(),
        })
    } else {
        Err("Note not found".to_string())
    }
}
