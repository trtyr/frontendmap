use crate::model;
use std::collections::{HashMap, HashSet, VecDeque};
use super::query::{find_used_components, find_used_stores};

pub fn analyze_deps(map: &model::FrontendMap, from: Option<&str>) {
    println!("=== Dependency Matrix ===\n");
    
    // Build dependency matrix
    let mut matrix: HashMap<String, Vec<String>> = HashMap::new();
    
    for comp in &map.components {
        let used = find_used_components(map, comp);
        let stores = find_used_stores(map, comp);
        let mut deps = used;
        deps.extend(stores);
        matrix.insert(comp.name.clone(), deps);
    }
    
    // Filter if from is specified
    if let Some(from_name) = from {
        if let Some(deps) = matrix.get(from_name) {
            println!("{} depends on:", from_name);
            for dep in deps {
                println!("  - {}", dep);
            }
        } else {
            eprintln!("Component '{}' not found", from_name);
        }
        return;
    }
    
    // Show full matrix
    for (name, deps) in &matrix {
        if !deps.is_empty() {
            println!("{} → {}", name, deps.join(", "));
        }
    }
}

pub fn analyze_fanout(map: &model::FrontendMap, limit: usize) {
    println!("=== Fan-in/Fan-out Analysis ===\n");
    
    let mut fan_in: HashMap<String, usize> = HashMap::new();
    let mut fan_out: HashMap<String, usize> = HashMap::new();
    
    // Calculate fan-out (dependencies)
    for comp in &map.components {
        let used = find_used_components(map, comp);
        let stores = find_used_stores(map, comp);
        fan_out.insert(comp.name.clone(), used.len() + stores.len());
    }
    
    // Calculate fan-in (dependents)
    for comp in &map.components {
        for user in &comp.used_by {
            *fan_in.entry(user.to_string_lossy().to_string()).or_insert(0) += 1;
        }
    }
    
    // Sort by fan-out
    let mut fan_out_vec: Vec<_> = fan_out.iter().collect();
    fan_out_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("Top Fan-out (depends on many):");
    for (name, count) in fan_out_vec.iter().take(limit) {
        println!("  - {} ({} dependencies)", name, count);
    }
    
    // Sort by fan-in
    let mut fan_in_vec: Vec<_> = fan_in.iter().collect();
    fan_in_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("\nTop Fan-in (depended on by many):");
    for (name, count) in fan_in_vec.iter().take(limit) {
        println!("  - {} ({} dependents)", name, count);
    }
}

pub fn analyze_tests(map: &model::FrontendMap, name: Option<&str>) {
    println!("=== Test Impact Analysis ===\n");
    
    match name {
        Some(component_name) => {
            // Find components that would be affected
            let mut impacted = Vec::new();
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            
            queue.push_back(component_name.to_string());
            visited.insert(component_name.to_string());
            
            while let Some(current) = queue.pop_front() {
                impacted.push(current.clone());
                
                // Find components that use this one
                for comp in &map.components {
                    if comp.name == current {
                        continue;
                    }
                    
                    let used = find_used_components(map, comp);
                    if used.contains(&current) && !visited.contains(&comp.name) {
                        visited.insert(comp.name.clone());
                        queue.push_back(comp.name.clone());
                    }
                }
            }
            
            println!("Components affected by changes to '{}':", component_name);
            for comp in &impacted {
                println!("  - {}", comp);
            }
            println!("\nTotal: {} components", impacted.len());
            
            // Find related test files
            println!("\nRelated test files:");
            for comp in &impacted {
                let test_patterns = vec![
                    format!("{}.test.", comp),
                    format!("{}.spec.", comp),
                    format!("{}Test.", comp),
                    format!("{}Spec.", comp),
                ];
                
                for pattern in &test_patterns {
                    // This would require file system search, simplified for now
                    println!("  - Look for: {}", pattern);
                }
            }
        }
        None => {
            // Show all components and their test coverage hints
            println!("Components and test file hints:");
            for comp in &map.components {
                let has_test = map.components.iter().any(|c| 
                    c.name.contains(&format!("{}Test", comp.name)) ||
                    c.name.contains(&format!("{}Spec", comp.name)) ||
                    c.name.contains(&format!("{}.test", comp.name)) ||
                    c.name.contains(&format!("{}.spec", comp.name))
                );
                
                if has_test {
                    println!("  ✓ {} - has test file", comp.name);
                } else {
                    println!("  ✗ {} - no test file found", comp.name);
                }
            }
        }
    }
}
