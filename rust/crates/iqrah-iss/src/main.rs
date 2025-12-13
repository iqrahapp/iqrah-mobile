//! ISS CLI - Iqrah Student Simulation Command Line Interface
//!
//! Run simulations against the real Iqrah scheduler to evaluate efficiency.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use iqrah_core::ContentRepository;
use iqrah_iss::{
    run_comparison, Scenario, SchedulerVariant, SimulationConfig, SimulationMetrics, Simulator,
    StudentProfile,
};
use iqrah_storage::{create_content_repository, open_content_db_readonly};
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
        #[arg(short, long)]
        days: Option<u32>,

        /// Goal ID (e.g., "surah:1", "juz:30")
        #[arg(short, long)]
        goal: Option<String>,
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
        #[arg(short, long)]
        days: Option<u32>,

        /// Goal ID
        #[arg(short, long)]
        goal: Option<String>,

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

    /// List available scenario presets
    ListPresets,

    /// Compare multiple scheduler variants
    Compare {
        /// Scenario name or preset (alternative to --scenario)
        /// Built-in: juz_amma_casual, juz_amma_dedicated, etc.
        #[arg(long)]
        preset: Option<String>,

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
        #[arg(short, long)]
        days: Option<u32>,

        /// Goal ID
        #[arg(short, long)]
        goal: Option<String>,

        /// Output JSON file for results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include individual student metrics in output
        #[arg(long)]
        include_individual: bool,

        /// Student profile to use (strong_dedicated, normal_dedicated, harsh_stress_test)
        #[arg(long)]
        student_profile: Option<String>,

        /// Output directory for event tracking files (JSONL + analysis markdown)
        #[arg(long)]
        emit_events: Option<PathBuf>,

        /// M1.2: Enable gate trace output (CSV + markdown summary)
        #[arg(long)]
        trace: bool,

        /// M1.2: Output directory for gate trace files (default: ./trace_output)
        #[arg(long, default_value = "./trace_output")]
        trace_dir: PathBuf,
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

    // Load content database (read-only, no migrations)
    info!("Loading content database from {:?}", cli.content_db);
    let db_path = cli.content_db.to_str().context("Invalid database path")?;
    let pool = open_content_db_readonly(db_path)
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
            run_single(content_repo, &scenario, seed, days, goal.as_deref()).await?;
        }
        Commands::Batch {
            scenario,
            count,
            seed,
            days,
            goal,
            output,
        } => {
            run_batch(
                content_repo,
                &scenario,
                count,
                seed,
                days,
                goal.as_deref(),
                output,
            )
            .await?;
        }
        Commands::GenConfig { output } => {
            generate_config(&output)?;
        }
        Commands::ListPresets => {
            list_presets();
        }
        Commands::Compare {
            preset,
            scenario,
            variants,
            students,
            seed,
            days,
            goal,
            output,
            include_individual,
            student_profile,
            emit_events,
            trace,
            trace_dir,
        } => {
            // Determine which scenario to use
            let scenario_name = preset.as_ref().unwrap_or(&scenario);

            // Parse student profile if provided
            let profile = student_profile
                .as_ref()
                .and_then(|s| StudentProfile::from_str(s));
            if student_profile.is_some() && profile.is_none() {
                warn!(
                    "Unknown student profile '{}', using default",
                    student_profile.as_ref().unwrap()
                );
            }

            run_compare(
                content_repo,
                scenario_name,
                &variants,
                students,
                seed,
                days,
                goal.as_deref(),
                output,
                include_individual,
                profile,
                emit_events,
                trace,
                trace_dir,
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
    days: Option<u32>,
    goal: Option<&str>,
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
    info!("  Goal: {}", scenario.goal_id);
    info!("  Days: {}", scenario.target_days);
    info!("  Seed: {}", seed);

    let (metrics, _debug, _sanity) = simulator
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
    days: Option<u32>,
    goal: Option<&str>,
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
    info!("  Goal: {}", scenario.goal_id);
    info!("  Days: {}", scenario.target_days);
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
        let (metrics, _debug, _sanity) = simulator
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

fn create_scenario(name: &str, days: Option<u32>, goal: Option<&str>) -> Scenario {
    // First try to load from YAML preset file
    let yaml_path = format!("crates/iqrah-iss/configs/scenarios/{}.yaml", name);
    if let Ok(contents) = std::fs::read_to_string(&yaml_path) {
        if let Ok(mut scenario) = serde_yaml::from_str::<Scenario>(&contents) {
            // CLI arguments override YAML only if provided
            if let Some(d) = days {
                scenario.target_days = d;
            }
            if let Some(g) = goal {
                scenario.goal_id = g.to_string();
            }
            info!(
                "Loaded scenario '{}' from {} (goal={}, days={})",
                scenario.name, yaml_path, scenario.goal_id, scenario.target_days
            );
            return scenario;
        }
    }

    // Fall back to built-in archetypes
    let mut scenario = match name {
        "casual" | "juz_amma_casual" => Scenario::casual_learner(),
        "dedicated" | "juz_amma_dedicated" => Scenario::dedicated_student(),
        "surah_fatiha_dedicated" => {
            let mut s = Scenario::dedicated_student();
            s.goal_id = "surah:1".to_string();
            // Fatiha is short, shorten target days
            s.target_days = 30;
            s
        }
        _ => Scenario::default(),
    };
    scenario.target_days = days.unwrap_or(30);
    scenario.goal_id = goal.unwrap_or("surah:1").to_string();
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

fn list_presets() {
    println!("=== Available Scenario Presets ===\n");
    println!("Use with: --preset <name> or load YAML from configs/scenarios/\n");

    println!("Built-in archetypes:");
    println!("  casual        - Casual learner (high skip prob, shorter sessions)");
    println!("  dedicated     - Dedicated student (low skip prob, longer sessions)");
    println!("  default       - Default balanced params");
    println!();

    println!("YAML preset files (in configs/scenarios/):");
    println!("  juz_amma_casual.yaml        - Juz 30 / 1 year / casual learner");
    println!("  juz_amma_dedicated.yaml     - Juz 30 / 6 months / dedicated student");
    println!("  surah_baqarah_dedicated.yaml - Surah 2 / 1 year / dedicated");
    println!("  heterogeneous_juz30.yaml    - Heterogeneous population example");
    println!();

    println!("To use a preset YAML file:");
    println!("  iqrah-iss batch --scenario configs/scenarios/juz_amma_casual.yaml");
    println!("  iqrah-iss compare --preset juz_amma_dedicated -V iqrah_default,random");
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
    coverage_acq: f64,
    mean_r_acq: f64,
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
            coverage_acq: m.coverage_acq,
            mean_r_acq: m.mean_r_acq,
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
    days: Option<u32>,
    goal: Option<&str>,
    output: Option<PathBuf>,
    include_individual: bool,
    profile: Option<StudentProfile>,
    emit_events: Option<PathBuf>,
    trace_enabled: bool,
    trace_dir: PathBuf,
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
    let mut base_scenario = create_scenario(scenario_name, days, goal);

    // Apply student profile if specified
    if let Some(p) = profile {
        base_scenario.student_profile = Some(p);
        info!("Using student profile: {}", p.name());
    }

    // Log event tracking status
    if let Some(ref event_dir) = emit_events {
        info!("Event tracking enabled, output dir: {:?}", event_dir);
        // Create directory if it doesn't exist
        std::fs::create_dir_all(event_dir)?;
    }

    info!("Running comparison:");
    info!("  Variants: {:?}", variant_names);
    info!("  Students per variant: {}", students);
    info!("  Scenario: {}", base_scenario.name);
    info!("  Goal: {}", base_scenario.goal_id);
    info!("  Days: {}", base_scenario.target_days);
    info!("  Seed: {}", seed);
    if let Some(ref p) = base_scenario.student_profile {
        info!("  Student Profile: {}", p.name());
    }

    println!("\n=== Scheduler Comparison ===");
    println!(
        "Variants: {:?}",
        variants.iter().map(|v| v.name()).collect::<Vec<_>>()
    );
    println!("Students per variant: {}", students);
    println!("Goal: {}", base_scenario.goal_id);
    println!("Days: {}", base_scenario.target_days);
    if let Some(ref p) = base_scenario.student_profile {
        println!("Profile: {}", p.name());
    }
    println!();

    // M1.2: Configure gate trace if enabled
    let trace_config = if trace_enabled {
        info!("Gate trace enabled, output dir: {:?}", trace_dir);
        std::fs::create_dir_all(&trace_dir)?;
        iqrah_iss::config::DebugTraceConfig {
            enabled: true,
            out_dir: trace_dir.to_string_lossy().to_string(),
        }
    } else {
        iqrah_iss::config::DebugTraceConfig::default()
    };

    // Run comparison
    let (results, debug_report) = run_comparison(
        Arc::clone(&content_repo),
        &base_scenario,
        &variants,
        students,
        seed,
        0.1, // expected_rpm
        include_individual,
        trace_config,
    )
    .await?;

    // Print results
    println!("\n=== Results ===\n");
    println!(
        "{:<20} {:>8} {:>10} {:>10} {:>10} {:>10} | {:>10} {:>8} {:>10} | {:>10} {:>8}",
        "Variant",
        "Score",
        "Coverage%",
        "RPM",
        "GaveUp%",
        "Mastery",
        "Cov(T)",
        "R(T)",
        "RPM(T)",
        "Cov(Acq)",
        "R(Acq)"
    );
    println!("{}", "-".repeat(135));

    for variant in &results.variants {
        let m = &variant.metrics;
        let mastery_str = m
            .days_to_mastery_mean
            .map(|d| format!("{:.1}d", d))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<20} {:>8.3} {:>10.1} {:>10.4} {:>10.1} {:>10} | {:>10.1} {:>8.2} {:>10.4} | {:>10.1} {:>8.2}",
            variant.variant,
            m.final_score_mean,
            m.coverage_pct_mean * 100.0,          // = mean_retrievability * 100
            m.retention_per_minute_mean,
            m.gave_up_fraction * 100.0,
            mastery_str,
            m.coverage_h_0_9_mean * 100.0,        // M1: coverage_h@0.9
            m.mean_r_t_mean,                       // M1: mean_retrievability
            m.rpm_t_mean,
            m.coverage_acq_mean * 100.0,
            m.mean_r_acq_mean
        );
    }
    println!("{}", "-".repeat(72));

    // Save to file if requested
    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(&output_path, json)?;
        println!("\nResults saved to {:?}", output_path);

        if let Some(report) = debug_report {
            let debug_path = if let Some(stem) = output_path.file_stem() {
                let mut p = output_path.clone();
                p.set_file_name(format!("{}_debug.json", stem.to_string_lossy()));
                p
            } else {
                output_path.with_extension("debug.json")
            };

            let debug_json = serde_json::to_string_pretty(&report)?;
            std::fs::write(&debug_path, debug_json)?;
            println!("Debug report saved to {:?}", debug_path);
        }
    }

    // Event tracking: Run one student per variant with event collection
    if let Some(ref event_dir) = emit_events {
        use iqrah_iss::{event_channel, write_events_jsonl, EventAnalyzer, Simulator};

        println!("\n=== Collecting Events (student 0 per variant) ===");

        for &variant in &variants {
            let scenario = base_scenario.with_scheduler(variant);
            let mut config = SimulationConfig {
                scenarios: vec![scenario.clone()],
                base_seed: seed,
                event_log_enabled: true,
                ..Default::default()
            };
            config.event_log_enabled = true;

            // Create event channel
            let (event_sender, event_receiver) = event_channel(true);

            // Create simulator with external event sender
            let sim = Simulator::with_event_sender(Arc::clone(&content_repo), config, event_sender);

            // Run student 0
            let _ = sim.simulate_student(&scenario, 0).await;

            // Collect events
            let events = event_receiver.collect();
            println!("  {}: {} events collected", variant.name(), events.len());

            // Write JSONL
            let events_path = event_dir.join(format!("{}_events.jsonl", variant.name()));
            if let Err(e) = write_events_jsonl(&events, &events_path) {
                warn!("Failed to write events for {}: {}", variant.name(), e);
            } else {
                println!("    -> {:?}", events_path);
            }

            // Generate analysis report
            let analyzer = EventAnalyzer::from_events(events);
            let report_path = event_dir.join(format!("{}_analysis.md", variant.name()));
            if let Err(e) = analyzer.write_report(&report_path, variant.name()) {
                warn!("Failed to write analysis for {}: {}", variant.name(), e);
            } else {
                println!("    -> {:?}", report_path);
            }
        }

        println!("\nEvent tracking complete. Files saved to {:?}", event_dir);
    }

    Ok(())
}
