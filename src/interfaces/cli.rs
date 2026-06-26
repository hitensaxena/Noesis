use clap::{Parser, Subcommand};

/// CLI interface for the Noesis cognitive architecture.
#[derive(Parser, Debug)]
#[command(
    name = "noesis",
    about = "Decentralized cognitive architecture — emergent intelligence through recursive signal propagation",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the Noesis daemon
    Start {
        /// Enable REST API
        #[arg(long, default_value_t = false)]
        rest: bool,

        /// REST API port
        #[arg(long, default_value_t = 8647)]
        port: u16,

        /// Enable MCP protocol server (for AI agent tool calling)
        #[arg(long, default_value_t = false)]
        mcp: bool,

        /// MCP server port
        #[arg(long, default_value_t = 8645)]
        mcp_port: u16,

        /// Storage backend: memory or postgres
        #[arg(long, default_value = "memory")]
        storage: String,

        /// Postgres connection URL (overrides auto-detection)
        #[arg(long)]
        database_url: Option<String>,

        /// Redis connection URL (overrides auto-detection)
        #[arg(long)]
        redis_url: Option<String>,
    },

    /// Inject a raw experience into the system
    Inject {
        /// The experience text to inject
        text: String,

        /// Source identifier
        #[arg(long, default_value = "cli")]
        source: String,
    },

    /// Inspect a component's state
    Inspect {
        /// Component to inspect (field|processor|signals|all)
        #[arg(default_value = "all")]
        target: String,

        /// Optional specific component name
        name: Option<String>,
    },

    /// List registered components
    List {
        /// What to list (fields|processors|signals)
        #[arg(default_value = "all")]
        target: String,
    },

    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        action: Option<PluginCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List loaded plugins
    List,
    /// Load a plugin from a path
    Load { path: String },
}
