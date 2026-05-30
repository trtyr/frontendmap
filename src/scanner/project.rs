use anyhow::Result;
use std::fs;
use std::path::Path;
use crate::model::{ProjectInfo, Framework};
use serde_json::Value;

fn get_all_deps(json: &Value) -> std::collections::HashMap<String, bool> {
    let empty_map = serde_json::Map::new();
    let deps = json["dependencies"].as_object().unwrap_or(&empty_map);
    let dev_deps = json["devDependencies"].as_object().unwrap_or(&empty_map);
    deps.keys()
        .chain(dev_deps.keys())
        .map(|k| (k.clone(), true))
        .collect()
}

pub fn scan_project(root: &Path) -> Result<ProjectInfo> {
    let package_json_path = root.join("package.json");
    
    let (name, framework, features, tech_stack) = if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        let name = json["name"].as_str().unwrap_or("unknown").to_string();
        let framework = detect_framework(root, &json);
        let features = detect_features(&json);
        let tech_stack = detect_tech_stack(&json);
        
        (name, framework, features, tech_stack)
    } else {
        // Try to detect from directory structure
        let framework = detect_framework_from_structure(root);
        ("unknown".to_string(), framework, vec![], vec![])
    };
    
    // Count files
    let file_count = count_files(root);
    let component_count = 0; // Will be updated after component scan
    
    Ok(ProjectInfo {
        root: root.to_path_buf(),
        name,
        framework,
        features,
        tech_stack,
        file_count,
        component_count,
    })
}

fn detect_framework(root: &Path, json: &Value) -> Framework {
    let all_deps = get_all_deps(json);
    
    // Check for meta-frameworks first (they take priority)
    if all_deps.contains_key("next") || root.join("next.config.js").exists() || root.join("next.config.mjs").exists() {
        return Framework::Next;
    }
    
    if all_deps.contains_key("nuxt") || root.join("nuxt.config.ts").exists() || root.join("nuxt.config.js").exists() {
        return Framework::Nuxt;
    }
    
    if all_deps.contains_key("@sveltejs/kit") || root.join("svelte.config.js").exists() {
        return Framework::SvelteKit;
    }
    
    // Check for frameworks
    if all_deps.contains_key("react") || all_deps.contains_key("react-dom") {
        return Framework::React;
    }
    
    if all_deps.contains_key("vue") {
        return Framework::Vue;
    }
    
    if all_deps.contains_key("svelte") {
        return Framework::Svelte;
    }
    
    if all_deps.contains_key("@angular/core") {
        return Framework::Angular;
    }
    
    if all_deps.contains_key("solid-js") {
        return Framework::Solid;
    }
    
    if all_deps.contains_key("preact") {
        return Framework::Preact;
    }
    
    if all_deps.contains_key("@builder.io/qwik") {
        return Framework::Qwik;
    }
    
    if all_deps.contains_key("astro") {
        return Framework::Astro;
    }
    
    // Try to detect from structure
    detect_framework_from_structure(root)
}

fn detect_framework_from_structure(root: &Path) -> Framework {
    // Check for framework-specific config files
    if root.join("next.config.js").exists() || root.join("next.config.mjs").exists() || root.join("next.config.ts").exists() {
        return Framework::Next;
    }
    
    if root.join("nuxt.config.ts").exists() || root.join("nuxt.config.js").exists() {
        return Framework::Nuxt;
    }
    
    if root.join("svelte.config.js").exists() {
        return Framework::SvelteKit;
    }
    
    if root.join("angular.json").exists() {
        return Framework::Angular;
    }
    
    if root.join("vite.config.ts").exists() || root.join("vite.config.js").exists() {
        // Check for framework-specific vite config
        let vite_config = root.join("vite.config.ts");
        if vite_config.exists() {
            if let Ok(content) = fs::read_to_string(&vite_config) {
                if content.contains("@vitejs/plugin-react") {
                    return Framework::React;
                }
                if content.contains("@vitejs/plugin-vue") {
                    return Framework::Vue;
                }
                if content.contains("@sveltejs/vite-plugin-svelte") {
                    return Framework::Svelte;
                }
                if content.contains("vite-plugin-solid") {
                    return Framework::Solid;
                }
            }
        }
    }
    
    // Check for src directory structure
    let src_dir = root.join("src");
    if src_dir.exists() {
        // Check for Vue-style structure
        if src_dir.join("App.vue").exists() || src_dir.join("main.ts").exists() {
            // Could be Vue or React, check for more clues
            if src_dir.join("components").exists() {
                // Look for .vue files
                if has_files_with_extension(&src_dir, "vue") {
                    return Framework::Vue;
                }
                // Look for .jsx/.tsx files
                if has_files_with_extension(&src_dir, "jsx") || has_files_with_extension(&src_dir, "tsx") {
                    return Framework::React;
                }
            }
        }
        
        // Check for Svelte
        if has_files_with_extension(&src_dir, "svelte") {
            return Framework::Svelte;
        }
    }
    
    Framework::Unknown
}

fn has_files_with_extension(dir: &Path, ext: &str) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(file_ext) = entry.path().extension() {
                if file_ext == ext {
                    return true;
                }
            }
        }
    }
    false
}

fn detect_features(json: &Value) -> Vec<String> {
    let mut features = Vec::new();
    let all_deps = get_all_deps(json);
    
    // Routing
    let routing_deps = ["react-router", "react-router-dom", "vue-router", "@angular/router", 
                       "@sveltejs/kit", "next", "nuxt", "wouter", "tanstack-router"];
    if routing_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("routing".to_string());
    }
    
    // State management
    let state_deps = ["redux", "@reduxjs/toolkit", "zustand", "pinia", "vuex", 
                     "jotai", "recoil", "mobx", "valtio", "xstate", "nanostores",
                     "@ngrx/store", "@ngxs/store"];
    if state_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("state-management".to_string());
    }
    
    // HTTP clients
    let http_deps = ["axios", "ky", "got", "node-fetch", "superagent", "ofetch"];
    if http_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("http-client".to_string());
    }
    
    // CSS frameworks
    let css_deps = ["tailwindcss", "bootstrap", "bulma", "material-ui", "@mui/material", 
                   "antd", "ant-design-vue", "@chakra-ui/react", "@mantine/core"];
    if css_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("css-framework".to_string());
    }
    
    // CSS-in-JS
    let css_in_js_deps = ["styled-components", "@emotion/react", "@emotion/styled", "stitches"];
    if css_in_js_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("css-in-js".to_string());
    }
    
    // TypeScript
    if all_deps.contains_key("typescript") || all_deps.contains_key("ts-node") {
        features.push("typescript".to_string());
    }
    
    // Testing
    let test_deps = ["jest", "vitest", "@testing-library/react", "@testing-library/vue", 
                    "cypress", "playwright", "@playwright/test"];
    if test_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("testing".to_string());
    }
    
    // Build tools
    let build_deps = ["vite", "webpack", "esbuild", "rollup", "parcel", "turbopack"];
    if build_deps.iter().any(|d| all_deps.contains_key(*d)) {
        features.push("build-tool".to_string());
    }
    
    features
}

fn detect_tech_stack(json: &Value) -> Vec<String> {
    let mut stack = Vec::new();
    let all_deps = get_all_deps(json);
    
    // Add framework
    let framework = detect_framework_from_json(json);
    if framework != Framework::Unknown {
        stack.push(framework.as_str().to_string());
    }
    
    // Add major dependencies (limit to top 10)
    let major_deps = [
        // UI Libraries
        "antd", "ant-design-vue", "@mui/material", "@chakra-ui/react", "@mantine/core",
        "element-plus", "naive-ui", "vuetify",
        // State Management
        "redux", "@reduxjs/toolkit", "zustand", "pinia", "vuex", "jotai", "recoil", "mobx",
        // HTTP Clients
        "axios", "ky", "got", "ofetch",
        // CSS
        "tailwindcss", "styled-components", "@emotion/react",
        // Build Tools
        "vite", "webpack", "esbuild",
        // Testing
        "jest", "vitest", "cypress", "playwright",
        // Utilities
        "lodash", "dayjs", "moment", "date-fns",
    ];
    
    let mut count = 0;
    for dep in major_deps {
        if all_deps.contains_key(dep) {
            stack.push(dep.to_string());
            count += 1;
            if count >= 10 {
                break;
            }
        }
    }
    
    stack
}

fn detect_framework_from_json(json: &Value) -> Framework {
    let all_deps = get_all_deps(json);
    
    if all_deps.contains_key("next") {
        return Framework::Next;
    }
    if all_deps.contains_key("nuxt") {
        return Framework::Nuxt;
    }
    if all_deps.contains_key("@sveltejs/kit") {
        return Framework::SvelteKit;
    }
    if all_deps.contains_key("react") {
        return Framework::React;
    }
    if all_deps.contains_key("vue") {
        return Framework::Vue;
    }
    if all_deps.contains_key("svelte") {
        return Framework::Svelte;
    }
    if all_deps.contains_key("@angular/core") {
        return Framework::Angular;
    }
    if all_deps.contains_key("solid-js") {
        return Framework::Solid;
    }
    if all_deps.contains_key("preact") {
        return Framework::Preact;
    }
    if all_deps.contains_key("@builder.io/qwik") {
        return Framework::Qwik;
    }
    if all_deps.contains_key("astro") {
        return Framework::Astro;
    }
    
    Framework::Unknown
}

fn count_files(root: &Path) -> usize {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .build();
    
    let mut count = 0;
    for entry in walker {
        if let Ok(entry) = entry {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Some(ext) = entry.path().extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if matches!(ext.as_str(), "js" | "jsx" | "ts" | "tsx" | "vue" | "svelte" | "astro" | "qwik") {
                        count += 1;
                    }
                }
            }
        }
    }
    
    count
}
