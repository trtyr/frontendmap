use crate::model;
use std::collections::{HashSet, VecDeque, HashMap};

pub fn query_entries(map: &model::FrontendMap) {
    println!("=== Entry Points ===\n");
    
    // App root component
    if let Some(app) = map.components.iter().find(|c| c.name == "App" || c.name == "app") {
        println!("App Component:");
        println!("  {} ({})", app.name, app.file.display());
    }
    
    // Main entry files
    let entry_files = vec!["main.ts", "main.js", "main.tsx", "main.jsx", "index.ts", "index.js"];
    println!("\nEntry Files:");
    for entry_name in &entry_files {
        let path = map.project.root.join("src").join(entry_name);
        if path.exists() {
            println!("  src/{}", entry_name);
        }
    }
    
    // Router
    let router_files = vec!["router/index.ts", "router/index.js", "routes.ts", "routes.js"];
    println!("\nRouter:");
    for router_name in &router_files {
        let path = map.project.root.join("src").join(router_name);
        if path.exists() {
            println!("  src/{}", router_name);
        }
    }
    
    // Store files
    if !map.stores.is_empty() {
        println!("\nStores:");
        for store in &map.stores {
            println!("  {} ({})", store.name, store.file.display());
        }
    }
    
    // Layout components
    let layout_components: Vec<_> = map.components.iter()
        .filter(|c| c.name.to_lowercase().contains("layout") || 
                    c.name.to_lowercase().contains("app") ||
                    c.file.to_string_lossy().contains("layout"))
        .collect();
    if !layout_components.is_empty() {
        println!("\nLayout Components:");
        for comp in &layout_components {
            println!("  {} ({})", comp.name, comp.file.display());
        }
    }
    
    // Route page components
    if !map.routes.is_empty() {
        println!("\nPage Components (from routes):");
        for route in &map.routes {
            println!("  {} → {}", route.path, route.component);
        }
    }
}

pub fn query_similar(map: &model::FrontendMap, name: &str, limit: usize) {
    let target = match map.components.iter().find(|c| c.name == name) {
        Some(c) => c,
        None => {
            eprintln!("Component '{}' not found", name);
            return;
        }
    };
    
    println!("=== Components similar to '{}' ===\n", name);
    
    let mut scores: Vec<(&model::Component, f64)> = Vec::new();
    
    for comp in &map.components {
        if comp.name == name {
            continue;
        }
        
        let mut score = 0.0;
        
        // Same directory
        if comp.file.parent() == target.file.parent() {
            score += 2.0;
        }
        
        // Similar name patterns
        let target_prefix = get_common_prefix(name);
        let comp_prefix = get_common_prefix(&comp.name);
        if !target_prefix.is_empty() && target_prefix == comp_prefix {
            score += 3.0;
        }
        
        // Same component type
        if std::mem::discriminant(&comp.kind) == std::mem::discriminant(&target.kind) {
            score += 1.0;
        }
        
        // Similar props count
        let prop_diff = (comp.props.len() as i32 - target.props.len() as i32).abs();
        if prop_diff <= 2 {
            score += 1.0;
        }
        
        // Used by similar components
        let target_users: HashSet<_> = target.used_by.iter().collect();
        let comp_users: HashSet<_> = comp.used_by.iter().collect();
        let common_users = target_users.intersection(&comp_users).count();
        if common_users > 0 {
            score += common_users as f64;
        }
        
        // Name contains target or vice versa
        if comp.name.contains(name) || name.contains(&comp.name) {
            score += 2.0;
        }
        
        if score > 0.0 {
            scores.push((comp, score));
        }
    }
    
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    for (i, (comp, score)) in scores.iter().enumerate() {
        if i >= limit {
            break;
        }
        println!("{} (score: {:.1}) - {}", comp.name, score, comp.file.display());
    }
    
    if scores.is_empty() {
        println!("No similar components found");
    }
}

fn get_common_prefix(name: &str) -> String {
    // Extract common prefix patterns like "User", "Agent", "List", etc.
    let common_prefixes = vec![
        "User", "Agent", "List", "Detail", "Form", "Modal", "Card", "Table",
        "Header", "Footer", "Sidebar", "Nav", "Tab", "Panel", "Drawer",
        "Button", "Input", "Select", "Checkbox", "Radio", "Switch",
        "Create", "Edit", "Delete", "Update", "View", "Show", "Display",
    ];
    
    for prefix in common_prefixes {
        if name.starts_with(prefix) {
            return prefix.to_string();
        }
    }
    
    String::new()
}

pub fn query_deps(map: &model::FrontendMap, name: &str, depth: usize) {
    // Check if it's a store
    if let Some(store) = map.stores.iter().find(|s| s.name == name) {
        println!("=== Dependencies for store '{}' ===\n", name);
        println!("Store: {} ({})", store.name, store.file.display());
        println!("Type: {}", store.kind);
        if !store.subscribers.is_empty() {
            println!("\nSubscribers:");
            for sub in &store.subscribers {
                println!("  - {}", sub);
            }
        }
        return;
    }
    
    // Check if it's a component
    if !map.components.iter().any(|c| c.name == name) {
        eprintln!("Component '{}' not found", name);
        return;
    }
    
    println!("=== Dependencies for '{}' (depth: {}) ===\n", name, depth);
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((name.to_string(), 0));
    visited.insert(name.to_string());
    
    while let Some((current_name, current_depth)) = queue.pop_front() {
        if current_depth > depth {
            continue;
        }
        
        let indent = "  ".repeat(current_depth);
        
        if let Some(comp) = map.components.iter().find(|c| c.name == current_name) {
            // Show component info
            if current_depth == 0 {
                println!("{}├── {} (root)", indent, current_name);
            }
            
            // Find components this one uses (by looking at imports in file)
            let used_components = find_used_components(map, comp);
            for used in &used_components {
                if !visited.contains(used) {
                    visited.insert(used.clone());
                    println!("{}├── {}", "  ".repeat(current_depth + 1), used);
                    if current_depth + 1 < depth {
                        queue.push_back((used.clone(), current_depth + 1));
                    }
                }
            }
            
            // Show stores used
            let used_stores = find_used_stores(map, comp);
            for store in &used_stores {
                println!("{}├── {} (store)", "  ".repeat(current_depth + 1), store);
            }
            
            // Show API calls
            for api in &map.api_calls {
                if api.component == current_name {
                    println!("{}├── {} {} (API)", "  ".repeat(current_depth + 1), api.method, api.endpoint);
                }
            }
        }
    }
}

pub fn find_used_components(map: &model::FrontendMap, comp: &model::Component) -> Vec<String> {
    let mut used = Vec::new();
    
    // Read the component file
    if let Ok(content) = std::fs::read_to_string(&comp.file) {
        for other in &map.components {
            if other.name == comp.name {
                continue;
            }
            
            // Check for import or JSX usage
            let import_pattern = format!("import.*{}", other.name);
            let jsx_pattern = format!("<{}", other.name);
            
            if content.contains(&other.name) && 
               (content.contains(&format!("import")) || content.contains(&jsx_pattern)) {
                used.push(other.name.clone());
            }
        }
    }
    
    used
}

pub fn find_used_stores(map: &model::FrontendMap, comp: &model::Component) -> Vec<String> {
    let mut used = Vec::new();
    
    if let Ok(content) = std::fs::read_to_string(&comp.file) {
        for store in &map.stores {
            if content.contains(&store.name) {
                used.push(store.name.clone());
            }
        }
    }
    
    used
}

pub fn query_impact(map: &model::FrontendMap, name: &str, depth: usize) {
    println!("=== Impact Analysis for '{}' (depth: {}) ===\n", name, depth);
    
    // Check if it's a component
    let is_component = map.components.iter().any(|c| c.name == name);
    // Check if it's a store
    let is_store = map.stores.iter().any(|s| s.name == name);
    
    if !is_component && !is_store {
        eprintln!("'{}' not found as component or store", name);
        return;
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((name.to_string(), 0));
    visited.insert(name.to_string());
    
    let mut impacted = Vec::new();
    
    while let Some((current_name, current_depth)) = queue.pop_front() {
        if current_depth > depth {
            continue;
        }
        
        if current_depth > 0 {
            impacted.push((current_name.clone(), current_depth));
        }
        
        // Find components that use this one
        for comp in &map.components {
            if comp.name == current_name {
                continue;
            }
            
            let uses_current = if is_store {
                // Check if component uses this store
                find_used_stores(map, comp).contains(&current_name)
            } else {
                // Check if component uses this component
                find_used_components(map, comp).contains(&current_name)
            };
            
            if uses_current && !visited.contains(&comp.name) {
                visited.insert(comp.name.clone());
                queue.push_back((comp.name.clone(), current_depth + 1));
            }
        }
        
        // If it's a store, also check subscribers
        if is_store {
            if let Some(store) = map.stores.iter().find(|s| s.name == current_name) {
                for sub in &store.subscribers {
                    if !visited.contains(sub) {
                        visited.insert(sub.clone());
                        queue.push_back((sub.clone(), current_depth + 1));
                    }
                }
            }
        }
    }
    
    if impacted.is_empty() {
        println!("No components would be directly affected");
    } else {
        println!("Components affected by changes to '{}':\n", name);
        for (comp_name, depth) in &impacted {
            println!("  {} (depth: {})", comp_name, depth);
        }
        println!("\nTotal: {} components", impacted.len());
    }
    
    // Also show API calls that might be affected
    let affected_apis: Vec<_> = map.api_calls.iter()
        .filter(|a| impacted.iter().any(|(name, _)| *name == a.component))
        .collect();
    
    if !affected_apis.is_empty() {
        println!("\nAPI calls that might be affected:");
        for api in &affected_apis {
            println!("  {} {} ({})", api.method, api.endpoint, api.component);
        }
    }
}

pub fn query_flow(map: &model::FrontendMap, name: &str, depth: usize) {
    let store = match map.stores.iter().find(|s| s.name == name) {
        Some(s) => s,
        None => {
            eprintln!("Store '{}' not found", name);
            return;
        }
    };
    
    println!("=== Data Flow for store '{}' (depth: {}) ===\n", name, depth);
    println!("Store: {} ({})", store.name, store.file.display());
    println!("Type: {}", store.kind);
    
    // Show direct subscribers
    println!("\nDirect Subscribers:");
    for sub in &store.subscribers {
        println!("  ├── {}", sub);
    }
    
    // Show flow through components
    println!("\nData Flow Graph:");
    println!("  {}", name);
    
    let mut visited = HashSet::new();
    visited.insert(name.to_string());
    
    for sub in &store.subscribers {
        if visited.contains(sub) {
            continue;
        }
        visited.insert(sub.clone());
        
        println!("  ├── {}", sub);
        
        // Find what this subscriber component uses
        if let Some(comp) = map.components.iter().find(|c| c.name == *sub) {
            let used_components = find_used_components(map, comp);
            for used in &used_components {
                if !visited.contains(used) {
                    println!("  │   ├── {}", used);
                }
            }
            
            // Find what APIs this component calls
            let apis: Vec<_> = map.api_calls.iter()
                .filter(|a| a.component == *sub)
                .collect();
            for api in &apis {
                println!("  │   ├── {} {} (API)", api.method, api.endpoint);
            }
        }
    }
    
    // Show related stores
    let related_stores: Vec<_> = map.stores.iter()
        .filter(|s| s.name != name && s.subscribers.iter().any(|sub| store.subscribers.contains(sub)))
        .collect();
    
    if !related_stores.is_empty() {
        println!("\nRelated Stores (share subscribers):");
        for related in &related_stores {
            let common: Vec<_> = related.subscribers.iter()
                .filter(|sub| store.subscribers.contains(sub))
                .collect();
            let common_str: Vec<&str> = common.iter().map(|s| s.as_str()).collect();
            println!("  {} (common: {})", related.name, common_str.join(", "));
        }
    }
}

pub fn query_scope(map: &model::FrontendMap, target: &str) {
    println!("=== Scope: {} ===\n", target);
    
    // Check if it's a file path
    let path = std::path::Path::new(target);
    if path.exists() {
        // Show components in this file
        let components: Vec<_> = map.components.iter()
            .filter(|c| c.file == path)
            .collect();
        if !components.is_empty() {
            println!("Components in {}:", target);
            for comp in &components {
                println!("  - {} (line {})", comp.name, comp.line);
            }
        }
        
        // Show API calls in this file
        let apis: Vec<_> = map.api_calls.iter()
            .filter(|a| a.file == path)
            .collect();
        if !apis.is_empty() {
            println!("\nAPI calls in {}:", target);
            for api in &apis {
                println!("  - {} {} (line {})", api.method, api.endpoint, api.line);
            }
        }
        return;
    }
    
    // Check if it's a component name
    if let Some(comp) = map.components.iter().find(|c| c.name == target) {
        println!("Component: {}", comp.name);
        println!("File: {}", comp.file.display());
        println!("Line: {}", comp.line);
        
        // Show what this component uses
        let used = find_used_components(map, comp);
        if !used.is_empty() {
            println!("\nUses components:");
            for u in &used {
                println!("  - {}", u);
            }
        }
        
        let stores = find_used_stores(map, comp);
        if !stores.is_empty() {
            println!("\nUses stores:");
            for s in &stores {
                println!("  - {}", s);
            }
        }
        
        let apis: Vec<_> = map.api_calls.iter()
            .filter(|a| a.component == target)
            .collect();
        if !apis.is_empty() {
            println!("\nAPI calls:");
            for api in &apis {
                println!("  - {} {}", api.method, api.endpoint);
            }
        }
        return;
    }
    
    // Check if it's a directory
    let dir_path = map.project.root.join(target);
    if dir_path.exists() && dir_path.is_dir() {
        let components: Vec<_> = map.components.iter()
            .filter(|c| c.file.starts_with(&dir_path))
            .collect();
        if !components.is_empty() {
            println!("Components in {}/:", target);
            for comp in &components {
                println!("  - {} ({})", comp.name, comp.file.display());
            }
        }
        return;
    }
    
    eprintln!("'{}' not found as file, component, or directory", target);
}

pub fn query_path(map: &model::FrontendMap, from: &str, to: &str) {
    println!("=== Path from '{}' to '{}' ===\n", from, to);
    
    // BFS to find shortest path
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<String, String> = HashMap::new();
    
    queue.push_back(from.to_string());
    visited.insert(from.to_string());
    
    let mut found = false;
    
    while let Some(current) = queue.pop_front() {
        if current == to {
            found = true;
            break;
        }
        
        // Get neighbors (components this one uses)
        let neighbors = if let Some(comp) = map.components.iter().find(|c| c.name == current) {
            find_used_components(map, comp)
        } else {
            Vec::new()
        };
        
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                parent.insert(neighbor.clone(), current.clone());
                queue.push_back(neighbor);
            }
        }
        
        // Also check store subscribers
        if let Some(store) = map.stores.iter().find(|s| s.name == current) {
            for sub in &store.subscribers {
                if !visited.contains(sub) {
                    visited.insert(sub.clone());
                    parent.insert(sub.clone(), current.clone());
                    queue.push_back(sub.clone());
                }
            }
        }
    }
    
    if !found {
        println!("No path found from '{}' to '{}'", from, to);
        return;
    }
    
    // Reconstruct path
    let mut path = Vec::new();
    let mut current = to.to_string();
    while current != from {
        path.push(current.clone());
        if let Some(p) = parent.get(&current) {
            current = p.clone();
        } else {
            break;
        }
    }
    path.push(from.to_string());
    path.reverse();
    
    println!("Path ({} steps):", path.len() - 1);
    for (i, node) in path.iter().enumerate() {
        let indent = "  ".repeat(i);
        if i == 0 {
            println!("{}├── {} (start)", indent, node);
        } else if i == path.len() - 1 {
            println!("{}├── {} (end)", indent, node);
        } else {
            println!("{}├── {}", indent, node);
        }
    }
}

pub fn query_export(map: &model::FrontendMap, format: &str, output: Option<&str>) {
    let content = match format {
        "json" => serde_json::to_string_pretty(map).unwrap_or_default(),
        "dot" => export_dot(map),
        "mermaid" => export_mermaid(map),
        _ => {
            eprintln!("Unknown format: {}. Supported: json, dot, mermaid", format);
            return;
        }
    };
    
    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &content) {
                eprintln!("Failed to write to {}: {}", path, e);
            } else {
                println!("✓ Exported to {}", path);
            }
        }
        None => println!("{}", content),
    }
}

fn export_dot(map: &model::FrontendMap) -> String {
    let mut dot = String::from("digraph frontendmap {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");
    
    // Add components
    for comp in &map.components {
        dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", comp.name, comp.name));
    }
    
    // Add stores
    for store in &map.stores {
        dot.push_str(&format!("  \"{}\" [label=\"{}\" shape=ellipse];\n", store.name, store.name));
    }
    
    dot.push_str("\n");
    
    // Add edges (component usage)
    for comp in &map.components {
        let used = find_used_components(map, comp);
        for u in &used {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", comp.name, u));
        }
        
        let stores = find_used_stores(map, comp);
        for s in &stores {
            dot.push_str(&format!("  \"{}\" -> \"{}\" [style=dashed];\n", comp.name, s));
        }
    }
    
    dot.push_str("}\n");
    dot
}

fn export_mermaid(map: &model::FrontendMap) -> String {
    let mut mermaid = String::from("graph LR\n");
    
    // Add components
    for comp in &map.components {
        mermaid.push_str(&format!("  {}[{}]\n", comp.name, comp.name));
    }
    
    // Add stores
    for store in &map.stores {
        mermaid.push_str(&format!("  {}({})\n", store.name, store.name));
    }
    
    mermaid.push_str("\n");
    
    // Add edges
    for comp in &map.components {
        let used = find_used_components(map, comp);
        for u in &used {
            mermaid.push_str(&format!("  {} --> {}\n", comp.name, u));
        }
        
        let stores = find_used_stores(map, comp);
        for s in &stores {
            mermaid.push_str(&format!("  {} -.-> {}\n", comp.name, s));
        }
    }
    
    mermaid
}
