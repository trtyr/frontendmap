use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use crate::model::Route;

pub fn scan_routes(root: &Path) -> Result<Vec<Route>> {
    let mut routes = Vec::new();
    
    // Scan for various router patterns
    scan_react_router(root, &mut routes)?;
    scan_vue_router(root, &mut routes)?;
    scan_angular_router(root, &mut routes)?;
    scan_svelte_router(root, &mut routes)?;
    scan_next_pages(root, &mut routes)?;
    scan_nuxt_pages(root, &mut routes)?;
    scan_sveltekit_routes(root, &mut routes)?;
    
    Ok(routes)
}

fn scan_react_router(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    // Pattern for React Router v6: <Route path="/xxx" element={<Component />} />
    let route_v6_re = Regex::new(r#"<Route\s+[^>]*path\s*=\s*["']([^"']+)["'][^>]*element\s*=\s*\{<(\w+)"#).expect("invalid regex pattern");
    // Pattern for route config objects: { path: '/xxx', element: <Component /> }
    let config_re = Regex::new(r#"\{\s*path\s*:\s*["']([^"']+)["']\s*,\s*element\s*:\s*(?:<)?(\w+)"#).expect("invalid regex pattern");
    // Pattern for createBrowserRouter: { path: '/xxx', element: <Component /> }
    let browser_router_re = Regex::new(r#"\{\s*path\s*:\s*["']([^"']+)["']\s*,\s*element\s*:\s*<(\w+)"#).expect("invalid regex pattern");
    
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        
        let path = entry.path();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(caps) = route_v6_re.captures(line) {
                routes.push(Route {
                    path: caps[1].to_string(),
                    component: caps[2].to_string(),
                    file: path.to_path_buf(),
                    line: line_num + 1,
                });
            } else if let Some(caps) = config_re.captures(line) {
                routes.push(Route {
                    path: caps[1].to_string(),
                    component: caps[2].to_string(),
                    file: path.to_path_buf(),
                    line: line_num + 1,
                });
            } else if let Some(caps) = browser_router_re.captures(line) {
                routes.push(Route {
                    path: caps[1].to_string(),
                    component: caps[2].to_string(),
                    file: path.to_path_buf(),
                    line: line_num + 1,
                });
            }
        }
    }
    
    Ok(())
}

fn scan_vue_router(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    // Pattern for Vue Router array config (multiline)
    let path_re = Regex::new(r#"path\s*:\s*["']([^"']+)["']"#).expect("invalid regex pattern");
    let name_re = Regex::new(r#"name\s*:\s*["'](\w+)["']"#).expect("invalid regex pattern");
    let import_re = Regex::new(r#"import\(['"]([^'"]+)['"]\)"#).expect("invalid regex pattern");
    // Pattern for component reference: component: ComponentName
    let component_re = Regex::new(r#"component\s*:\s*(\w+)"#).expect("invalid regex pattern");
    
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        
        let path = entry.path();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let lines: Vec<&str> = content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            
            if let Some(path_caps) = path_re.captures(line) {
                let route_path = path_caps[1].to_string();
                let mut route_name = String::new();
                let mut component_name = String::new();
                
                // Look for name and import/component in nearby lines
                let search_end = (i + 10).min(lines.len());
                for j in i..search_end {
                    let nearby = lines[j];
                    
                    if route_name.is_empty() {
                        if let Some(name_caps) = name_re.captures(nearby) {
                            route_name = name_caps[1].to_string();
                        }
                    }
                    
                    if component_name.is_empty() {
                        // Try lazy import first
                        if let Some(import_caps) = import_re.captures(nearby) {
                            let import_path = import_caps[1].to_string();
                            component_name = import_path.rsplit('/').next()
                                .unwrap_or(&route_name)
                                .replace(".vue", "")
                                .replace(".ts", "")
                                .replace(".js", "");
                        }
                        // Try component reference
                        else if let Some(comp_caps) = component_re.captures(nearby) {
                            component_name = comp_caps[1].to_string();
                        }
                    }
                    
                    if !route_name.is_empty() && !component_name.is_empty() {
                        break;
                    }
                }
                
                if component_name.is_empty() {
                    component_name = route_name.clone();
                }
                
                if !route_path.is_empty() && !component_name.is_empty() {
                    routes.push(Route {
                        path: route_path,
                        component: component_name,
                        file: path.to_path_buf(),
                        line: i + 1,
                    });
                }
            }
            
            i += 1;
        }
    }
    
    Ok(())
}

fn scan_angular_router(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    // Pattern for Angular routes: { path: 'xxx', component: XxxComponent }
    let route_re = Regex::new(r#"\{\s*path\s*:\s*['"]([^'"]+)['"]\s*,\s*component\s*:\s*(\w+)"#).expect("invalid regex pattern");
    
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        
        let path = entry.path();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(caps) = route_re.captures(line) {
                routes.push(Route {
                    path: caps[1].to_string(),
                    component: caps[2].to_string(),
                    file: path.to_path_buf(),
                    line: line_num + 1,
                });
            }
        }
    }
    
    Ok(())
}

fn scan_svelte_router(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    // Svelte uses file-based routing, handled by scan_sveltekit_routes
    Ok(())
}

fn scan_next_pages(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let pages_dirs = vec![
        root.join("pages"),
        root.join("src").join("pages"),
        root.join("app"),
        root.join("src").join("app"),
    ];
    
    for dir in &pages_dirs {
        if dir.exists() {
            scan_directory_routes(dir, root, routes, "next")?;
        }
    }
    
    Ok(())
}

fn scan_nuxt_pages(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let pages_dirs = vec![
        root.join("pages"),
        root.join("src").join("pages"),
    ];
    
    for dir in &pages_dirs {
        if dir.exists() {
            scan_directory_routes(dir, root, routes, "nuxt")?;
        }
    }
    
    Ok(())
}

fn scan_sveltekit_routes(root: &Path, routes: &mut Vec<Route>) -> Result<()> {
    let routes_dir = root.join("src").join("routes");
    if routes_dir.exists() {
        scan_directory_routes(&routes_dir, root, routes, "sveltekit")?;
    }
    
    Ok(())
}

fn scan_directory_routes(dir: &Path, root: &Path, routes: &mut Vec<Route>, framework: &str) -> Result<()> {
    let walker = ignore::WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(path);
        
        // Skip non-page files
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if filename.starts_with('_') || filename.starts_with('.') {
            continue;
        }
        
        // Check for valid page extensions
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if !matches!(ext.as_str(), "tsx" | "jsx" | "ts" | "js" | "vue" | "svelte") {
                continue;
            }
        } else {
            continue;
        }
        
        // Convert file path to route path
        let route_path = file_to_route_path(relative, framework);
        
        // Extract component name from filename
        let component_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default")
            .to_string();
        
        routes.push(Route {
            path: route_path,
            component: component_name,
            file: path.to_path_buf(),
            line: 1,
        });
    }
    
    Ok(())
}

fn file_to_route_path(relative: &Path, framework: &str) -> String {
    let mut parts = Vec::new();
    
    for component in relative.components() {
        let part = component.as_os_str().to_string_lossy();
        
        // Skip file extension
        if part.contains('.') {
            let name = part.rsplit('.').next().unwrap_or(&part);
            if name != "index" {
                parts.push(name.to_string());
            }
        } else {
            parts.push(part.to_string());
        }
    }
    
    // Handle dynamic routes based on framework
    let path = parts.join("/");
    let path = match framework {
        "next" | "nuxt" | "sveltekit" => {
            // [id] -> :id, [...slug] -> *slug
            let dynamic_re = Regex::new(r"\[(\w+)\]").expect("invalid regex pattern");
            let catch_all_re = Regex::new(r"\[\.\.\.(\w+)\]").expect("invalid regex pattern");
            let path = dynamic_re.replace_all(&path, ":$1").to_string();
            catch_all_re.replace_all(&path, "*$1").to_string()
        }
        _ => path,
    };
    
    if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path)
    }
}
