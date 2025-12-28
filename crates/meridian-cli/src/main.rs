use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(name = "meridian")]
#[command(version, about = "Meridian GIS Platform CLI", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import geospatial data from various formats
    Import(commands::import::ImportArgs),

    /// Export geospatial data to various formats
    Export(commands::export::ExportArgs),

    /// Query spatial data
    Query(commands::query::QueryArgs),

    /// Start the Meridian API server
    Serve(commands::serve::ServeArgs),

    /// Perform spatial analysis operations
    Analyze(commands::analyze::AnalyzeArgs),

    /// Database management commands
    Db(commands::db::DbArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("meridian={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Execute the command
    match cli.command {
        Commands::Import(args) => commands::import::execute(args).await,
        Commands::Export(args) => commands::export::execute(args).await,
        Commands::Query(args) => commands::query::execute(args).await,
        Commands::Serve(args) => commands::serve::execute(args).await,
        Commands::Analyze(args) => commands::analyze::execute(args).await,
        Commands::Db(args) => commands::db::execute(args).await,
    }
}
