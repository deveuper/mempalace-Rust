#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod cli;
mod config;
mod dialect;
mod entity;
mod knowledge_graph;
mod layers;
mod mcp;
mod miner;
mod room;
mod search;
mod storage;
mod utils;

use cli::{handle_init, handle_mine, handle_search, handle_status, handle_wakeup};
use config::Config;

#[derive(Parser)]
#[command(name = "mempalace")]
#[command(about = "Give your AI a memory — mine projects and conversations into a searchable palace")]
#[command(version = "3.0.0-v3")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Palace directory (default: ~/.mempalace)
    #[arg(short, long, global = true)]
    palace: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new palace for a directory
    Init {
        /// Directory to initialize
        dir: PathBuf,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// Mine files into the palace
    Mine {
        /// Directory to mine
        dir: PathBuf,
        /// Mining mode: projects, convos, or general
        #[arg(short, long, default_value = "projects")]
        mode: String,
        /// Extract mode for general mining
        #[arg(short, long)]
        extract: Option<String>,
    },

    /// Search the palace
    Search {
        /// Search query
        query: String,
        /// Filter by wing
        #[arg(short, long)]
        wing: Option<String>,
        /// Filter by room
        #[arg(short, long)]
        room: Option<String>,
        /// Number of results
        #[arg(short, long, default_value = "5")]
        n: usize,
    },

    /// Show wake-up context
    #[command(name = "wake-up")]
    WakeUp {
        /// Filter by wing
        #[arg(short, long)]
        wing: Option<String>,
    },

    /// Show palace status
    Status,

    /// Run MCP server
    Mcp {
        /// Transport type
        #[arg(short, long, default_value = "stdio")]
        transport: String,
    },

    /// Compress content using AAAK dialect
    Compress {
        /// File to compress
        file: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Repair palace integrity
    Repair,

    /// Split mega-files into per-session files
    Split {
        /// File to split
        file: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            if cli.verbose {
                "mempalace=debug"
            } else {
                "mempalace=info"
            },
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("MemPalace v3.0.0 - Rust Edition");

    let config = Config::load(cli.palace).await?;

    match cli.command {
        Commands::Init { dir, yes } => {
            handle_init(&config, dir, yes).await?;
        }
        Commands::Mine { dir, mode, extract } => {
            handle_mine(&config, dir, &mode, extract).await?;
        }
        Commands::Search { query, wing, room, n } => {
            handle_search(&config, &query, wing, room, n).await?;
        }
        Commands::WakeUp { wing } => {
            handle_wakeup(&config, wing).await?;
        }
        Commands::Status => {
            handle_status(&config).await?;
        }
        Commands::Mcp { transport } => {
            mcp::run_server(&config, &transport).await?;
        }
        Commands::Compress { file, output } => {
            cli::handle_compress(&config, file, output).await?;
        }
        Commands::Repair => {
            cli::handle_repair(&config).await?;
        }
        Commands::Split { file, output } => {
            cli::handle_split(&config, file, output).await?;
        }
    }

    Ok(())
}
