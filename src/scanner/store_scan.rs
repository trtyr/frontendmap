use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use crate::model::{Store, StoreKind};

fn scan_store_by_pattern<F>(
    root: &Path,
    stores: &mut Vec<Store>,
    kind: StoreKind,
    patterns: &[&str],
    extract_name: F,
    precondition: Option<&str>,
    skip_subscribers: bool,
) -> Result<()>
where
    F: Fn(&regex::Captures) -> String,
{
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .add_custom_ignore_filename(".gitignore")
        .filter_entry(|e| {
            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                !matches!(name.as_ref(), "node_modules" | "dist" | "build" | ".next" | ".nuxt" | ".svelte-kit" | ".git" | ".svn" | "vendor" | "coverage" | "__pycache__" | ".cache")
            } else {
                true
            }
        })
        .build();

    let regexes: Vec<Regex> = patterns.iter()
        .map(|p| Regex::new(p).expect("invalid regex"))
        .collect();

    let precond_re = precondition
        .map(|p| Regex::new(p).expect("invalid precondition regex"));

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        // Skip test files
        if is_test_file(path) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(ref pre) = precond_re {
            if !pre.is_match(&content) {
                continue;
            }
        }

        for re in &regexes {
            for caps in re.captures_iter(&content) {
                let name = extract_name(&caps);
                let subscribers = if skip_subscribers { Vec::new() } else { find_subscribers(root, &name) };
                stores.push(Store {
                    name,
                    file: path.to_path_buf(),
                    kind: kind.clone(),
                    subscribers,
                });
            }
        }
    }
    Ok(())
}

pub fn scan_stores(root: &Path) -> Result<Vec<Store>> {
    let mut stores = Vec::new();
    
    // Scan for different store patterns
    scan_zustand_stores(root, &mut stores)?;
    scan_redux_stores(root, &mut stores)?;
    scan_context_stores(root, &mut stores)?;
    scan_pinia_stores(root, &mut stores)?;
    scan_vuex_stores(root, &mut stores)?;
    scan_jotai_stores(root, &mut stores)?;
    scan_recoil_stores(root, &mut stores)?;
    scan_mobx_stores(root, &mut stores)?;
    scan_valtio_stores(root, &mut stores)?;
    scan_xstate_stores(root, &mut stores)?;
    scan_nanostores(root, &mut stores)?;
    scan_angular_stores(root, &mut stores)?;
    
    // Batch find subscribers (one walk instead of N walks)
    batch_find_subscribers(root, &mut stores);
    
    Ok(stores)
}

fn scan_zustand_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Zustand,
        &[r"const\s+(use\w+)\s*=\s*create\s*(?:<[^>]*>)?\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_redux_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Redux,
        &[
            r"const\s+(\w+Slice)\s*=\s*createSlice\s*\(",
            r"const\s+(\w*store\w*)\s*=\s*configureStore\s*\(",
        ],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_context_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Context,
        &[
            r"const\s+(\w+Context)\s*=\s*createContext\s*(?:<[^>]*>)?\s*\(",
            r"const\s+(\w+Provider)\s*=\s*\(\s*\{[^}]*\}\s*\)\s*=>",
        ],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_pinia_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Pinia,
        &[r"export\s+const\s+(use\w+Store)\s*=\s*defineStore\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_vuex_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Vuex,
        &[r"(?:export\s+(?:default\s+)?)?(?:const\s+(\w+)\s*=\s*)?createStore\s*\("],
        |caps| caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "vuex-store".to_string()),
        None,
        true,
    )
}

fn scan_jotai_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Jotai,
        &[r"const\s+(\w+Atom)\s*=\s*atom\s*(?:<[^>]*>)?\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_recoil_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Recoil,
        &[
            r"const\s+(\w+Atom)\s*=\s*atom\s*\(",
            r"const\s+(\w+Selector)\s*=\s*selector\s*\(",
        ],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_mobx_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Mobx,
        &[r"class\s+(\w+Store)\s*\{"],
        |caps| caps[1].to_string(),
        Some(r"make(?:Auto)?Observable\s*\("),
        true,
    )
}

fn scan_valtio_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Valtio,
        &[r"const\s+(\w+)\s*=\s*proxy\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_xstate_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Xstate,
        &[r"const\s+(\w+Machine)\s*=\s*createMachine\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_nanostores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Nanostores,
        &[r"const\s+(\$\w+)\s*=\s*(?:atom|map|deepMap)\s*\("],
        |caps| caps[1].to_string(),
        None,
        true,
    )
}

fn scan_angular_stores(root: &Path, stores: &mut Vec<Store>) -> Result<()> {
    scan_store_by_pattern(
        root, stores, StoreKind::Unknown,
        &[r"(?:export\s+)?class\s+(\w+Service)\s*\{"],
        |caps| caps[1].to_string(),
        Some(r#"@Injectable\s*\(\s*\{[^}]*providedIn\s*:\s*['"]root['"]"#),
        true,
    )
}

fn batch_find_subscribers(root: &Path, stores: &mut Vec<Store>) {
    if stores.is_empty() { return; }
    
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .add_custom_ignore_filename(".gitignore")
        .filter_entry(|e| {
            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                !matches!(name.as_ref(), "node_modules" | "dist" | "build" | ".next" | ".nuxt" | ".svelte-kit" | ".git" | ".svn" | "vendor" | "coverage" | "__pycache__" | ".cache")
            } else {
                true
            }
        })
        .build();
    
    // Pre-compile regexes for all store names
    let store_regexes: Vec<(String, Regex, Regex, Regex)> = stores.iter().map(|s| {
        let import_re = Regex::new(&format!(r"import\s+.*{}\s+.*from", regex::escape(&s.name))).expect("invalid regex");
        let use_re = Regex::new(&format!(r"{}\s*\(", regex::escape(&s.name))).expect("invalid regex");
        let inject_re = Regex::new(&format!(r"inject\s*\(\s*{}", regex::escape(&s.name))).expect("invalid regex");
        (s.name.clone(), import_re, use_re, inject_re)
    }).collect();
    
    for entry in walker {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) { continue; }
        let path = entry.path();
        if is_test_file(path) { continue; }
        let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => continue };
        let component_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
        
        for (store_name, import_re, use_re, inject_re) in &store_regexes {
            if import_re.is_match(&content) || use_re.is_match(&content) || inject_re.is_match(&content) {
                if let Some(store) = stores.iter_mut().find(|s| &s.name == store_name) {
                    store.subscribers.push(component_name.clone());
                }
            }
        }
    }
}

fn find_subscribers(root: &Path, store_name: &str) -> Vec<String> {
    let mut subscribers = Vec::new();
    
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .add_custom_ignore_filename(".gitignore")
        .filter_entry(|e| {
            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                !matches!(name.as_ref(), "node_modules" | "dist" | "build" | ".next" | ".nuxt" | ".svelte-kit" | ".git" | ".svn" | "vendor" | "coverage" | "__pycache__" | ".cache")
            } else {
                true
            }
        })
        .build();
    
    let import_re = Regex::new(&format!(r"import\s+.*{}\s+.*from", regex::escape(store_name))).expect("invalid regex pattern");
    let use_re = Regex::new(&format!(r"{}\s*\(", regex::escape(store_name))).expect("invalid regex pattern");
    let inject_re = Regex::new(&format!(r"inject\s*\(\s*{}", regex::escape(store_name))).expect("invalid regex pattern");
    
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        
        let path = entry.path();
        // Skip test files
        if is_test_file(path) {
            continue;
        }
        
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        if import_re.is_match(&content) || use_re.is_match(&content) || inject_re.is_match(&content) {
            let component_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            subscribers.push(component_name);
        }
    }
    
    subscribers
}

fn is_test_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/__tests__/") || path_str.contains("\\__tests\\") {
        return true;
    }
    if let Some(name) = path.file_stem() {
        let name = name.to_string_lossy();
        if name.ends_with(".test") || name.ends_with(".spec") {
            return true;
        }
    }
    false
}
