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

#[tokio::main]
async fn main() -> Result<()> {
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

            // Initialize database connections
            let content_pool = iqrah_storage::init_content_db(&content_db).await?;
            let user_pool = iqrah_storage::init_user_db(&user_db).await?;

            // Run migration
            iqrah_storage::migrate_from_old_db(&old_db, &content_pool, &user_pool).await?;

            println!("\n✅ Migration Complete!");
            println!("  (Migration implementation pending - databases initialized)");

            Ok(())
        }
    }
}
