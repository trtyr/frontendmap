use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use crate::model::ApiCall;

pub fn scan_api_calls(root: &Path) -> Result<Vec<ApiCall>> {
    let mut api_calls = Vec::new();
    
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    // Define patterns for various HTTP clients and wrappers
    let patterns = vec![
        // Standard fetch
        (r#"fetch\s*\(\s*["']([^"']+)["']"#, "fetch", "GET"),
        // Axios
        (r#"axios\.(get|post|put|delete|patch|head|options)\s*\(\s*["']([^"']+)["']"#, "axios", ""),
        // Ky
        (r#"ky\.(get|post|put|delete|patch|head)\s*\(\s*["']([^"']+)["']"#, "ky", ""),
        // Got
        (r#"got\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#, "got", ""),
        // Superagent
        (r#"superagent\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#, "superagent", ""),
        // OFetch (Nuxt/Vite)
        (r#"ofetch\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#, "ofetch", ""),
        // Custom wrappers (common patterns)
        (r#"(?:request|apiFetch|httpClient|api|fetchApi|makeRequest)\s*(?:<[^>]*>)?\s*\(\s*["']([^"']+)["']"#, "custom", "GET"),
        // GraphQL
        (r#"useQuery\s*\(\s*gql`\s*(?:query\s+)?(\w+)"#, "graphql", "QUERY"),
        (r#"useMutation\s*\(\s*gql`\s*mutation\s+(\w+)"#, "graphql", "MUTATION"),
        (r#"gql`\s*(?:query|mutation)\s+(\w+)"#, "graphql", ""),
        // React Query / TanStack Query
        (r#"queryKey\s*:\s*\[["']([^"']+)["']"#, "react-query", "GET"),
        (r#"mutationKey\s*:\s*\[["']([^"']+)["']"#, "react-query", "MUTATION"),
    ];
    
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
        
        // Get component/module name from filename
        let module_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            for (pattern, client, default_method) in &patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if let Some(caps) = re.captures(line) {
                        let endpoint = if caps.len() > 2 {
                            caps[2].to_string()
                        } else {
                            caps[1].to_string()
                        };
                        
                        let method = if caps.len() > 2 && !caps[1].is_empty() {
                            caps[1].to_uppercase()
                        } else if !default_method.is_empty() {
                            default_method.to_string()
                        } else {
                            detect_method_from_context(&lines, line_num)
                        };
                        
                        // Skip test files and mock endpoints
                        let file_str = path.to_string_lossy().to_lowercase();
                        if file_str.contains("test") || file_str.contains("mock") || file_str.contains("__tests__") {
                            continue;
                        }
                        
                        api_calls.push(ApiCall {
                            component: module_name.clone(),
                            file: path.to_path_buf(),
                            endpoint,
                            method,
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
        
        // Check for API function definitions (multiline)
        scan_api_functions(&content, path, &module_name, &mut api_calls);
    }
    
    Ok(api_calls)
}

fn scan_api_functions(content: &str, path: &Path, module_name: &str, api_calls: &mut Vec<ApiCall>) {
    // Pattern for exported API functions
    let func_patterns = vec![
        // export async function fetchXxx() { return request('/api/xxx') }
        r#"export\s+(?:async\s+)?function\s+(\w+)\s*\([^)]*\)\s*\{[^}]*(?:request|apiFetch|fetch|axios)\s*(?:<[^>]*>)?\s*\(\s*["']([^"']+)["']"#,
        // export const fetchXxx = async () => { return request('/api/xxx') }
        r#"export\s+const\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>\s*\{[^}]*(?:request|apiFetch|fetch|axios)\s*(?:<[^>]*>)?\s*\(\s*["']([^"']+)["']"#,
        // export function getXxx() { return api.get('/api/xxx') }
        r#"export\s+(?:async\s+)?function\s+(\w+)\s*\([^)]*\)\s*\{[^}]*\w+\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#,
    ];
    
    for pattern in func_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(content) {
                let func_name = caps[1].to_string();
                let endpoint = if caps.len() > 3 {
                    caps[3].to_string()
                } else {
                    caps[2].to_string()
                };
                let method = if caps.len() > 3 {
                    caps[2].to_uppercase()
                } else {
                    "GET".to_string()
                };
                let line = content[..caps.get(0).expect("regex match should have a capture group").start()].lines().count() + 1;
                
                api_calls.push(ApiCall {
                    component: func_name,
                    file: path.to_path_buf(),
                    endpoint,
                    method,
                    line,
                });
            }
        }
    }
}

fn detect_method_from_context(lines: &[&str], current_line: usize) -> String {
    // Look in nearby lines for method hints
    let start = current_line.saturating_sub(5);
    let end = (current_line + 5).min(lines.len());
    
    for i in start..end {
        let line = lines[i].to_lowercase();
        
        // Check for method in options object
        if line.contains("method") {
            if line.contains("post") {
                return "POST".to_string();
            }
            if line.contains("put") {
                return "PUT".to_string();
            }
            if line.contains("delete") {
                return "DELETE".to_string();
            }
            if line.contains("patch") {
                return "PATCH".to_string();
            }
            if line.contains("head") {
                return "HEAD".to_string();
            }
            if line.contains("options") {
                return "OPTIONS".to_string();
            }
        }
        
        // Check for POST indicators
        if line.contains("body") || line.contains("payload") || line.contains("create") || line.contains("add") {
            return "POST".to_string();
        }
        
        // Check for PUT indicators
        if line.contains("update") || line.contains("edit") {
            return "PUT".to_string();
        }
        
        // Check for DELETE indicators
        if line.contains("delete") || line.contains("remove") {
            return "DELETE".to_string();
        }
    }
    
    "GET".to_string()
}
