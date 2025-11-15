use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "iqrah")]
#[command(about = "Iqrah CLI - Development and migration tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate from old single iqrah.db to new two-database architecture
    Migrate {
        /// Path to old iqrah.db
        #[arg(long)]
        old_db: String,

        /// Path for new content.db
        #[arg(long)]
        content_db: String,

        /// Path for new user.db
        #[arg(long)]
        user_db: String,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate { old_db, content_db, user_db } => {
            println!("Starting database migration...");
            println!("  Old DB: {}", old_db);
            println!("  Content DB: {}", content_db);
            println!("  User DB: {}", user_db);
            println!();

            let stats = iqrah_storage::migrate_database(&old_db, &content_db, &user_db)?;

            println!("\n✅ Migration Complete!");
            println!("  Nodes: {}", stats.nodes_migrated);
            println!("  Edges: {}", stats.edges_migrated);
            println!("  Arabic texts: {}", stats.arabic_texts_migrated);
            println!("  Translations: {}", stats.translations_migrated);
            println!("  Memory states: {}", stats.memory_states_migrated);
            println!("  Propagation events: {}", stats.propagation_events_migrated);

            Ok(())
        }
    }
}
