use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON;", [])?;
        
        let db = Database { conn: Mutex::new(conn) };
        db.create_tables()?;
        
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Resources Table: Unified store for Notes, Files, Links, Tasks
        conn.execute(
            "CREATE TABLE IF NOT EXISTS resources (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL, -- 'note', 'file', 'link', 'task'
                path TEXT,          -- Nullable for non-file resources
                title TEXT,
                content TEXT,       -- Markdown content or Description
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                extra_metadata TEXT -- JSON blob for extended props
            )",
            [],
        )?;

        // Tags Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;

        // Resource <-> Tags (Many-to-Many)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS resource_tags (
                resource_id TEXT,
                tag_id INTEGER,
                PRIMARY KEY (resource_id, tag_id),
                FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Links Table: Graph Edges (Bidirectional / Directed)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
                source_id TEXT,
                target_id TEXT,
                type TEXT, -- 'wikilink', 'reference', 'parent'
                PRIMARY KEY (source_id, target_id),
                FOREIGN KEY(source_id) REFERENCES resources(id) ON DELETE CASCADE,
                FOREIGN KEY(target_id) REFERENCES resources(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }
}
