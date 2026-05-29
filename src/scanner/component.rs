use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::model::{Component, ComponentKind, Prop};

pub fn scan_components(root: &Path) -> Result<Vec<Component>> {
    let mut components = Vec::new();
    let mut file_components: HashMap<PathBuf, Vec<String>> = HashMap::new();
    
    // Walk through all JS/TS/JSX/TSX/Vue/Svelte files
    let walker = ignore::WalkBuilder::new(root)
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
        if !is_component_file(path) {
            continue;
        }
        
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let file_comps = extract_components(path, &content);
        for comp in file_comps {
            file_components.entry(path.to_path_buf()).or_default().push(comp.name.clone());
            components.push(comp);
        }
    }
    
    // Build reference relationships
    build_references(&mut components, root);
    
    Ok(components)
}

fn is_component_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jsx" | "tsx" | "vue" | "svelte" | "astro")
    } else {
        // Also check .js/.ts files that might contain components
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "js" | "ts") {
                // Check if filename starts with uppercase (component convention)
                if let Some(name) = path.file_stem() {
                    let name = name.to_string_lossy();
                    return name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                }
            }
        }
        false
    }
}

fn extract_components(file_path: &Path, content: &str) -> Vec<Component> {
    let mut components = Vec::new();
    
    // Check file extension to determine extraction method
    if let Some(ext) = file_path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "vue" => return extract_vue_components(file_path, content),
            "svelte" => return extract_svelte_components(file_path, content),
            "astro" => return extract_astro_components(file_path, content),
            _ => {} // Continue with JS/TS extraction
        }
    }
    
    // JS/TS component extraction patterns
    let patterns = vec![
        // Pattern 1: export default function ComponentName
        (r"export\s+default\s+(?:async\s+)?function\s+(\w+)", ComponentKind::Function),
        // Pattern 2: export const ComponentName = () =>
        (r"export\s+const\s+(\w+)\s*=\s*(?:\([^)]*\)|[a-zA-Z_]\w*)\s*=>", ComponentKind::Arrow),
        // Pattern 3: export function ComponentName
        (r"export\s+(?:async\s+)?function\s+(\w+)", ComponentKind::Function),
        // Pattern 4: export default ComponentName (for class components)
        (r"export\s+default\s+class\s+(\w+)", ComponentKind::Class),
        // Pattern 5: export class ComponentName
        (r"export\s+class\s+(\w+)", ComponentKind::Class),
        // Pattern 6: const ComponentName = () => ... export default ComponentName
        (r"const\s+(\w+)\s*=\s*(?:\([^)]*\)|[a-zA-Z_]\w*)\s*=>", ComponentKind::Arrow),
        // Pattern 7: function ComponentName() { ... } export default ComponentName
        (r"(?:async\s+)?function\s+(\w+)\s*\(", ComponentKind::Function),
    ];
    
    let lines: Vec<&str> = content.lines().collect();
    
    for (line_num, line) in lines.iter().enumerate() {
        let line = line.trim();
        
        for (pattern, kind) in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    let name = caps[1].to_string();
                    if is_component_name(&name) {
                        let props = extract_js_props(content, &name);
                        components.push(Component {
                            name,
                            file: file_path.to_path_buf(),
                            kind: kind.clone(),
                            props,
                            used_by: Vec::new(),
                            uses: Vec::new(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
    
    components
}

fn extract_vue_components(file_path: &Path, content: &str) -> Vec<Component> {
    let mut components = Vec::new();
    
    // Vue component name is the filename (without extension)
    let name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Skip non-component files
    if name.starts_with('_') || name == "index" {
        // For index files, use parent directory name
        if name == "index" {
            if let Some(parent) = file_path.parent() {
                let parent_name = parent.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let props = extract_vue_props(content);
                let script_line = find_script_line(content);
                
                components.push(Component {
                    name: parent_name,
                    file: file_path.to_path_buf(),
                    kind: ComponentKind::Function,
                    props,
                    used_by: Vec::new(),
                    uses: Vec::new(),
                    line: script_line,
                });
            }
        }
        return components;
    }
    
    let props = extract_vue_props(content);
    let script_line = find_script_line(content);
    
    components.push(Component {
        name,
        file: file_path.to_path_buf(),
        kind: ComponentKind::Function,
        props,
        used_by: Vec::new(),
        uses: Vec::new(),
        line: script_line,
    });
    
    components
}

fn extract_svelte_components(file_path: &Path, content: &str) -> Vec<Component> {
    let mut components = Vec::new();
    
    // Svelte component name is the filename
    let name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Extract props from export let
    let props = extract_svelte_props(content);
    let script_line = find_script_line(content);
    
    components.push(Component {
        name,
        file: file_path.to_path_buf(),
        kind: ComponentKind::Function,
        props,
        used_by: Vec::new(),
        uses: Vec::new(),
        line: script_line,
    });
    
    components
}

fn extract_astro_components(file_path: &Path, content: &str) -> Vec<Component> {
    let mut components = Vec::new();
    
    // Astro component name is the filename
    let name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Extract props from Astro interface
    let props = extract_astro_props(content);
    let script_line = find_script_line(content);
    
    components.push(Component {
        name,
        file: file_path.to_path_buf(),
        kind: ComponentKind::Function,
        props,
        used_by: Vec::new(),
        uses: Vec::new(),
        line: script_line,
    });
    
    components
}

fn find_script_line(content: &str) -> usize {
    let script_re = Regex::new(r"<script").expect("invalid regex pattern");
    content.lines().enumerate()
        .find(|(_, l)| script_re.is_match(l))
        .map(|(i, _)| i + 1)
        .unwrap_or(1)
}

fn is_component_name(name: &str) -> bool {
    // Component names should start with uppercase
    // Skip common non-component exports
    let skip_names = ["default", "props", "emits", "setup", "data", "methods", "computed", "watch"];
    !skip_names.contains(&name) && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

fn extract_js_props(content: &str, component_name: &str) -> Vec<Prop> {
    let mut props = Vec::new();
    
    // Pattern 1: function Component({ prop1, prop2 })
    let destructure_re = Regex::new(&format!(r"{}\s*\(\s*\{{([^}}]*)\}}", regex::escape(component_name))).expect("invalid regex pattern");
    
    // Pattern 2: function Component(props: { prop1: type1, prop2: type2 })
    let type_re = Regex::new(&format!(r"{}\s*\(\s*(?:props|{{[^}}]*}})\s*:\s*\{{([^}}]*)\}}", regex::escape(component_name))).expect("invalid regex pattern");
    
    // Pattern 3: interface Props { ... } or type Props = { ... }
    let interface_re = Regex::new(r"(?:interface|type)\s+(?:Props|IProps)\s*(?:=\s*)?\{([^}]+)\}").expect("invalid regex pattern");
    
    if let Some(caps) = destructure_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split(',') {
            let prop = prop.trim();
            if !prop.is_empty() {
                let parts: Vec<&str> = prop.split(':').collect();
                let name = parts[0].trim().to_string();
                let type_annotation = if parts.len() > 1 {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                };
                props.push(Prop {
                    name,
                    type_annotation,
                    required: true,
                });
            }
        }
    } else if let Some(caps) = type_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split(',') {
            let prop = prop.trim();
            if !prop.is_empty() {
                let parts: Vec<&str> = prop.split(':').collect();
                let name = parts[0].trim().trim_matches('?').to_string();
                let type_annotation = if parts.len() > 1 {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                };
                let required = !prop.contains('?');
                props.push(Prop {
                    name,
                    type_annotation,
                    required,
                });
            }
        }
    } else if let Some(caps) = interface_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split('\n') {
            let prop = prop.trim().trim_end_matches(';');
            if !prop.is_empty() && !prop.starts_with("//") {
                let parts: Vec<&str> = prop.split(':').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().trim_matches('?').to_string();
                    let type_annotation = Some(parts[1].trim().to_string());
                    let required = !prop.contains('?');
                    props.push(Prop {
                        name,
                        type_annotation,
                        required,
                    });
                }
            }
        }
    }
    
    props
}

fn extract_vue_props(content: &str) -> Vec<Prop> {
    let mut props = Vec::new();
    
    // Pattern 1: defineProps<{ prop1: type1, prop2: type2 }>()
    let type_props_re = Regex::new(r"defineProps\s*<\s*\{([^}]+)\}\s*>").expect("invalid regex pattern");
    // Pattern 2: defineProps({ prop1: type, prop2: type })
    let obj_props_re = Regex::new(r"defineProps\s*\(\s*\{([^}]+)\}\s*\)").expect("invalid regex pattern");
    // Pattern 3: props: { prop1: type, prop2: type } (Options API)
    let options_re = Regex::new(r"props\s*:\s*\{([^}]+)\}").expect("invalid regex pattern");
    
    if let Some(caps) = type_props_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split(',') {
            let prop = prop.trim();
            if !prop.is_empty() {
                let parts: Vec<&str> = prop.split(':').collect();
                let name = parts[0].trim().trim_matches('?').to_string();
                let type_annotation = if parts.len() > 1 {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                };
                let required = !prop.contains('?');
                props.push(Prop {
                    name,
                    type_annotation,
                    required,
                });
            }
        }
    } else if let Some(caps) = obj_props_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split(',') {
            let prop = prop.trim();
            if !prop.is_empty() {
                let parts: Vec<&str> = prop.split(':').collect();
                let name = parts[0].trim().to_string();
                let type_annotation = if parts.len() > 1 {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                };
                props.push(Prop {
                    name,
                    type_annotation,
                    required: true,
                });
            }
        }
    } else if let Some(caps) = options_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split(',') {
            let prop = prop.trim();
            if !prop.is_empty() {
                let parts: Vec<&str> = prop.split(':').collect();
                let name = parts[0].trim().to_string();
                let type_annotation = if parts.len() > 1 {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                };
                props.push(Prop {
                    name,
                    type_annotation,
                    required: true,
                });
            }
        }
    }
    
    props
}

fn extract_svelte_props(content: &str) -> Vec<Prop> {
    let mut props = Vec::new();
    
    // Svelte props: export let prop1, export let prop2: type
    let prop_re = Regex::new(r"export\s+let\s+(\w+)(?:\s*:\s*(\w+))?").expect("invalid regex pattern");
    
    for caps in prop_re.captures_iter(content) {
        let name = caps[1].to_string();
        let type_annotation = caps.get(2).map(|m| m.as_str().to_string());
        props.push(Prop {
            name,
            type_annotation,
            required: true,
        });
    }
    
    props
}

fn extract_astro_props(content: &str) -> Vec<Prop> {
    let mut props = Vec::new();
    
    // Astro props: interface Props { prop1: type1, prop2: type2 }
    let interface_re = Regex::new(r"interface\s+Props\s*\{([^}]+)\}").expect("invalid regex pattern");
    
    if let Some(caps) = interface_re.captures(content) {
        let props_str = &caps[1];
        for prop in props_str.split('\n') {
            let prop = prop.trim().trim_end_matches(';');
            if !prop.is_empty() && !prop.starts_with("//") {
                let parts: Vec<&str> = prop.split(':').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().trim_matches('?').to_string();
                    let type_annotation = Some(parts[1].trim().to_string());
                    let required = !prop.contains('?');
                    props.push(Prop {
                        name,
                        type_annotation,
                        required,
                    });
                }
            }
        }
    }
    
    props
}

fn build_references(components: &mut Vec<Component>, root: &Path) {
    let component_names: Vec<String> = components.iter().map(|c| c.name.clone()).collect();
    
    // Walk through all files to find references
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    let mut references: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
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
        
        // Check for component usage in JSX/Vue/Svelte
        for comp_name in &component_names {
            // Pattern 1: <ComponentName or <ComponentName>
            let jsx_pattern = format!(r"<{}", regex::escape(comp_name));
            let jsx_re = Regex::new(&jsx_pattern).expect("invalid regex pattern");
            
            // Pattern 2: import { ComponentName } from
            let import_pattern = format!(r"import\s+.*{}\s+.*from", regex::escape(comp_name));
            let import_re = Regex::new(&import_pattern).expect("invalid regex pattern");
            
            // Pattern 3: import ComponentName from
            let default_import_pattern = format!(r"import\s+{}\s+from", regex::escape(comp_name));
            let default_import_re = Regex::new(&default_import_pattern).expect("invalid regex pattern");
            
            if jsx_re.is_match(&content) || import_re.is_match(&content) || default_import_re.is_match(&content) {
                references.entry(comp_name.clone()).or_default().push(path.to_path_buf());
            }
        }
    }
    
    // Update components with references
    for comp in components.iter_mut() {
        if let Some(refs) = references.get(&comp.name) {
            comp.used_by = refs.clone();
        }
    }
}
