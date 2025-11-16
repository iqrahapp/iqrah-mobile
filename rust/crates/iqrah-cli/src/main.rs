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
    /// Set user memory state for a node
    SetState {
        /// User ID
        user_id: String,
        /// Node ID
        node_id: String,
        /// Energy level (0.0 to 1.0)
        #[arg(long)]
        energy: f64,
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
    /// Start an Echo Recall session
    Start {
        /// Exercise type (echo-recall)
        exercise_type: String,
        /// Ayah node IDs (e.g., VERSE:103:2)
        ayah_node_ids: Vec<String>,
    },
    /// Submit an action in an exercise session
    Action {
        /// Exercise type (echo-recall)
        exercise_type: String,
        /// Session ID
        session_id: String,
        /// Word node ID
        word_node_id: String,
        /// Recall time in milliseconds
        recall_time_ms: u32,
    },
    /// End an exercise session
    End {
        /// Session ID
        session_id: String,
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
            DebugCommands::SetState { user_id, node_id, energy } => {
                debug::set_state(&cli.server, &user_id, &node_id, energy).await?;
            }
            DebugCommands::ProcessReview {
                user_id,
                node_id,
                grade,
            } => {
                debug::process_review(&cli.server, &user_id, &node_id, &grade).await?;
            }
        },
        Commands::Exercise { command } => {
            // Create server configuration once for all exercise commands
            let config = exercise::ServerConfig::new(&cli.server)?;

            match command {
                ExerciseCommands::Run {
                    exercise_type,
                    node_id,
                } => {
                    exercise::run(&config, &exercise_type, &node_id).await?;
                }
                ExerciseCommands::Start {
                    exercise_type,
                    ayah_node_ids,
                } => {
                    exercise::start(&config, &exercise_type, &ayah_node_ids).await?;
                }
                ExerciseCommands::Action {
                    exercise_type,
                    session_id,
                    word_node_id,
                    recall_time_ms,
                } => {
                    exercise::action(
                        &config,
                        &exercise_type,
                        &session_id,
                        &word_node_id,
                        recall_time_ms,
                    )
                    .await?;
                }
                ExerciseCommands::End { session_id } => {
                    exercise::end(&config, &session_id).await?;
                }
            }
        },
    }

    Ok(())
}
