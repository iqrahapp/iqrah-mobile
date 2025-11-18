use anyhow::Result;
use chrono::Utc;
use colored::*;
use iqrah_core::{
    scheduler_v2::{
        blend_profile, generate_session, BanditOptimizer, ProfileName, SessionMode, UserProfile,
        BLEND_RATIO, DEFAULT_SAFE_PROFILE,
    },
    ContentRepository, UserRepository,
};
use iqrah_storage::{
    content::{init_content_db, SqliteContentRepository},
    user::{init_user_db, SqliteUserRepository},
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;

/// Generate a learning session using scheduler v2
pub async fn generate(
    user_id: &str,
    goal_id: &str,
    session_size: usize,
    mode: &str,
    enable_bandit: bool,
    verbose: bool,
) -> Result<()> {
    println!(
        "🎯 {}",
        format!("Generating session for goal: {}", goal_id)
            .bright_cyan()
            .bold()
    );
    println!();

    // Get database paths from environment or use defaults
    let content_db_path =
        std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "data/content.db".to_string());
    let user_db_path = std::env::var("USER_DB_PATH").unwrap_or_else(|_| "data/user.db".to_string());

    println!("   {}: {}", "Content DB".dimmed(), content_db_path.dimmed());
    println!("   {}: {}", "User DB".dimmed(), user_db_path.dimmed());
    println!();

    // Initialize databases
    let content_pool = init_content_db(&content_db_path).await?;
    let user_pool = init_user_db(&user_db_path).await?;

    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

    // Parse session mode
    let session_mode = match mode {
        "revision" => SessionMode::Revision,
        "mixed-learning" | "mixed" => SessionMode::MixedLearning,
        _ => anyhow::bail!("Invalid session mode. Use 'revision' or 'mixed-learning'"),
    };

    // Get current timestamp
    let now_ts = Utc::now().timestamp_millis();

    // Fetch candidates from content repository (returns defaults for energy/next_due_ts)
    println!("   Fetching candidates for goal...");
    let mut candidates = content_repo
        .get_scheduler_candidates(goal_id, user_id, now_ts)
        .await?;

    if candidates.is_empty() {
        println!();
        println!(
            "❌ {}",
            format!("No candidates found for goal '{}'", goal_id)
                .red()
                .bold()
        );
        println!(
            "   {}",
            "Make sure the goal exists and has nodes assigned to it.".yellow()
        );
        return Ok(());
    }

    println!(
        "   {} {}",
        "Found".green(),
        format!("{} candidate nodes", candidates.len())
            .green()
            .bold()
    );

    // Fetch memory states from user repository and merge
    println!("   Fetching user memory states...");
    let node_ids: Vec<String> = candidates.iter().map(|c| c.id.clone()).collect();
    let memory_basics_map = user_repo.get_memory_basics(user_id, &node_ids).await?;

    // Merge memory states into candidates
    for candidate in &mut candidates {
        if let Some(basics) = memory_basics_map.get(&candidate.id) {
            candidate.energy = basics.energy;
            candidate.next_due_ts = basics.next_due_ts;
        }
    }

    println!(
        "   Found memory states for {} nodes",
        memory_basics_map.len()
    );

    // Fetch prerequisite parent relationships
    println!("   Fetching prerequisite relationships...");
    let parent_map = content_repo.get_prerequisite_parents(&node_ids).await?;
    let parent_count: usize = parent_map.values().map(|v| v.len()).sum();
    println!("   Found {} prerequisite edges", parent_count);

    // Collect all parent IDs and fetch their energies
    let all_parent_ids: Vec<String> = parent_map
        .values()
        .flatten()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    println!("   Fetching parent energies...");
    let parent_energies = user_repo
        .get_parent_energies(user_id, &all_parent_ids)
        .await?;
    println!("   Found energies for {} parents", parent_energies.len());

    // Determine user profile (with optional bandit optimization)
    let (profile, chosen_profile_name) = if enable_bandit {
        // Fetch goal to get goal_group
        println!("   Fetching goal metadata...");
        let goal = content_repo.get_goal(goal_id).await?;

        let goal_group = goal
            .as_ref()
            .map(|g| g.goal_group.as_str())
            .unwrap_or("default");

        println!("   Goal group: {}", goal_group);

        // Fetch bandit arms for this user + goal_group
        println!("   Fetching bandit state...");
        let mut arms = user_repo.get_bandit_arms(user_id, goal_group).await?;

        // Initialize arms if empty
        if arms.is_empty() {
            println!("   No bandit state found - initializing all profiles...");
            arms = ProfileName::all()
                .iter()
                .map(|name| iqrah_core::scheduler_v2::BanditArmState {
                    profile_name: *name,
                    successes: 1.0,
                    failures: 1.0,
                })
                .collect();

            // Persist initial state
            for arm in &arms {
                user_repo
                    .update_bandit_arm(
                        user_id,
                        goal_group,
                        arm.profile_name.as_str(),
                        arm.successes,
                        arm.failures,
                    )
                    .await?;
            }
        }

        println!("   Loaded {} bandit arms", arms.len());

        // Use Thompson Sampling to choose profile
        let rng = StdRng::from_entropy();
        let mut optimizer = BanditOptimizer::new(rng);
        let chosen = optimizer.choose_arm(&arms);

        println!("   Thompson Sampling chose: {}", chosen.as_str());

        // Blend chosen profile with safe profile
        let blended = blend_profile(chosen);

        (blended, Some(chosen))
    } else {
        println!("   Using balanced profile (bandit disabled)");
        (UserProfile::balanced(), None)
    };

    // Generate session
    println!();
    println!(
        "   Generating session (size={}, mode={:?})...",
        session_size, session_mode
    );
    let session_node_ids = generate_session(
        candidates.clone(),
        parent_map.clone(),
        parent_energies.clone(),
        &profile,
        session_size,
        now_ts,
        session_mode,
    );

    // Display results
    println!();
    println!("✅ {}", "Session Generated!".green().bold());
    println!();
    println!(
        "   {}: {}",
        "Nodes in session".bright_white().bold(),
        session_node_ids.len().to_string().bright_cyan().bold()
    );
    if let Some(profile_name) = chosen_profile_name {
        println!(
            "   {}: {} (blended {:.0}%/{:.0}% with {})",
            "Bandit profile".bright_white().bold(),
            profile_name.as_str().bright_magenta().bold(),
            BLEND_RATIO * 100.0,
            (1.0 - BLEND_RATIO) * 100.0,
            DEFAULT_SAFE_PROFILE.as_str()
        );
    }
    println!();

    if verbose {
        // Display profile weights
        println!("   Profile Weights:");
        println!("      Urgency: {:.2}", profile.w_urgency);
        println!("      Readiness: {:.2}", profile.w_readiness);
        println!("      Foundation: {:.2}", profile.w_foundation);
        println!("      Influence: {:.2}", profile.w_influence);
        println!();

        // Display detailed table
        println!("   Session Details:");
        println!();
        println!(
            "   {:<15} {:>8} {:>8} {:>8} {:>8} {:>10}",
            "Node ID", "Found", "Infl", "Diff", "Energy", "Quran Order"
        );
        println!("   {}", "-".repeat(75));

        for node_id in &session_node_ids {
            if let Some(candidate) = candidates.iter().find(|c| c.id == *node_id) {
                println!(
                    "   {:<15} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>10}",
                    candidate.id,
                    candidate.foundational_score,
                    candidate.influence_score,
                    candidate.difficulty_score,
                    candidate.energy,
                    candidate.quran_order
                );
            }
        }
        println!();
    } else {
        // Simple list
        for node_id in &session_node_ids {
            println!("   - {}", node_id);
        }
        println!();
        println!("   (Use --verbose for detailed node information)");
    }

    Ok(())
}
