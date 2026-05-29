use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "frontendmap")]
#[command(version)]
#[command(about = "Frontend project satellite map — index, query, and navigate your web project")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a frontend project
    Index {
        /// Project root directory
        #[arg(default_value = ".")]
        project: String,

        /// Output file path
        #[arg(short, long, default_value = ".frontendmap/map.json")]
        output: String,
    },

    /// Query the project map
    Query {
        #[command(subcommand)]
        command: QueryCommands,
    },

    /// AI-oriented navigation
    Nav {
        #[command(subcommand)]
        command: NavCommands,
    },

    /// Static analysis
    Analyze {
        #[command(subcommand)]
        command: AnalyzeCommands,
    },
}

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Show project summary
    Summary,

    /// List all components
    Components {
        /// Filter by name
        #[arg(short, long)]
        name: Option<String>,

        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show component details
    Inspect {
        /// Component name
        name: String,
    },

    /// Find where a component is used
    UsedBy {
        /// Component name
        name: String,
    },

    /// List all routes
    Routes,

    /// List all API calls
    Apis {
        /// Filter by component
        #[arg(short, long)]
        component: Option<String>,
    },

    /// List all stores
    Stores,

    /// List entry points
    Entries,

    /// Find similar components
    Similar {
        /// Component name
        name: String,

        /// Max results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show component dependencies
    Deps {
        /// Component name
        name: String,

        /// Max depth
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },

    /// Show impact analysis
    Impact {
        /// Component name
        name: String,

        /// Max depth
        #[arg(short, long, default_value = "3")]
        depth: usize,
    },

    /// Show data flow through stores
    Flow {
        /// Store name
        name: String,

        /// Max depth
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },

    /// Show file/module scope
    Scope {
        /// File path or component name
        target: String,
    },

    /// Find shortest path between two components
    Path {
        /// From component
        from: String,

        /// To component
        to: String,
    },

    /// Export graph in various formats
    Export {
        /// Export format
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum NavCommands {
    /// Show entry points and callee chains
    Guide,

    /// Show project quality score
    Quality,

    /// Show health issues (cycles, god components, dead code)
    Health,

    /// Generate report
    Report {
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show compact project overview
    Map {
        /// Show full details
        #[arg(short, long)]
        full: bool,
    },
}

#[derive(Subcommand)]
pub enum AnalyzeCommands {
    /// Show module dependency matrix
    Deps {
        /// Filter by module
        #[arg(short, long)]
        from: Option<String>,
    },

    /// Show fan-in/fan-out analysis
    Fanout {
        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show test impact analysis
    Tests {
        /// Component name
        name: Option<String>,
    },
}
