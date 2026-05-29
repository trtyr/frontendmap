mod model;
mod cli;
mod scanner;
mod store;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, QueryCommands, NavCommands, AnalyzeCommands};
use commands::{query, nav, analyze};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { project, output } => {
            let map = scanner::scan(&project)?;
            store::save(&map, &output)?;
            println!("✓ Indexed {} components in {} files", 
                map.project.component_count, map.project.file_count);
            println!("  Output: {}", output);
        }
        Commands::Query { command } => {
            let map = store::load(".")?;
            match command {
                QueryCommands::Summary => {
                    println!("Project: {}", map.project.name);
                    println!("Framework: {}", map.project.framework.as_str());
                    println!("Components: {}", map.project.component_count);
                    println!("Files: {}", map.project.file_count);
                    println!("Routes: {}", map.routes.len());
                    println!("API Calls: {}", map.api_calls.len());
                    println!("Stores: {}", map.stores.len());
                    if !map.project.features.is_empty() {
                        println!("Features: {}", map.project.features.join(", "));
                    }
                    if !map.project.tech_stack.is_empty() {
                        println!("Tech Stack: {}", map.project.tech_stack.join(", "));
                    }
                }
                QueryCommands::Components { name, limit } => {
                    let components = match name {
                        Some(n) => map.components.iter()
                            .filter(|c| c.name.to_lowercase().contains(&n.to_lowercase()))
                            .collect::<Vec<_>>(),
                        None => map.components.iter().collect(),
                    };
                    for (i, comp) in components.iter().enumerate() {
                        if i >= limit { break; }
                        println!("{} ({}) - {}", comp.name, comp.kind, comp.file.display());
                    }
                    println!("\nTotal: {} components", components.len());
                }
                QueryCommands::Inspect { name } => {
                    if let Some(comp) = map.components.iter().find(|c| c.name == name) {
                        println!("Component: {}", comp.name);
                        println!("File: {}", comp.file.display());
                        println!("Type: {}", comp.kind);
                        println!("Line: {}", comp.line);
                        if !comp.props.is_empty() {
                            println!("Props:");
                            for prop in &comp.props {
                                let type_str = prop.type_annotation.as_deref().unwrap_or("any");
                                let req = if prop.required { "required" } else { "optional" };
                                println!("  - {}: {} ({})", prop.name, type_str, req);
                            }
                        }
                        if !comp.used_by.is_empty() {
                            println!("Used by:");
                            for user in &comp.used_by {
                                println!("  - {}", user.display());
                            }
                        }
                        if !comp.uses.is_empty() {
                            println!("Uses:");
                            for used in &comp.uses {
                                println!("  - {}", used);
                            }
                        }
                    } else {
                        eprintln!("Component '{}' not found", name);
                    }
                }
                QueryCommands::UsedBy { name } => {
                    if let Some(comp) = map.components.iter().find(|c| c.name == name) {
                        if comp.used_by.is_empty() {
                            println!("{} is not used by any component", name);
                        } else {
                            println!("{} is used by:", name);
                            for user in &comp.used_by {
                                println!("  - {}", user.display());
                            }
                        }
                    } else {
                        eprintln!("Component '{}' not found", name);
                    }
                }
                QueryCommands::Routes => {
                    if map.routes.is_empty() {
                        println!("No routes found");
                    } else {
                        for route in &map.routes {
                            println!("{} → {} ({})", route.path, route.component, route.file.display());
                        }
                        println!("\nTotal: {} routes", map.routes.len());
                    }
                }
                QueryCommands::Apis { component } => {
                    let apis = match component {
                        Some(c) => map.api_calls.iter()
                            .filter(|a| a.component.to_lowercase().contains(&c.to_lowercase()))
                            .collect::<Vec<_>>(),
                        None => map.api_calls.iter().collect(),
                    };
                    for api in &apis {
                        println!("{} {} → {} ({})", api.method, api.endpoint, 
                            api.component, api.file.display());
                    }
                    println!("\nTotal: {} API calls", apis.len());
                }
                QueryCommands::Stores => {
                    if map.stores.is_empty() {
                        println!("No stores found");
                    } else {
                        for store in &map.stores {
                            println!("{} ({}) - {}", store.name, store.kind, store.file.display());
                            if !store.subscribers.is_empty() {
                                println!("  Subscribers: {}", store.subscribers.join(", "));
                            }
                        }
                        println!("\nTotal: {} stores", map.stores.len());
                    }
                }
                QueryCommands::Entries => {
                    query::query_entries(&map);
                }
                QueryCommands::Similar { name, limit } => {
                    query::query_similar(&map, &name, limit);
                }
                QueryCommands::Deps { name, depth } => {
                    query::query_deps(&map, &name, depth);
                }
                QueryCommands::Impact { name, depth } => {
                    query::query_impact(&map, &name, depth);
                }
                QueryCommands::Flow { name, depth } => {
                    query::query_flow(&map, &name, depth);
                }
                QueryCommands::Scope { target } => {
                    query::query_scope(&map, &target);
                }
                QueryCommands::Path { from, to } => {
                    query::query_path(&map, &from, &to);
                }
                QueryCommands::Export { format, output } => {
                    query::query_export(&map, &format, output.as_deref());
                }
            }
        }
        Commands::Nav { command } => {
            let map = store::load(".")?;
            match command {
                NavCommands::Guide => {
                    nav::nav_guide(&map);
                }
                NavCommands::Quality => {
                    nav::nav_quality(&map);
                }
                NavCommands::Health => {
                    nav::nav_health(&map);
                }
                NavCommands::Report { output } => {
                    nav::nav_report(&map, output.as_deref());
                }
                NavCommands::Map { full } => {
                    nav::nav_map(&map, full);
                }
            }
        }
        Commands::Analyze { command } => {
            let map = store::load(".")?;
            match command {
                AnalyzeCommands::Deps { from } => {
                    analyze::analyze_deps(&map, from.as_deref());
                }
                AnalyzeCommands::Fanout { limit } => {
                    analyze::analyze_fanout(&map, limit);
                }
                AnalyzeCommands::Tests { name } => {
                    analyze::analyze_tests(&map, name.as_deref());
                }
            }
        }
    }

    Ok(())
}
