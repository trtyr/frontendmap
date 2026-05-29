use crate::model;
use std::collections::HashMap;
use super::query::{find_used_components, find_used_stores};

pub fn nav_guide(map: &model::FrontendMap) {
    println!("=== Navigation Guide ===\n");
    
    // Entry points
    println!("1. Entry Points:");
    if let Some(app) = map.components.iter().find(|c| c.name == "App") {
        println!("   - App: {}", app.file.display());
    }
    for route in map.routes.iter().take(5) {
        println!("   - Route {}: {} → {}", route.path, route.component, route.file.display());
    }
    
    // Core stores
    if !map.stores.is_empty() {
        println!("\n2. Core Stores:");
        for store in &map.stores {
            println!("   - {} ({} subscribers)", store.name, store.subscribers.len());
        }
    }
    
    // Key components (most used)
    let mut usage_count: HashMap<String, usize> = HashMap::new();
    for comp in &map.components {
        for user in &comp.used_by {
            *usage_count.entry(user.to_string_lossy().to_string()).or_insert(0) += 1;
        }
    }
    let mut usage_vec: Vec<_> = usage_count.iter().collect();
    usage_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    if !usage_vec.is_empty() {
        println!("\n3. Key Components (most referenced):");
        for (name, count) in usage_vec.iter().take(5) {
            println!("   - {} ({} references)", name, count);
        }
    }
    
    // API endpoints
    if !map.api_calls.is_empty() {
        println!("\n4. API Endpoints:");
        for api in map.api_calls.iter().take(5) {
            println!("   - {} {} ({})", api.method, api.endpoint, api.component);
        }
        if map.api_calls.len() > 5 {
            println!("   ... and {} more", map.api_calls.len() - 5);
        }
    }
}

pub fn nav_quality(map: &model::FrontendMap) {
    println!("=== Quality Score ===\n");
    
    let mut score = 100.0;
    let mut issues = Vec::new();
    
    // Check for components without props (might be too simple)
    let no_props = map.components.iter().filter(|c| c.props.is_empty()).count();
    if no_props > map.components.len() / 2 {
        score -= 10.0;
        issues.push(format!("{} components have no props ({}%)", no_props, no_props * 100 / map.components.len()));
    }
    
    // Check for unused components
    let unused = map.components.iter().filter(|c| c.used_by.is_empty()).count();
    if unused > 0 {
        score -= (unused as f64 * 2.0).min(20.0);
        issues.push(format!("{} components appear unused", unused));
    }
    
    // Check for stores without subscribers
    let empty_stores = map.stores.iter().filter(|s| s.subscribers.is_empty()).count();
    if empty_stores > 0 {
        score -= (empty_stores as f64 * 5.0).min(15.0);
        issues.push(format!("{} stores have no subscribers", empty_stores));
    }
    
    // Check for API calls without error handling (basic check)
    // This would require reading files, skip for now
    
    // Check for consistent naming
    let mut naming_issues = 0;
    for comp in &map.components {
        if comp.name.contains('_') && !comp.name.starts_with('_') {
            naming_issues += 1;
        }
    }
    if naming_issues > 0 {
        score -= (naming_issues as f64 * 0.5).min(5.0);
        issues.push(format!("{} components use snake_case (should be PascalCase)", naming_issues));
    }
    
    score = score.max(0.0);
    
    println!("Score: {:.0}/100", score);
    if !issues.is_empty() {
        println!("\nIssues:");
        for issue in &issues {
            println!("  - {}", issue);
        }
    } else {
        println!("\nNo issues found!");
    }
}

pub fn nav_health(map: &model::FrontendMap) {
    println!("=== Health Check ===\n");
    
    // Check for god components (too many dependencies)
    let mut god_components = Vec::new();
    for comp in &map.components {
        let used = find_used_components(map, comp);
        let stores = find_used_stores(map, comp);
        let apis: Vec<_> = map.api_calls.iter().filter(|a| a.component == comp.name).collect();
        let total_deps = used.len() + stores.len() + apis.len();
        
        if total_deps > 10 {
            god_components.push((comp.name.clone(), total_deps));
        }
    }
    
    if !god_components.is_empty() {
        println!("⚠ God Components (too many dependencies):");
        for (name, deps) in &god_components {
            println!("  - {} ({} dependencies)", name, deps);
        }
    }
    
    // Check for circular dependencies (simplified)
    let mut cycles = Vec::new();
    for comp in &map.components {
        let used = find_used_components(map, comp);
        for u in &used {
            if let Some(other) = map.components.iter().find(|c| c.name == *u) {
                let other_used = find_used_components(map, other);
                if other_used.contains(&comp.name) {
                    cycles.push((comp.name.clone(), u.clone()));
                }
            }
        }
    }
    
    if !cycles.is_empty() {
        println!("\n⚠ Circular Dependencies:");
        for (a, b) in &cycles {
            println!("  - {} ↔ {}", a, b);
        }
    }
    
    // Check for unused components
    let unused: Vec<_> = map.components.iter()
        .filter(|c| c.used_by.is_empty() && !c.name.starts_with('_'))
        .collect();
    
    if !unused.is_empty() {
        println!("\n⚠ Potentially Unused Components:");
        for comp in unused.iter().take(10) {
            println!("  - {} ({})", comp.name, comp.file.display());
        }
        if unused.len() > 10 {
            println!("  ... and {} more", unused.len() - 10);
        }
    }
    
    // Check for stores without subscribers
    let empty_stores: Vec<_> = map.stores.iter()
        .filter(|s| s.subscribers.is_empty())
        .collect();
    
    if !empty_stores.is_empty() {
        println!("\n⚠ Empty Stores (no subscribers):");
        for store in &empty_stores {
            println!("  - {} ({})", store.name, store.file.display());
        }
    }
    
    if god_components.is_empty() && cycles.is_empty() && unused.is_empty() && empty_stores.is_empty() {
        println!("✓ No health issues found!");
    }
}

pub fn nav_report(map: &model::FrontendMap, output: Option<&str>) {
    let report = generate_report(map);
    
    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &report) {
                eprintln!("Failed to write report to {}: {}", path, e);
            } else {
                println!("✓ Report saved to {}", path);
            }
        }
        None => println!("{}", report),
    }
}

fn generate_report(map: &model::FrontendMap) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("# Frontend Project Report: {}\n\n", map.project.name));
    report.push_str(&format!("**Framework:** {}\n", map.project.framework.as_str()));
    report.push_str(&format!("**Components:** {}\n", map.project.component_count));
    report.push_str(&format!("**Files:** {}\n", map.project.file_count));
    report.push_str(&format!("**Routes:** {}\n", map.routes.len()));
    report.push_str(&format!("**API Calls:** {}\n", map.api_calls.len()));
    report.push_str(&format!("**Stores:** {}\n\n", map.stores.len()));
    
    // Entry Points
    report.push_str("## Entry Points\n\n");
    if let Some(app) = map.components.iter().find(|c| c.name == "App") {
        report.push_str(&format!("- App: {}\n", app.file.display()));
    }
    for route in map.routes.iter().take(5) {
        report.push_str(&format!("- Route {}: {} → {}\n", route.path, route.component, route.file.display()));
    }
    
    // Stores
    if !map.stores.is_empty() {
        report.push_str("\n## State Management\n\n");
        for store in &map.stores {
            report.push_str(&format!("- **{}** ({}) - {} subscribers\n", 
                store.name, store.kind, store.subscribers.len()));
        }
    }
    
    // API Endpoints
    if !map.api_calls.is_empty() {
        report.push_str("\n## API Endpoints\n\n");
        for api in &map.api_calls {
            report.push_str(&format!("- {} {} ({})\n", api.method, api.endpoint, api.component));
        }
    }
    
    report
}

pub fn nav_map(map: &model::FrontendMap, full: bool) {
    println!("=== Project Map ===\n");
    println!("Project: {}", map.project.name);
    println!("Framework: {}", map.project.framework.as_str());
    println!("Components: {}", map.project.component_count);
    println!("Files: {}", map.project.file_count);
    println!("Routes: {}", map.routes.len());
    println!("API Calls: {}", map.api_calls.len());
    println!("Stores: {}", map.stores.len());
    
    if full {
        println!("\n--- Components ---");
        for comp in &map.components {
            println!("  {} - {}", comp.name, comp.file.display());
        }
        
        println!("\n--- Routes ---");
        for route in &map.routes {
            println!("  {} → {}", route.path, route.component);
        }
        
        println!("\n--- Stores ---");
        for store in &map.stores {
            println!("  {} ({} subscribers)", store.name, store.subscribers.len());
        }
        
        println!("\n--- API Calls ---");
        for api in &map.api_calls {
            println!("  {} {} → {}", api.method, api.endpoint, api.component);
        }
    }
}
