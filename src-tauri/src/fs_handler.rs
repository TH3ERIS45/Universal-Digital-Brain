use walkdir::WalkDir;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

pub fn scan_directory<P: AsRef<Path>>(root: P) -> Vec<FileInfo> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().is_dir();
        
        // Skip hidden files/dirs (starting with .)
        if name.starts_with('.') {
            continue;
        }

        files.push(FileInfo {
            path: path.to_string_lossy().to_string(),
            name,
            is_dir,
        });
    }
    
    files
}
