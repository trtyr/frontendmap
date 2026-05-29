use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use crate::model::FrontendMap;

pub fn save(map: &FrontendMap, output_path: &str) -> Result<()> {
    let path = Path::new(output_path);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(map)?;
    fs::write(path, json)?;
    
    Ok(())
}

pub fn load(project_path: &str) -> Result<FrontendMap> {
    let root = Path::new(project_path);
    let map_path = root.join(".frontendmap").join("map.json");
    
    if !map_path.exists() {
        anyhow::bail!("Map file not found at {}. Run 'frontendmap index' first.", map_path.display());
    }
    
    let content = fs::read_to_string(&map_path)
        .context("Failed to read map file")?;
    
    let map: FrontendMap = serde_json::from_str(&content)
        .context("Failed to parse map file")?;
    
    Ok(map)
}
