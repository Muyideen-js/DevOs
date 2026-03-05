use std::path::PathBuf;

/// Get the app data directory for DevOS config.
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("devos")
}

/// Load recent projects from config file.
pub fn load_recent_projects() -> Vec<PathBuf> {
    let path = config_dir().join("recent.json");
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Save recent projects to config file.
pub fn save_recent_projects(projects: &[PathBuf]) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create config dir: {}", e))?;

    let path = dir.join("recent.json");
    let json =
        serde_json::to_string_pretty(projects).map_err(|e| format!("JSON error: {}", e))?;

    std::fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

/// Add a project to the recent list (max 10, dedup, newest first).
pub fn add_recent_project(projects: &mut Vec<PathBuf>, path: PathBuf) {
    projects.retain(|p| p != &path);
    projects.insert(0, path);
    projects.truncate(10);
}
