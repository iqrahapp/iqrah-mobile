use anyhow::Result;
use clap::{Parser, Subcommand};

mod debug;
mod exercise;
mod import;
mod integrity;
mod package;
mod schedule;
mod translator;
mod verify_update;

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
    /// Import CBOR graph data into database
    Import {
        /// Path to CBOR graph file (e.g., iqrah-graph-v1.0.0.cbor.zst)
        cbor_file: String,
    },
    /// Translator management commands
    Translator {
        #[command(subcommand)]
        command: TranslatorCommands,
    },
    /// Package management commands
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
    /// Generate a learning session using scheduler v2
    Schedule {
        /// User ID
        #[arg(long)]
        user_id: String,
        /// Goal ID (e.g., "memorization:surah-1")
        #[arg(long)]
        goal_id: String,
        /// Session size (number of items)
        #[arg(long, default_value = "20")]
        session_size: usize,
        /// Session mode (revision or mixed-learning)
        #[arg(long, default_value = "mixed-learning")]
        mode: String,
        /// Enable bandit optimization (Thompson Sampling for profile selection)
        #[arg(long)]
        enable_bandit: bool,
        /// Verbose output (show detailed node information and profile weights)
        #[arg(long, short)]
        verbose: bool,
    },
    /// Check database integrity (find orphaned user records)
    CheckIntegrity {
        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
    /// Verify content.db update compatibility with user progress
    VerifyUpdate {
        /// Path to old content.db
        #[arg(long)]
        old_db: String,
        /// Path to new content.db
        #[arg(long)]
        new_db: String,
        /// Path to user.db (default: data/user.db)
        #[arg(long, default_value = "data/user.db")]
        user_db: String,
        /// User ID to check (default: "default")
        #[arg(long, default_value = "default")]
        user_id: String,
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
enum TranslatorCommands {
    /// List all available languages
    ListLanguages,
    /// List translators for a language
    ListTranslators {
        /// Language code (e.g., 'en', 'ar')
        language_code: String,
    },
    /// Get translator details by ID
    GetTranslator {
        /// Translator ID
        translator_id: i32,
    },
    /// Get user's preferred translator
    GetPreferred {
        /// User ID
        user_id: String,
    },
    /// Set user's preferred translator
    SetPreferred {
        /// User ID
        user_id: String,
        /// Translator ID
        translator_id: i32,
    },
    /// Get verse translation for specific translator
    GetTranslation {
        /// Verse key (e.g., '1:1')
        verse_key: String,
        /// Translator ID
        translator_id: i32,
    },
    /// Import translators from JSON file
    Import {
        /// Path to translator metadata JSON file
        metadata_file: String,
        /// Base path for translation files
        translations_base: String,
    },
}

#[derive(Subcommand)]
enum PackageCommands {
    /// List all available packages
    List,
    /// Get package details by ID
    Get {
        /// Package ID
        package_id: String,
    },
    /// List installed packages
    ListInstalled,
    /// Install a package
    Install {
        /// Package ID to install
        package_id: String,
    },
    /// Uninstall a package
    Uninstall {
        /// Package ID to uninstall
        package_id: String,
    },
    /// Enable a package
    Enable {
        /// Package ID to enable
        package_id: String,
    },
    /// Disable a package
    Disable {
        /// Package ID to disable
        package_id: String,
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
            DebugCommands::SetState {
                user_id,
                node_id,
                energy,
            } => {
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
        Commands::Import { cbor_file } => {
            import::import_cbor(&cbor_file).await?;
        }
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
        }
        Commands::Translator { command } => match command {
            TranslatorCommands::ListLanguages => {
                translator::list_languages(&cli.server).await?;
            }
            TranslatorCommands::ListTranslators { language_code } => {
                translator::list_translators(&cli.server, &language_code).await?;
            }
            TranslatorCommands::GetTranslator { translator_id } => {
                translator::get_translator(&cli.server, translator_id).await?;
            }
            TranslatorCommands::GetPreferred { user_id } => {
                translator::get_preferred(&cli.server, &user_id).await?;
            }
            TranslatorCommands::SetPreferred {
                user_id,
                translator_id,
            } => {
                translator::set_preferred(&cli.server, &user_id, translator_id).await?;
            }
            TranslatorCommands::GetTranslation {
                verse_key,
                translator_id,
            } => {
                translator::get_translation(&cli.server, &verse_key, translator_id).await?;
            }
            TranslatorCommands::Import {
                metadata_file,
                translations_base,
            } => {
                translator::import_translators(&metadata_file, &translations_base).await?;
            }
        },
        Commands::Package { command } => match command {
            PackageCommands::List => {
                package::list_packages(&cli.server).await?;
            }
            PackageCommands::Get { package_id } => {
                package::get_package(&cli.server, &package_id).await?;
            }
            PackageCommands::ListInstalled => {
                package::list_installed(&cli.server).await?;
            }
            PackageCommands::Install { package_id } => {
                package::install_package(&cli.server, &package_id).await?;
            }
            PackageCommands::Uninstall { package_id } => {
                package::uninstall_package(&cli.server, &package_id).await?;
            }
            PackageCommands::Enable { package_id } => {
                package::enable_package(&cli.server, &package_id).await?;
            }
            PackageCommands::Disable { package_id } => {
                package::disable_package(&cli.server, &package_id).await?;
            }
        },
        Commands::Schedule {
            user_id,
            goal_id,
            session_size,
            mode,
            enable_bandit,
            verbose,
        } => {
            schedule::generate(
                &user_id,
                &goal_id,
                session_size,
                &mode,
                enable_bandit,
                verbose,
            )
            .await?;
        }
        Commands::CheckIntegrity { verbose } => {
            integrity::check_integrity(verbose).await?;
        }
        Commands::VerifyUpdate {
            old_db,
            new_db,
            user_db,
            user_id,
        } => {
            let result = verify_update::verify_update(&old_db, &new_db, &user_db, &user_id).await?;
            if !result.missing_nodes.is_empty() {
                std::process::exit(1); // Non-zero exit for CI integration
            }
        }
    }

    Ok(())
}
