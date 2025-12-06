//! ISS CLI - Iqrah Student Simulation Command Line Interface
//!
//! Run simulations against the real Iqrah scheduler to evaluate efficiency.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use iqrah_core::ContentRepository;
use iqrah_iss::{
    run_comparison, Scenario, SchedulerVariant, SimulationConfig, SimulationMetrics, Simulator,
};
use iqrah_storage::{create_content_repository, init_content_db};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "iqrah-iss")]
#[command(about = "Iqrah Student Simulations - Scheduler evaluation framework")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to content.db
    #[arg(short, long, default_value = "content.db")]
    content_db: PathBuf,

    /// Verbosity level (0=error, 1=warn, 2=info, 3=debug, 4=trace)
    #[arg(short, long, default_value = "2")]
    verbosity: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single student simulation
    Single {
        /// Scenario name or preset (casual, dedicated, or custom YAML path)
        #[arg(short, long, default_value = "default")]
        scenario: String,

        /// Base RNG seed for reproducibility
        #[arg(short = 'S', long, default_value = "42")]
        seed: u64,

        /// Number of days to simulate
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Goal ID (e.g., "surah:1", "juz:30")
        #[arg(short, long, default_value = "surah:1")]
        goal: String,
    },

    /// Run batch simulation with multiple students
    Batch {
        /// Scenario name or preset
        #[arg(short, long, default_value = "default")]
        scenario: String,

        /// Number of students to simulate
        #[arg(short, long, default_value = "100")]
        count: usize,

        /// Base RNG seed for reproducibility
        #[arg(short = 'S', long, default_value = "42")]
        seed: u64,

        /// Number of days to simulate
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Goal ID
        #[arg(short, long, default_value = "surah:1")]
        goal: String,

        /// Output JSON file for results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a sample configuration file
    GenConfig {
        /// Output YAML file
        #[arg(short, long, default_value = "iss_config.yaml")]
        output: PathBuf,
    },

    /// Compare multiple scheduler variants
    Compare {
        /// Scenario name or preset
        #[arg(short, long, default_value = "default")]
        scenario: String,

        /// Scheduler variants to compare (comma-separated)
        /// Options: iqrah_default, page_order, fixed_srs, random
        #[arg(
            short = 'V',
            long,
            value_delimiter = ',',
            default_value = "iqrah_default,random"
        )]
        variants: Vec<String>,

        /// Number of students per variant
        #[arg(short = 'n', long, default_value = "50")]
        students: usize,

        /// Base RNG seed for reproducibility
        #[arg(short = 'S', long, default_value = "42")]
        seed: u64,

        /// Number of days to simulate
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Goal ID
        #[arg(short, long, default_value = "surah:1")]
        goal: String,

        /// Output JSON file for results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include individual student metrics in output
        #[arg(long)]
        include_individual: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = match cli.verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // Load content database
    info!("Loading content database from {:?}", cli.content_db);
    let db_path = cli.content_db.to_str().context("Invalid database path")?;
    let pool = init_content_db(db_path)
        .await
        .context("Failed to open content.db")?;
    let content_repo: Arc<dyn ContentRepository> = Arc::new(create_content_repository(pool));

    match cli.command {
        Commands::Single {
            scenario,
            seed,
            days,
            goal,
        } => {
            run_single(content_repo, &scenario, seed, days, &goal).await?;
        }
        Commands::Batch {
            scenario,
            count,
            seed,
            days,
            goal,
            output,
        } => {
            run_batch(content_repo, &scenario, count, seed, days, &goal, output).await?;
        }
        Commands::GenConfig { output } => {
            generate_config(&output)?;
        }
        Commands::Compare {
            scenario,
            variants,
            students,
            seed,
            days,
            goal,
            output,
            include_individual,
        } => {
            run_compare(
                content_repo,
                &scenario,
                &variants,
                students,
                seed,
                days,
                &goal,
                output,
                include_individual,
            )
            .await?;
        }
    }

    Ok(())
}

async fn run_single(
    content_repo: Arc<dyn ContentRepository>,
    scenario_name: &str,
    seed: u64,
    days: u32,
    goal: &str,
) -> Result<()> {
    let scenario = create_scenario(scenario_name, days, goal);
    let config = SimulationConfig {
        scenarios: vec![scenario.clone()],
        base_seed: seed,
        ..Default::default()
    };

    let simulator = Simulator::new(content_repo, config.clone());

    info!("Running single student simulation");
    info!("  Scenario: {}", scenario.name);
    info!("  Goal: {}", goal);
    info!("  Days: {}", days);
    info!("  Seed: {}", seed);

    let metrics = simulator
        .simulate_student(&scenario, 0)
        .await
        .context("Simulation failed")?;

    print_metrics(&metrics, &config);

    Ok(())
}

async fn run_batch(
    content_repo: Arc<dyn ContentRepository>,
    scenario_name: &str,
    count: usize,
    seed: u64,
    days: u32,
    goal: &str,
    output: Option<PathBuf>,
) -> Result<()> {
    let scenario = create_scenario(scenario_name, days, goal);
    let config = SimulationConfig {
        scenarios: vec![scenario.clone()],
        base_seed: seed,
        ..Default::default()
    };

    let simulator = Simulator::new(content_repo, config.clone());

    info!("Running batch simulation");
    info!("  Scenario: {}", scenario.name);
    info!("  Students: {}", count);
    info!("  Goal: {}", goal);
    info!("  Days: {}", days);
    info!("  Seed: {}", seed);

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let mut all_metrics: Vec<SimulationMetrics> = Vec::with_capacity(count);

    for i in 0..count {
        let metrics = simulator
            .simulate_student(&scenario, i)
            .await
            .context(format!("Simulation failed for student {}", i))?;
        all_metrics.push(metrics);
        pb.inc(1);
    }

    pb.finish_with_message("Simulation complete");

    // Compute aggregate statistics
    print_batch_summary(&all_metrics, &config);

    // Save to JSON if output specified
    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&BatchResults {
            scenario: scenario_name.to_string(),
            student_count: count,
            seed,
            metrics: all_metrics
                .iter()
                .map(|m| MetricsSummary::from(m))
                .collect(),
        })?;
        std::fs::write(&output_path, json)?;
        info!("Results saved to {:?}", output_path);
    }

    Ok(())
}

fn create_scenario(name: &str, days: u32, goal: &str) -> Scenario {
    let mut scenario = match name {
        "casual" => Scenario::casual_learner(),
        "dedicated" => Scenario::dedicated_student(),
        _ => Scenario::default(),
    };
    scenario.target_days = days;
    scenario.goal_id = goal.to_string();
    scenario
}

fn print_metrics(metrics: &SimulationMetrics, config: &SimulationConfig) {
    println!("\n=== Simulation Results ===");
    println!("Days completed:        {}", metrics.total_days);
    println!("Total minutes:         {:.1}", metrics.total_minutes);
    println!(
        "Items mastered:        {}/{}",
        metrics.items_mastered, metrics.goal_item_count
    );
    println!(
        "Coverage:              {:.1}%",
        metrics.coverage_pct * 100.0
    );
    println!("Retention/minute:      {:.4}", metrics.retention_per_minute);
    println!(
        "Plan faithfulness:     {:.1}%",
        metrics.plan_faithfulness * 100.0
    );
    println!("Gave up:               {}", metrics.gave_up);

    if let Some(days) = metrics.days_to_mastery {
        println!("Days to mastery:       {}", days);
    } else {
        println!("Days to mastery:       Not reached");
    }

    let score = metrics.final_score(config.scenarios[0].target_days, config.expected_rpm);
    println!("\nFinal Score:           {:.3}", score);
}

fn print_batch_summary(all_metrics: &[SimulationMetrics], config: &SimulationConfig) {
    let n = all_metrics.len() as f64;

    let avg_coverage: f64 = all_metrics.iter().map(|m| m.coverage_pct).sum::<f64>() / n;
    let avg_rpm: f64 = all_metrics
        .iter()
        .map(|m| m.retention_per_minute)
        .sum::<f64>()
        / n;
    let avg_minutes: f64 = all_metrics.iter().map(|m| m.total_minutes).sum::<f64>() / n;
    let gave_up_count = all_metrics.iter().filter(|m| m.gave_up).count();

    let avg_score: f64 = all_metrics
        .iter()
        .map(|m| m.final_score(config.scenarios[0].target_days, config.expected_rpm))
        .sum::<f64>()
        / n;

    println!("\n=== Batch Simulation Summary ===");
    println!("Students simulated:    {}", all_metrics.len());
    println!("Average coverage:      {:.1}%", avg_coverage * 100.0);
    println!("Average RPM:           {:.4}", avg_rpm);
    println!("Average minutes:       {:.1}", avg_minutes);
    println!(
        "Students gave up:      {} ({:.1}%)",
        gave_up_count,
        gave_up_count as f64 / n * 100.0
    );
    println!("\nAverage Final Score:   {:.3}", avg_score);
}

fn generate_config(output: &PathBuf) -> Result<()> {
    let config = SimulationConfig::default();
    config.save(output)?;
    info!("Generated sample config at {:?}", output);
    println!("Sample configuration saved to {:?}", output);
    Ok(())
}

#[derive(serde::Serialize)]
struct BatchResults {
    scenario: String,
    student_count: usize,
    seed: u64,
    metrics: Vec<MetricsSummary>,
}

#[derive(serde::Serialize)]
struct MetricsSummary {
    total_days: u32,
    total_minutes: f64,
    items_mastered: usize,
    goal_item_count: usize,
    coverage_pct: f64,
    retention_per_minute: f64,
    plan_faithfulness: f64,
    gave_up: bool,
    days_to_mastery: Option<u32>,
}

impl From<&SimulationMetrics> for MetricsSummary {
    fn from(m: &SimulationMetrics) -> Self {
        Self {
            total_days: m.total_days,
            total_minutes: m.total_minutes,
            items_mastered: m.items_mastered,
            goal_item_count: m.goal_item_count,
            coverage_pct: m.coverage_pct,
            retention_per_minute: m.retention_per_minute,
            plan_faithfulness: m.plan_faithfulness,
            gave_up: m.gave_up,
            days_to_mastery: m.days_to_mastery,
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_compare(
    content_repo: Arc<dyn ContentRepository>,
    scenario_name: &str,
    variant_names: &[String],
    students: usize,
    seed: u64,
    days: u32,
    goal: &str,
    output: Option<PathBuf>,
    include_individual: bool,
) -> Result<()> {
    // Parse variant names
    let variants: Vec<SchedulerVariant> = variant_names
        .iter()
        .filter_map(|name| match SchedulerVariant::from_str(name) {
            Some(v) => Some(v),
            None => {
                warn!("Unknown scheduler variant: '{}', skipping", name);
                None
            }
        })
        .collect();

    if variants.is_empty() {
        return Err(anyhow::anyhow!("No valid scheduler variants specified"));
    }

    // Create base scenario
    let base_scenario = create_scenario(scenario_name, days, goal);

    info!("Running comparison:");
    info!("  Variants: {:?}", variant_names);
    info!("  Students per variant: {}", students);
    info!("  Scenario: {}", base_scenario.name);
    info!("  Goal: {}", goal);
    info!("  Days: {}", days);
    info!("  Seed: {}", seed);

    println!("\n=== Scheduler Comparison ===");
    println!(
        "Variants: {:?}",
        variants.iter().map(|v| v.name()).collect::<Vec<_>>()
    );
    println!("Students per variant: {}", students);
    println!("Goal: {}", goal);
    println!("Days: {}", days);
    println!();

    // Run comparison
    let results = run_comparison(
        content_repo,
        &base_scenario,
        &variants,
        students,
        seed,
        0.1, // expected_rpm
        include_individual,
    )
    .await?;

    // Print results
    println!("\n=== Results ===\n");
    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Variant", "Score", "Coverage%", "RPM", "GaveUp%", "Mastery"
    );
    println!("{}", "-".repeat(72));

    for variant in &results.variants {
        let m = &variant.metrics;
        let mastery_str = m
            .days_to_mastery_mean
            .map(|d| format!("{:.1}d", d))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<20} {:>10.3} {:>10.1} {:>10.4} {:>10.1} {:>10}",
            variant.variant,
            m.final_score_mean,
            m.coverage_pct_mean * 100.0,
            m.retention_per_minute_mean,
            m.gave_up_fraction * 100.0,
            mastery_str
        );
    }
    println!("{}", "-".repeat(72));

    // Save to file if requested
    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(&output_path, json)?;
        println!("\nResults saved to {:?}", output_path);
    }

    Ok(())
}
