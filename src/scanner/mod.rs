mod project;
mod component;
mod route;
mod api;
mod store_scan;

use anyhow::Result;
use std::path::Path;
use crate::model::FrontendMap;

pub fn scan(project_path: &str) -> Result<FrontendMap> {
    let root = Path::new(project_path);
    
    // Scan project info
    let mut project_info = project::scan_project(root)?;
    
    // Scan components
    let components = component::scan_components(root)?;
    
    // Update component count
    project_info.component_count = components.len();
    
    // Scan routes
    let routes = route::scan_routes(root, &project_info.framework)?;
    
    // Scan API calls
    let api_calls = api::scan_api_calls(root)?;
    
    // Scan stores
    let stores = store_scan::scan_stores(root)?;
    
    Ok(FrontendMap {
        schema_version: 1,
        project: project_info,
        components,
        routes,
        api_calls,
        stores,
    })
}
