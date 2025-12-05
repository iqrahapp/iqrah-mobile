use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;

mod content;
mod data_loader;
mod graph;
mod knowledge;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the knowledge graph and content database
    Build {
        /// Path to offline data directory
        #[arg(long)]
        data_dir: PathBuf,

        /// Path to morphology CSV file
        #[arg(long)]
        morphology: PathBuf,

        /// Output content database path
        #[arg(long)]
        output_db: PathBuf,

        /// Output graph file path (optional, for R&D visualization)
        #[arg(long)]
        output_graph: Option<PathBuf>,

        /// Chapter range (e.g., "1-114")
        #[arg(long, default_value = "1-114")]
        chapters: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            data_dir,
            morphology,
            output_db,
            output_graph,
            chapters,
        } => {
            println!("Starting build process...");
            let start = Instant::now();

            // 1. Build Content DB (schema + content data)
            println!("Building content database...");
            content::build(&data_dir, &morphology, &output_db).await?;

            // 2. Build Graph and write edges to content.db
            println!("Building knowledge graph...");
            // Pass output_graph as optional for GraphML export (useful for R&D)
            let graphml_path = output_graph.as_deref();
            graph::build(&data_dir, &morphology, &output_db, graphml_path, &chapters)?;

            let duration = start.elapsed();
            println!("Build complete in {:.2?}", duration);
        }
    }

    Ok(())
}
