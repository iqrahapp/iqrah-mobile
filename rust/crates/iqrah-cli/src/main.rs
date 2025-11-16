use anyhow::Result;
use clap::{Parser, Subcommand};

mod debug;
mod exercise;

/// Iqrah CLI - Development and testing tool for the Iqrah learning system
#[derive(Parser)]
#[command(name = "iqrah")]
#[command(about = "Iqrah CLI tool for testing and development", long_about = None)]
struct Cli {
    /// Server URL (default: http://127.0.0.1:3000)
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Debug commands for inspecting server state
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },
    /// Interactive exercise commands
    Exercise {
        #[command(subcommand)]
        command: ExerciseCommands,
    },
}

#[derive(Subcommand)]
enum DebugCommands {
    /// Get node metadata
    GetNode {
        /// Node ID (e.g., VERSE:2:255)
        node_id: String,
    },
    /// Get user memory state for a node
    GetState {
        /// User ID
        user_id: String,
        /// Node ID
        node_id: String,
    },
    /// Process a single review
    ProcessReview {
        /// User ID
        user_id: String,
        /// Node ID
        node_id: String,
        /// Review grade (Again, Hard, Good, Easy)
        grade: String,
    },
}

#[derive(Subcommand)]
enum ExerciseCommands {
    /// Run an interactive exercise session via WebSocket
    Run {
        /// Exercise type (e.g., MemorizationAyah)
        exercise_type: String,
        /// Node ID (e.g., VERSE:2:255)
        node_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Debug { command } => match command {
            DebugCommands::GetNode { node_id } => {
                debug::get_node(&cli.server, &node_id).await?;
            }
            DebugCommands::GetState { user_id, node_id } => {
                debug::get_state(&cli.server, &user_id, &node_id).await?;
            }
            DebugCommands::ProcessReview {
                user_id,
                node_id,
                grade,
            } => {
                debug::process_review(&cli.server, &user_id, &node_id, &grade).await?;
            }
        },
        Commands::Exercise { command } => match command {
            ExerciseCommands::Run {
                exercise_type,
                node_id,
            } => {
                exercise::run(&cli.server, &exercise_type, &node_id).await?;
            }
        },
    }

    Ok(())
}
