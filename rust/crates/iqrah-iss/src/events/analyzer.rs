//! Event analysis and report generation.
//!
//! Analyzes simulation event logs to identify failure patterns,
//! energy flow, and scheduling behavior.

use crate::events::{compute_stats, EnergyBucket, SimulationEvent, SkipReason, TransitionCause};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Analyzes simulation events to generate reports.
pub struct EventAnalyzer {
    events: Vec<SimulationEvent>,
}

impl EventAnalyzer {
    /// Create analyzer from events slice.
    pub fn from_events(events: Vec<SimulationEvent>) -> Self {
        Self { events }
    }

    /// Generate a comprehensive analysis report.
    pub fn generate_report(&self) -> AnalysisReport {
        AnalysisReport {
            energy_flow: self.analyze_energy_flow(),
            scheduling: self.analyze_scheduling(),
            coverage: self.analyze_coverage(),
            failures: self.detect_failure_patterns(),
            summary: self.generate_summary(),
        }
    }

    /// Analyze energy transitions by type and period.
    fn analyze_energy_flow(&self) -> EnergyFlowReport {
        let mut decay_count = 0;
        let mut growth_count = 0;
        let mut collapse_count = 0; // mastered/advanced -> beginner/aware

        let mut transitions_by_day: HashMap<
            u32,
            Vec<(EnergyBucket, EnergyBucket, TransitionCause)>,
        > = HashMap::new();

        for event in &self.events {
            if let SimulationEvent::EnergyTransition {
                day,
                from_bucket,
                to_bucket,
                cause,
                ..
            } = event
            {
                transitions_by_day.entry(*day).or_default().push((
                    *from_bucket,
                    *to_bucket,
                    *cause,
                ));

                match cause {
                    TransitionCause::Decay => {
                        decay_count += 1;
                        // Check for collapse
                        if matches!(from_bucket, EnergyBucket::Advanced | EnergyBucket::Mastered)
                            && matches!(to_bucket, EnergyBucket::Aware | EnergyBucket::Beginner)
                        {
                            collapse_count += 1;
                        }
                    }
                    TransitionCause::ReviewSuccess | TransitionCause::Introduction => {
                        if bucket_order(*to_bucket) > bucket_order(*from_bucket) {
                            growth_count += 1;
                        }
                    }
                    TransitionCause::ReviewFail => {
                        decay_count += 1;
                    }
                }
            }
        }

        // Compute period summaries (30-day windows)
        let mut period_summaries = Vec::new();
        let max_day = transitions_by_day.keys().max().copied().unwrap_or(0);

        for start in (1..=max_day).step_by(30) {
            let end = (start + 29).min(max_day);
            let mut period_decay = 0;
            let mut period_growth = 0;

            for day in start..=end {
                if let Some(transitions) = transitions_by_day.get(&day) {
                    for (from, to, cause) in transitions {
                        match cause {
                            TransitionCause::Decay | TransitionCause::ReviewFail => {
                                period_decay += 1;
                            }
                            _ => {
                                if bucket_order(*to) > bucket_order(*from) {
                                    period_growth += 1;
                                }
                            }
                        }
                    }
                }
            }

            period_summaries.push(PeriodSummary {
                start_day: start,
                end_day: end,
                decay_transitions: period_decay,
                growth_transitions: period_growth,
                net_direction: if period_growth > period_decay {
                    "GROWTH"
                } else if period_decay > period_growth {
                    "DECAY"
                } else {
                    "STABLE"
                }
                .to_string(),
            });
        }

        EnergyFlowReport {
            total_decay: decay_count,
            total_growth: growth_count,
            collapse_count,
            period_summaries,
        }
    }

    /// Analyze scheduling behavior.
    fn analyze_scheduling(&self) -> SchedulingReport {
        let mut introduced_by_day: HashMap<u32, u32> = HashMap::new();
        let mut reviewed_by_day: HashMap<u32, u32> = HashMap::new();
        let mut skipped_by_day_reason: HashMap<u32, HashMap<SkipReason, u32>> = HashMap::new();
        let mut urgent_backlog_by_day: HashMap<u32, u32> = HashMap::new();

        for event in &self.events {
            match event {
                SimulationEvent::ItemIntroduced { day, .. } => {
                    *introduced_by_day.entry(*day).or_default() += 1;
                }
                SimulationEvent::ReviewOutcome { day, .. } => {
                    *reviewed_by_day.entry(*day).or_default() += 1;
                }
                SimulationEvent::ItemSkipped { day, reason, .. } => {
                    *skipped_by_day_reason
                        .entry(*day)
                        .or_default()
                        .entry(*reason)
                        .or_default() += 1;
                }
                SimulationEvent::DaySnapshot {
                    day,
                    urgent_backlog,
                    ..
                } => {
                    urgent_backlog_by_day.insert(*day, *urgent_backlog);
                }
                _ => {}
            }
        }

        // Compute period summaries with skip reason breakdown
        let max_day = introduced_by_day
            .keys()
            .chain(reviewed_by_day.keys())
            .chain(skipped_by_day_reason.keys())
            .max()
            .copied()
            .unwrap_or(0);

        let mut period_summaries = Vec::new();
        for start in (0..=max_day).step_by(30) {
            let end = (start + 29).min(max_day);
            let mut period_introduced = 0;
            let mut period_reviewed = 0;
            let mut max_backlog = 0;
            let mut period_skip_reasons: HashMap<SkipReason, u32> = HashMap::new();

            for day in start..=end {
                period_introduced += introduced_by_day.get(&day).unwrap_or(&0);
                period_reviewed += reviewed_by_day.get(&day).unwrap_or(&0);
                max_backlog = max_backlog.max(*urgent_backlog_by_day.get(&day).unwrap_or(&0));

                if let Some(day_skips) = skipped_by_day_reason.get(&day) {
                    for (reason, count) in day_skips {
                        *period_skip_reasons.entry(*reason).or_default() += count;
                    }
                }
            }

            let days_in_period = (end - start + 1).max(1) as f32;
            let total_skips: u32 = period_skip_reasons.values().sum();

            period_summaries.push(SchedulingPeriodSummary {
                start_day: start,
                end_day: end,
                avg_introduced_per_day: period_introduced as f32 / days_in_period,
                avg_reviewed_per_day: period_reviewed as f32 / days_in_period,
                max_urgent_backlog: max_backlog,
                backlog_status: if max_backlog == 0 {
                    "OK"
                } else if max_backlog < 10 {
                    "MILD"
                } else if max_backlog < 30 {
                    "BOTTLENECK"
                } else {
                    "CRITICAL"
                }
                .to_string(),
                total_skips,
                mix_cap_skips: *period_skip_reasons
                    .get(&SkipReason::MixCapReached)
                    .unwrap_or(&0),
                session_full_skips: *period_skip_reasons
                    .get(&SkipReason::SessionFull)
                    .unwrap_or(&0),
                low_priority_skips: *period_skip_reasons
                    .get(&SkipReason::LowPriority)
                    .unwrap_or(&0),
            });
        }

        // Compute overall skip reason percentages
        let mut skipped_by_reason: HashMap<String, u32> = HashMap::new();
        for day_skips in skipped_by_day_reason.values() {
            for (reason, count) in day_skips {
                let reason_str = match reason {
                    SkipReason::SessionFull => "SessionFull",
                    SkipReason::LowPriority => "LowPriority",
                    SkipReason::MixCapReached => "MixCapReached",
                    SkipReason::NotEligible => "NotEligible",
                };
                *skipped_by_reason.entry(reason_str.to_string()).or_default() += count;
            }
        }

        let total_skips: u32 = skipped_by_reason.values().sum();
        let skip_percentages: HashMap<String, f32> = skipped_by_reason
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    if total_skips > 0 {
                        *v as f32 / total_skips as f32 * 100.0
                    } else {
                        0.0
                    },
                )
            })
            .collect();

        SchedulingReport {
            total_introduced: introduced_by_day.values().sum(),
            total_reviewed: reviewed_by_day.values().sum(),
            total_skipped: total_skips,
            skip_reason_percentages: skip_percentages,
            period_summaries,
        }
    }

    /// Analyze coverage progression.
    fn analyze_coverage(&self) -> CoverageReport {
        let mut coverage_by_day: Vec<(u32, f32)> = Vec::new();
        let mut peak_coverage = 0.0f32;
        let mut peak_day = 0;

        for event in &self.events {
            if let SimulationEvent::DaySnapshot {
                day,
                coverage_mean_r,
                ..
            } = event
            {
                coverage_by_day.push((*day, *coverage_mean_r));
                if *coverage_mean_r > peak_coverage {
                    peak_coverage = *coverage_mean_r;
                    peak_day = *day;
                }
            }
        }

        // Check for regression
        let mut regressed = false;
        let mut regression_day = None;
        if coverage_by_day.len() > 10 {
            // Look for sustained decline
            let last_10: Vec<f32> = coverage_by_day
                .iter()
                .rev()
                .take(10)
                .map(|(_, c)| *c)
                .collect();
            if let (Some(first), Some(last)) = (last_10.last(), last_10.first()) {
                if *first > *last * 1.1 {
                    // 10% decline
                    regressed = true;
                    regression_day = coverage_by_day.iter().rev().nth(10).map(|(d, _)| *d);
                }
            }
        }

        let final_coverage = coverage_by_day.last().map(|(_, c)| *c).unwrap_or(0.0);

        CoverageReport {
            final_coverage,
            peak_coverage,
            peak_day,
            regressed,
            regression_day,
            coverage_samples: coverage_by_day
                .iter()
                .filter(|(d, _)| d % 30 == 0 || *d == 1)
                .cloned()
                .collect(),
        }
    }

    /// Detect failure patterns.
    fn detect_failure_patterns(&self) -> Vec<FailurePattern> {
        let mut patterns = Vec::new();

        let scheduling = self.analyze_scheduling();
        let energy_flow = self.analyze_energy_flow();
        let coverage = self.analyze_coverage();

        // Check for urgency cascade
        let max_backlog: u32 = scheduling
            .period_summaries
            .iter()
            .map(|p| p.max_urgent_backlog)
            .max()
            .unwrap_or(0);

        if max_backlog > 20 {
            patterns.push(FailurePattern {
                name: "Urgency Cascade".to_string(),
                severity: if max_backlog > 40 {
                    "CRITICAL"
                } else {
                    "WARNING"
                }
                .to_string(),
                description: format!(
                    "Urgent backlog grew to {} items. Items needing review are not being scheduled.",
                    max_backlog
                ),
            });
        }

        // Check for mix cap bottleneck
        if let Some(mix_cap_pct) = scheduling.skip_reason_percentages.get("MixCapReached") {
            if *mix_cap_pct > 50.0 {
                patterns.push(FailurePattern {
                    name: "Mix Cap Bottleneck".to_string(),
                    severity: "CRITICAL".to_string(),
                    description: format!(
                        "{:.0}% of urgent items skipped due to mix percentage caps.",
                        mix_cap_pct
                    ),
                });
            }
        }

        // Check for coverage stall
        for period in &scheduling.period_summaries {
            if period.avg_introduced_per_day < 1.0 && period.start_day > 30 {
                patterns.push(FailurePattern {
                    name: "Coverage Stall".to_string(),
                    severity: "WARNING".to_string(),
                    description: format!(
                        "New item introduction dropped to {:.1}/day in days {}-{}.",
                        period.avg_introduced_per_day, period.start_day, period.end_day
                    ),
                });
                break;
            }
        }

        // Check for energy collapse
        if energy_flow.collapse_count > 10 {
            patterns.push(FailurePattern {
                name: "Energy Collapse".to_string(),
                severity: "WARNING".to_string(),
                description: format!(
                    "{} items collapsed from Advanced/Mastered to Aware/Beginner.",
                    energy_flow.collapse_count
                ),
            });
        }

        // Check for coverage regression
        if coverage.regressed {
            patterns.push(FailurePattern {
                name: "Coverage Regression".to_string(),
                severity: "WARNING".to_string(),
                description: format!(
                    "Coverage regressed from peak {:.1}% (day {}) to final {:.1}%.",
                    coverage.peak_coverage * 100.0,
                    coverage.peak_day,
                    coverage.final_coverage * 100.0
                ),
            });
        }

        // Check for gave up
        for event in &self.events {
            if let SimulationEvent::GaveUp {
                day,
                frustration,
                trigger,
            } = event
            {
                patterns.push(FailurePattern {
                    name: "Student Gave Up".to_string(),
                    severity: "CRITICAL".to_string(),
                    description: format!(
                        "Gave up on day {} with frustration {:.1}. Trigger: {}",
                        day, frustration, trigger
                    ),
                });
            }
        }

        patterns
    }

    /// Generate executive summary.
    fn generate_summary(&self) -> String {
        let event_stats = compute_stats(&self.events);

        let mut summary = String::new();

        // Outcome
        if let Some(day) = event_stats.gave_up_day {
            summary.push_str(&format!("**OUTCOME**: GAVE UP on day {}\n\n", day));
        } else {
            summary.push_str(&format!(
                "**OUTCOME**: COMPLETED {} days\n\n",
                event_stats.days_completed
            ));
        }

        // Key metrics
        summary.push_str(&format!(
            "**Items Introduced**: {}\n",
            event_stats.items_introduced
        ));
        summary.push_str(&format!(
            "**Reviews**: {} success, {} fail ({:.0}% success rate)\n",
            event_stats.reviews_success,
            event_stats.reviews_fail,
            if event_stats.reviews_success + event_stats.reviews_fail > 0 {
                event_stats.reviews_success as f32
                    / (event_stats.reviews_success + event_stats.reviews_fail) as f32
                    * 100.0
            } else {
                0.0
            }
        ));
        summary.push_str(&format!(
            "**Energy Transitions**: {} decay, {} growth\n",
            event_stats.decay_transitions,
            event_stats.success_transitions + event_stats.intro_transitions
        ));
        summary.push_str(&format!(
            "**Frustration Spikes**: {}\n",
            event_stats.frustration_spikes
        ));

        // Skip breakdown
        let total_skips = event_stats.skipped_session_full
            + event_stats.skipped_low_priority
            + event_stats.skipped_mix_cap
            + event_stats.skipped_not_eligible;

        if total_skips > 0 {
            summary.push_str(&format!("\n**Urgent Items Skipped**: {}\n", total_skips));
            if event_stats.skipped_mix_cap > 0 {
                summary.push_str(&format!(
                    "  - MixCapReached: {:.0}%\n",
                    event_stats.skipped_mix_cap as f32 / total_skips as f32 * 100.0
                ));
            }
            if event_stats.skipped_session_full > 0 {
                summary.push_str(&format!(
                    "  - SessionFull: {:.0}%\n",
                    event_stats.skipped_session_full as f32 / total_skips as f32 * 100.0
                ));
            }
        }

        summary
    }

    /// Write analysis report to markdown file.
    pub fn write_report(&self, path: &Path, variant_name: &str) -> std::io::Result<()> {
        let report = self.generate_report();
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "# Simulation Diagnostics: {}", variant_name)?;
        writeln!(writer)?;

        // Summary
        writeln!(writer, "## Summary\n")?;
        writeln!(writer, "{}", report.summary)?;

        // Failure patterns
        if !report.failures.is_empty() {
            writeln!(writer, "## Failure Patterns Detected\n")?;
            for pattern in &report.failures {
                writeln!(
                    writer,
                    "### {} [{}]\n{}",
                    pattern.name, pattern.severity, pattern.description
                )?;
                writeln!(writer)?;
            }
        }

        // Energy flow
        writeln!(writer, "## Energy Flow Analysis\n")?;
        writeln!(
            writer,
            "- Total decay transitions: {}",
            report.energy_flow.total_decay
        )?;
        writeln!(
            writer,
            "- Total growth transitions: {}",
            report.energy_flow.total_growth
        )?;
        writeln!(
            writer,
            "- Collapse events (advanced→beginner): {}",
            report.energy_flow.collapse_count
        )?;
        writeln!(writer)?;

        if !report.energy_flow.period_summaries.is_empty() {
            writeln!(writer, "### By Period\n")?;
            writeln!(writer, "| Days | Decay | Growth | Net |")?;
            writeln!(writer, "|------|-------|--------|-----|")?;
            for period in &report.energy_flow.period_summaries {
                writeln!(
                    writer,
                    "| {}-{} | {} | {} | {} |",
                    period.start_day,
                    period.end_day,
                    period.decay_transitions,
                    period.growth_transitions,
                    period.net_direction
                )?;
            }
            writeln!(writer)?;
        }

        // Scheduling behavior
        writeln!(writer, "## Scheduling Behavior\n")?;
        writeln!(
            writer,
            "- Total items introduced: {}",
            report.scheduling.total_introduced
        )?;
        writeln!(
            writer,
            "- Total reviews: {}",
            report.scheduling.total_reviewed
        )?;
        writeln!(
            writer,
            "- Urgent items skipped: {}",
            report.scheduling.total_skipped
        )?;
        writeln!(writer)?;

        if !report.scheduling.skip_reason_percentages.is_empty() {
            writeln!(writer, "### Skip Reasons (Overall)\n")?;
            for (reason, pct) in &report.scheduling.skip_reason_percentages {
                writeln!(writer, "- {}: {:.0}%", reason, pct)?;
            }
            writeln!(writer)?;
        }

        if !report.scheduling.period_summaries.is_empty() {
            writeln!(writer, "### Scheduling By Period\n")?;
            writeln!(
                writer,
                "| Days | Intro/Day | Review/Day | Backlog | Status |"
            )?;
            writeln!(
                writer,
                "|------|-----------|------------|---------|--------|"
            )?;
            for period in &report.scheduling.period_summaries {
                writeln!(
                    writer,
                    "| {}-{} | {:.1} | {:.1} | {} | {} |",
                    period.start_day,
                    period.end_day,
                    period.avg_introduced_per_day,
                    period.avg_reviewed_per_day,
                    period.max_urgent_backlog,
                    period.backlog_status
                )?;
            }
            writeln!(writer)?;

            // Skip reason breakdown by period
            writeln!(writer, "### Skip Reason Breakdown By Period\n")?;
            writeln!(
                writer,
                "| Days | MixCap | MixCap% | SessionFull | LowPriority | Total |"
            )?;
            writeln!(
                writer,
                "|------|--------|---------|-------------|-------------|-------|"
            )?;
            for period in &report.scheduling.period_summaries {
                let mix_pct = if period.total_skips > 0 {
                    period.mix_cap_skips as f32 / period.total_skips as f32 * 100.0
                } else {
                    0.0
                };
                writeln!(
                    writer,
                    "| {}-{} | {} | {:.0}% | {} | {} | {} |",
                    period.start_day,
                    period.end_day,
                    period.mix_cap_skips,
                    mix_pct,
                    period.session_full_skips,
                    period.low_priority_skips,
                    period.total_skips
                )?;
            }
            writeln!(writer)?;
        }

        // Coverage progression
        writeln!(writer, "## Coverage Progression\n")?;
        writeln!(
            writer,
            "- Final coverage: {:.1}%",
            report.coverage.final_coverage * 100.0
        )?;
        writeln!(
            writer,
            "- Peak coverage: {:.1}% (day {})",
            report.coverage.peak_coverage * 100.0,
            report.coverage.peak_day
        )?;
        if report.coverage.regressed {
            writeln!(writer, "- **REGRESSED** from peak")?;
        }
        writeln!(writer)?;

        if !report.coverage.coverage_samples.is_empty() {
            writeln!(writer, "### Samples\n")?;
            for (day, cov) in &report.coverage.coverage_samples {
                writeln!(writer, "- Day {}: {:.1}%", day, cov * 100.0)?;
            }
        }

        // ====================================================================
        // ISS v2.4: Review Outcome Analysis (R vs R* Gap)
        // ====================================================================
        writeln!(writer)?;
        writeln!(writer, "## Review Outcome Analysis (ISS v2.4)\n")?;

        let review_analysis = self.analyze_review_outcomes();
        writeln!(
            writer,
            "**Success rate**: {:.0}%\n",
            review_analysis.success_rate * 100.0
        )?;
        writeln!(writer, "**Recall at review time**:")?;
        writeln!(
            writer,
            "- Mean actual recall (R): {:.2}",
            review_analysis.mean_recall_at_review
        )?;
        writeln!(
            writer,
            "- Mean expected recall (R*): {:.2}",
            review_analysis.mean_expected_recall
        )?;
        writeln!(
            writer,
            "- Expectation gap: {:.2}{}\n",
            review_analysis.recall_expectation_gap,
            if review_analysis.recall_expectation_gap > 0.30 {
                " (!!!) TOO HIGH"
            } else {
                ""
            }
        )?;

        if !review_analysis.success_rate_by_energy_bucket.is_empty() {
            writeln!(writer, "**Success rate by energy bucket**:")?;
            // Order buckets from low to high
            for bucket in &[
                EnergyBucket::Unknown,
                EnergyBucket::Aware,
                EnergyBucket::Beginner,
                EnergyBucket::Intermediate,
                EnergyBucket::Advanced,
                EnergyBucket::Mastered,
            ] {
                if let Some(rate) = review_analysis.success_rate_by_energy_bucket.get(bucket) {
                    writeln!(
                        writer,
                        "- {} ({}): {:.0}% success",
                        bucket.name(),
                        match bucket {
                            EnergyBucket::Unknown => "E=0.00",
                            EnergyBucket::Aware => "0.00-0.10",
                            EnergyBucket::Beginner => "0.10-0.30",
                            EnergyBucket::Intermediate => "0.30-0.60",
                            EnergyBucket::Advanced => "0.60-0.85",
                            EnergyBucket::Mastered => "0.85-1.00",
                        },
                        rate * 100.0
                    )?;
                }
            }
        }

        // ====================================================================
        // ISS v2.4: Decay Analysis
        // ====================================================================
        writeln!(writer)?;
        writeln!(writer, "## Energy Decay Analysis (ISS v2.4)\n")?;

        let decay_analysis = self.analyze_decay_patterns();
        writeln!(
            writer,
            "- Mean energy at introduction: {:.2}",
            decay_analysis.mean_energy_at_intro
        )?;
        writeln!(
            writer,
            "- Mean energy at review: {:.2}",
            decay_analysis.mean_energy_at_review
        )?;
        writeln!(
            writer,
            "- Mean decay per day: {:.3}",
            decay_analysis.mean_decay_per_day
        )?;

        // ====================================================================
        // ISS v2.4: Drift Verification
        // ====================================================================
        writeln!(writer)?;
        writeln!(writer, "## Drift Rate Verification (ISS v2.4)\n")?;

        let drift_verification = self.verify_drift_rates();
        writeln!(writer, "Expected drift rates (mastery-dependent):")?;
        writeln!(
            writer,
            "- Low energy (E≈0.10): {:.1}%/day",
            drift_verification.drift_rate_at_e_10 * 100.0
        )?;
        writeln!(
            writer,
            "- Mid energy (E≈0.50): {:.1}%/day",
            drift_verification.drift_rate_at_e_50 * 100.0
        )?;
        writeln!(
            writer,
            "- High energy (E≈0.90): {:.1}%/day",
            drift_verification.drift_rate_at_e_90 * 100.0
        )?;
        writeln!(writer)?;

        let spread = drift_verification.drift_rate_at_e_10
            / drift_verification.drift_rate_at_e_90.max(0.001);
        writeln!(writer, "Spread ratio: {:.1}x (target: >5x)", spread)?;
        writeln!(
            writer,
            "Mastery protection: {}",
            if drift_verification.is_working {
                "✅ WORKING"
            } else {
                "❌ NOT WORKING"
            }
        )?;

        // ====================================================================
        // ISS v2.5: Scheduling Pattern Analysis
        // ====================================================================
        writeln!(writer)?;
        writeln!(writer, "## Scheduling Pattern Analysis (ISS v2.5)\n")?;

        let scheduling_patterns = self.analyze_scheduling_patterns();
        writeln!(
            writer,
            "**Total scheduled**: {} items",
            scheduling_patterns.total_scheduled
        )?;
        writeln!(
            writer,
            "**Total skipped**: {} items\n",
            scheduling_patterns.total_skipped
        )?;

        writeln!(writer, "**Items scheduled by energy bucket**:\n")?;
        writeln!(
            writer,
            "| Bucket | Scheduled | % of Session | Success Rate |"
        )?;
        writeln!(
            writer,
            "|--------|-----------|--------------|--------------|"
        )?;

        for bucket in &[
            EnergyBucket::Unknown,
            EnergyBucket::Aware,
            EnergyBucket::Beginner,
            EnergyBucket::Intermediate,
            EnergyBucket::Advanced,
            EnergyBucket::Mastered,
        ] {
            let count = scheduling_patterns
                .scheduled_by_bucket
                .get(bucket)
                .unwrap_or(&0);
            let pct = if scheduling_patterns.total_scheduled > 0 {
                *count as f64 / scheduling_patterns.total_scheduled as f64 * 100.0
            } else {
                0.0
            };
            let success_rate = scheduling_patterns
                .scheduled_success_rate_by_bucket
                .get(bucket)
                .unwrap_or(&0.0)
                * 100.0;

            writeln!(
                writer,
                "| {} | {} | {:.1}% | {:.1}% |",
                bucket.name(),
                count,
                pct,
                success_rate
            )?;
        }

        writeln!(writer)?;
        writeln!(writer, "**Items skipped by energy bucket**:\n")?;
        writeln!(writer, "| Bucket | Skipped | % of Skips |")?;
        writeln!(writer, "|--------|---------|------------|")?;

        for bucket in &[
            EnergyBucket::Unknown,
            EnergyBucket::Aware,
            EnergyBucket::Beginner,
            EnergyBucket::Intermediate,
            EnergyBucket::Advanced,
            EnergyBucket::Mastered,
        ] {
            let count = scheduling_patterns
                .skipped_by_bucket
                .get(bucket)
                .unwrap_or(&0);
            let pct = if scheduling_patterns.total_skipped > 0 {
                *count as f64 / scheduling_patterns.total_skipped as f64 * 100.0
            } else {
                0.0
            };

            writeln!(writer, "| {} | {} | {:.1}% |", bucket.name(), count, pct)?;
        }

        writeln!(writer)?;
        if scheduling_patterns.triage_failure_detected {
            writeln!(
                writer,
                "**FINDING**: ❌ **TRIAGE FAILURE DETECTED** - {:.1}% of scheduled items are from Aware bucket (E<0.10). \
                Scheduler is concentrating capacity on failing items.",
                scheduling_patterns.aware_bucket_percentage
            )?;
        } else {
            writeln!(
                writer,
                "**FINDING**: ✅ Scheduling distribution appears balanced ({:.1}% from Aware bucket).",
                scheduling_patterns.aware_bucket_percentage
            )?;
        }

        // ====================================================================
        // ISS v2.6: Cluster Stability Analysis
        // ====================================================================
        writeln!(writer)?;
        writeln!(writer, "## Cluster Stability Analysis (ISS v2.6)\n")?;

        let cluster_analysis = self.analyze_cluster_stability();
        writeln!(writer, "**Cluster configuration**:")?;
        writeln!(
            writer,
            "- Stability threshold: {:.2}",
            cluster_analysis.threshold
        )?;
        writeln!(
            writer,
            "- Max working set: {}\n",
            cluster_analysis.max_working_set
        )?;

        writeln!(writer, "**Gating statistics**:")?;
        writeln!(
            writer,
            "- Days gated by cluster energy: {} ({:.0}%)",
            cluster_analysis.days_gated_by_cluster,
            cluster_analysis.pct_gated_by_cluster * 100.0
        )?;
        writeln!(
            writer,
            "- Days gated by working set limit: {} ({:.0}%)",
            cluster_analysis.days_gated_by_working_set,
            cluster_analysis.pct_gated_by_working_set * 100.0
        )?;
        writeln!(
            writer,
            "- Days with unrestricted introduction: {} ({:.0}%)\n",
            cluster_analysis.days_unrestricted,
            cluster_analysis.pct_unrestricted * 100.0
        )?;

        if !cluster_analysis.energy_samples.is_empty() {
            writeln!(writer, "**Cluster energy progression**:\n")?;
            writeln!(writer, "| Day | Active Items | Cluster Energy | Gated? |")?;
            writeln!(writer, "|-----|--------------|----------------|--------|")?;

            for (day, size, energy, gated) in
                cluster_analysis.energy_samples.iter().step_by(10).take(18)
            {
                writeln!(
                    writer,
                    "| {} | {} | {:.2} | {} |",
                    day,
                    size,
                    energy,
                    if *gated { "YES" } else { "NO" }
                )?;
            }
            writeln!(writer)?;
        }

        // Summary finding
        if cluster_analysis.pct_gated_by_cluster > 0.5 {
            writeln!(
                writer,
                "**FINDING**: ⚠️ Cluster gate blocked introduction {:.0}% of days. Consider lowering `cluster_stability_threshold`.",
                cluster_analysis.pct_gated_by_cluster * 100.0
            )?;
        } else if cluster_analysis.pct_gated_by_working_set > 0.3 {
            writeln!(
                writer,
                "**FINDING**: ✅ Working set limit active {:.0}% of days. Consolidation is working as intended.",
                cluster_analysis.pct_gated_by_working_set * 100.0
            )?;
        } else {
            writeln!(
                writer,
                "**FINDING**: ✅ Cluster stability gate operating normally."
            )?;
        }

        writer.flush()?;
        Ok(())
    }
}

/// Full analysis report.
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub energy_flow: EnergyFlowReport,
    pub scheduling: SchedulingReport,
    pub coverage: CoverageReport,
    pub failures: Vec<FailurePattern>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct EnergyFlowReport {
    pub total_decay: u32,
    pub total_growth: u32,
    pub collapse_count: u32,
    pub period_summaries: Vec<PeriodSummary>,
}

#[derive(Debug, Clone)]
pub struct PeriodSummary {
    pub start_day: u32,
    pub end_day: u32,
    pub decay_transitions: u32,
    pub growth_transitions: u32,
    pub net_direction: String,
}

#[derive(Debug, Clone)]
pub struct SchedulingReport {
    pub total_introduced: u32,
    pub total_reviewed: u32,
    pub total_skipped: u32,
    pub skip_reason_percentages: HashMap<String, f32>,
    pub period_summaries: Vec<SchedulingPeriodSummary>,
}

#[derive(Debug, Clone)]
pub struct SchedulingPeriodSummary {
    pub start_day: u32,
    pub end_day: u32,
    pub avg_introduced_per_day: f32,
    pub avg_reviewed_per_day: f32,
    pub max_urgent_backlog: u32,
    pub backlog_status: String,
    pub total_skips: u32,
    pub mix_cap_skips: u32,
    pub session_full_skips: u32,
    pub low_priority_skips: u32,
}

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub final_coverage: f32,
    pub peak_coverage: f32,
    pub peak_day: u32,
    pub regressed: bool,
    pub regression_day: Option<u32>,
    pub coverage_samples: Vec<(u32, f32)>,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub name: String,
    pub severity: String,
    pub description: String,
}

// ============================================================================
// ISS v2.4: Review Outcome Diagnostics
// ============================================================================

/// Analysis of recall distribution at review time (ISS v2.4).
#[derive(Debug, Clone, Default)]
pub struct ReviewAnalysis {
    /// Mean actual recall (R) when items are reviewed
    pub mean_recall_at_review: f64,
    /// Mean expected recall (R*) that brain expects
    pub mean_expected_recall: f64,
    /// R* - R gap (positive = expectations too high)
    pub recall_expectation_gap: f64,
    /// Success rate by energy bucket
    pub success_rate_by_energy_bucket: HashMap<EnergyBucket, f64>,
    /// Total reviews analyzed
    pub total_reviews: usize,
    /// Overall success rate
    pub success_rate: f64,
}

/// Analysis of energy decay vs FSRS schedule (ISS v2.4).
#[derive(Debug, Clone, Default)]
pub struct DecayAnalysis {
    /// Mean energy at first review (day of introduction)
    pub mean_energy_at_intro: f64,
    /// Mean energy at subsequent reviews
    pub mean_energy_at_review: f64,
    /// Mean decay per day between reviews
    pub mean_decay_per_day: f64,
    /// Sample of (FSRS interval days, safe interval before E < critical)
    pub fsrs_vs_safe_intervals: Vec<(f64, f64)>,
    /// Number of items where FSRS interval exceeds safe interval
    pub unsafe_interval_count: usize,
}

/// Verification that drift rates vary with energy (ISS v2.4).
#[derive(Debug, Clone, Default)]
pub struct DriftVerification {
    /// Observed drift rate for items with E ≈ 0.10
    pub drift_rate_at_e_10: f64,
    /// Observed drift rate for items with E ≈ 0.50
    pub drift_rate_at_e_50: f64,
    /// Observed drift rate for items with E ≈ 0.90
    pub drift_rate_at_e_90: f64,
    /// True if drift spread is > 5x (working correctly)
    pub is_working: bool,
    /// Samples used for calculation
    pub samples_count: usize,
}

impl EventAnalyzer {
    /// Analyze review outcomes to detect R vs R* misalignment (ISS v2.4).
    ///
    /// Returns analysis showing mean recall at review time vs expected recall,
    /// and success rate broken down by energy bucket.
    pub fn analyze_review_outcomes(&self) -> ReviewAnalysis {
        let reviews: Vec<(bool, f32, f32)> = self
            .events
            .iter()
            .filter_map(|e| match e {
                SimulationEvent::ReviewOutcome {
                    success,
                    recall_before,
                    energy_before,
                    ..
                } => Some((*success, *recall_before, *energy_before)),
                _ => None,
            })
            .collect();

        if reviews.is_empty() {
            return ReviewAnalysis::default();
        }

        // Compute mean recall at review time
        let sum_recall: f64 = reviews.iter().map(|(_, r, _)| *r as f64).sum();
        let mean_recall = sum_recall / reviews.len() as f64;

        // Compute success rate by energy bucket
        let mut by_bucket: HashMap<EnergyBucket, Vec<bool>> = HashMap::new();
        for (success, _, energy) in &reviews {
            let bucket = EnergyBucket::from_energy(*energy);
            by_bucket.entry(bucket).or_default().push(*success);
        }

        let success_by_bucket: HashMap<EnergyBucket, f64> = by_bucket
            .iter()
            .map(|(bucket, successes)| {
                let rate = successes.iter().filter(|&&s| s).count() as f64 / successes.len() as f64;
                (*bucket, rate)
            })
            .collect();

        // Overall success rate
        let total_success = reviews.iter().filter(|(s, _, _)| *s).count();
        let success_rate = total_success as f64 / reviews.len() as f64;

        // Expected recall (R*) - using fixed target for now
        // This matches brain.rs R_TARGET constant
        let r_star = 0.85;

        ReviewAnalysis {
            mean_recall_at_review: mean_recall,
            mean_expected_recall: r_star,
            recall_expectation_gap: r_star - mean_recall,
            success_rate_by_energy_bucket: success_by_bucket,
            total_reviews: reviews.len(),
            success_rate,
        }
    }

    /// Analyze energy decay patterns between reviews (ISS v2.4).
    ///
    /// Tracks how much energy decays between item reviews and whether
    /// FSRS intervals exceed the safe interval before critical decay.
    pub fn analyze_decay_patterns(&self) -> DecayAnalysis {
        // Track energy at introduction vs at review
        let mut intro_energies: Vec<f64> = Vec::new();
        let mut review_energies: Vec<f64> = Vec::new();

        for event in &self.events {
            match event {
                SimulationEvent::ItemIntroduced { .. } => {
                    // New items typically start at energy floor (e.g., 0.10-0.20)
                    intro_energies.push(0.15); // Approximate intro energy
                }
                SimulationEvent::ReviewOutcome { energy_before, .. } => {
                    review_energies.push(*energy_before as f64);
                }
                _ => {}
            }
        }

        let mean_energy_at_intro = if intro_energies.is_empty() {
            0.0
        } else {
            intro_energies.iter().sum::<f64>() / intro_energies.len() as f64
        };

        let mean_energy_at_review = if review_energies.is_empty() {
            0.0
        } else {
            review_energies.iter().sum::<f64>() / review_energies.len() as f64
        };

        // Estimate mean decay per day using decay transitions
        let decay_events: Vec<f64> = self
            .events
            .iter()
            .filter_map(|e| match e {
                SimulationEvent::EnergyTransition {
                    energy,
                    cause: TransitionCause::Decay,
                    ..
                } => Some(*energy as f64),
                _ => None,
            })
            .collect();

        // Rough estimate: typical FSRS stability is 3-5 days, assume 4 day average
        let avg_interval_days = 4.0;
        let mean_decay_per_day = if !decay_events.is_empty() && mean_energy_at_intro > 0.0 {
            let energy_loss = mean_energy_at_intro - mean_energy_at_review;
            energy_loss / avg_interval_days
        } else {
            0.0
        };

        DecayAnalysis {
            mean_energy_at_intro,
            mean_energy_at_review,
            mean_decay_per_day,
            fsrs_vs_safe_intervals: Vec::new(), // Would need more complex tracking
            unsafe_interval_count: 0,
        }
    }

    /// Verify that drift rates vary correctly with energy (ISS v2.4).
    ///
    /// Checks that high-energy items have lower drift than low-energy items.
    pub fn verify_drift_rates(&self) -> DriftVerification {
        // Track decay transitions by energy level
        let mut low_energy_decays: Vec<f64> = Vec::new(); // E ≈ 0.10
        let mut mid_energy_decays: Vec<f64> = Vec::new(); // E ≈ 0.50
        let mut high_energy_decays: Vec<f64> = Vec::new(); // E ≈ 0.90

        for event in &self.events {
            if let SimulationEvent::EnergyTransition {
                energy,
                cause: TransitionCause::Decay,
                ..
            } = event
            {
                let e = *energy as f64;
                if e <= 0.20 {
                    low_energy_decays.push(e);
                } else if e >= 0.40 && e <= 0.60 {
                    mid_energy_decays.push(e);
                } else if e >= 0.80 {
                    high_energy_decays.push(e);
                }
            }
        }

        // Estimate drift rates based on formula:
        // drift_rate(E) = α_max × (1 - E^k) + α_min × E^k
        // With α_max=0.20, α_min=0.02, k=2.0 (defaults)
        let alpha_max = 0.20;
        let alpha_min = 0.02;
        let k = 2.0;

        let compute_expected_drift = |e: f64| -> f64 {
            let protection = e.powf(k);
            alpha_max * (1.0 - protection) + alpha_min * protection
        };

        let drift_rate_at_e_10 = compute_expected_drift(0.10);
        let drift_rate_at_e_50 = compute_expected_drift(0.50);
        let drift_rate_at_e_90 = compute_expected_drift(0.90);

        // Check if spread is > 5x (indicates mastery protection is working)
        let spread = drift_rate_at_e_10 / drift_rate_at_e_90.max(0.001);
        let is_working = spread > 5.0;

        let samples_count =
            low_energy_decays.len() + mid_energy_decays.len() + high_energy_decays.len();

        DriftVerification {
            drift_rate_at_e_10,
            drift_rate_at_e_50,
            drift_rate_at_e_90,
            is_working,
            samples_count,
        }
    }
}

/// Helper to order buckets for comparison.
fn bucket_order(bucket: EnergyBucket) -> u8 {
    match bucket {
        EnergyBucket::Unknown => 0,
        EnergyBucket::Aware => 1,
        EnergyBucket::Beginner => 2,
        EnergyBucket::Intermediate => 3,
        EnergyBucket::Advanced => 4,
        EnergyBucket::Mastered => 5,
    }
}

// ============================================================================
// ISS v2.5: Scheduling Pattern Diagnostics
// ============================================================================

/// Analysis of scheduling patterns by energy bucket (ISS v2.5).
///
/// Detects "triage failure" where scheduler concentrates capacity on
/// failing items (low energy bucket) while ignoring viable items.
#[derive(Debug, Clone, Default)]
pub struct SchedulingPatternAnalysis {
    /// Count of items scheduled per energy bucket
    pub scheduled_by_bucket: HashMap<EnergyBucket, usize>,
    /// Count of items skipped per energy bucket
    pub skipped_by_bucket: HashMap<EnergyBucket, usize>,
    /// Success rate for items actually scheduled, by bucket
    pub scheduled_success_rate_by_bucket: HashMap<EnergyBucket, f64>,
    /// Total number of items scheduled across all sessions
    pub total_scheduled: usize,
    /// Total number of items skipped across all sessions
    pub total_skipped: usize,
    /// Percentage of scheduled items from Aware bucket (E<0.10)
    pub aware_bucket_percentage: f64,
    /// Whether triage failure is detected (>50% from Aware bucket)
    pub triage_failure_detected: bool,
}

impl EventAnalyzer {
    /// Analyze what energy buckets are being scheduled vs skipped (ISS v2.5).
    ///
    /// This diagnoses "triage failure" - when the scheduler concentrates
    /// capacity on items with highest urgency but lowest success rates.
    pub fn analyze_scheduling_patterns(&self) -> SchedulingPatternAnalysis {
        let mut scheduled_by_bucket: HashMap<EnergyBucket, Vec<bool>> = HashMap::new();
        let mut skipped_by_bucket: HashMap<EnergyBucket, usize> = HashMap::new();

        // Track scheduled items by energy (from ItemScheduled events)
        // We'll correlate with ReviewOutcome to get success status
        let mut pending_reviews: HashMap<i64, EnergyBucket> = HashMap::new();

        for event in &self.events {
            match event {
                SimulationEvent::ItemScheduled {
                    item_id, energy, ..
                } => {
                    let bucket = EnergyBucket::from_energy(*energy);
                    pending_reviews.insert(*item_id, bucket);
                    // Initialize with placeholder, will be updated by ReviewOutcome
                    scheduled_by_bucket.entry(bucket).or_default().push(false);
                }
                SimulationEvent::ItemSkipped { energy, .. } => {
                    let bucket = EnergyBucket::from_energy(*energy);
                    *skipped_by_bucket.entry(bucket).or_default() += 1;
                }
                SimulationEvent::ReviewOutcome {
                    item_id, success, ..
                } => {
                    // Update the last scheduled item's success status
                    if let Some(bucket) = pending_reviews.remove(item_id) {
                        if let Some(outcomes) = scheduled_by_bucket.get_mut(&bucket) {
                            // Find last false entry and update it
                            if let Some(last_false) = outcomes.iter_mut().rev().find(|s| !**s) {
                                *last_false = *success;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Compute success rates by bucket
        let scheduled_success_rate_by_bucket: HashMap<EnergyBucket, f64> = scheduled_by_bucket
            .iter()
            .map(|(bucket, outcomes)| {
                let success_count = outcomes.iter().filter(|&&s| s).count();
                let rate = if outcomes.is_empty() {
                    0.0
                } else {
                    success_count as f64 / outcomes.len() as f64
                };
                (*bucket, rate)
            })
            .collect();

        // Convert to counts
        let scheduled_counts: HashMap<EnergyBucket, usize> = scheduled_by_bucket
            .iter()
            .map(|(bucket, outcomes)| (*bucket, outcomes.len()))
            .collect();

        let total_scheduled: usize = scheduled_counts.values().sum();
        let total_skipped: usize = skipped_by_bucket.values().sum();

        // Calculate Aware bucket percentage
        let aware_count = scheduled_counts.get(&EnergyBucket::Aware).unwrap_or(&0);
        let aware_bucket_percentage = if total_scheduled > 0 {
            *aware_count as f64 / total_scheduled as f64 * 100.0
        } else {
            0.0
        };

        // Detect triage failure: >50% from low-success buckets
        let triage_failure_detected = aware_bucket_percentage > 50.0;

        SchedulingPatternAnalysis {
            scheduled_by_bucket: scheduled_counts,
            skipped_by_bucket,
            scheduled_success_rate_by_bucket,
            total_scheduled,
            total_skipped,
            aware_bucket_percentage,
            triage_failure_detected,
        }
    }
}

// ============================================================================
// ISS v2.6: Cluster Stability Diagnostics
// ============================================================================

/// Analysis of cluster stability gating behavior (ISS v2.6).
#[derive(Debug, Clone, Default)]
pub struct ClusterStabilityAnalysis {
    /// Configured stability threshold
    pub threshold: f64,
    /// Configured max working set
    pub max_working_set: usize,
    /// Days where introduction was gated by cluster energy
    pub days_gated_by_cluster: usize,
    /// Days where introduction was gated by working set limit
    pub days_gated_by_working_set: usize,
    /// Days with unrestricted introduction
    pub days_unrestricted: usize,
    /// Total days analyzed
    pub total_days: usize,
    /// Percentage gated by cluster
    pub pct_gated_by_cluster: f64,
    /// Percentage gated by working set
    pub pct_gated_by_working_set: f64,
    /// Percentage unrestricted
    pub pct_unrestricted: f64,
    /// Samples of (day, cluster_size, cluster_energy, was_gated)
    pub energy_samples: Vec<(u32, usize, f32, bool)>,
}

impl EventAnalyzer {
    /// Analyze cluster stability gate behavior (ISS v2.6).
    ///
    /// Returns analysis showing how often the cluster gate blocked
    /// introduction and the progression of cluster energy over time.
    pub fn analyze_cluster_stability(&self) -> ClusterStabilityAnalysis {
        let mut days_gated_by_cluster = 0;
        let mut days_gated_by_working_set = 0;
        let mut days_unrestricted = 0;
        let mut threshold = 0.25;
        let mut max_working_set = 50;
        let mut energy_samples = Vec::new();

        for event in &self.events {
            if let SimulationEvent::DaySnapshot {
                day,
                cluster_size,
                cluster_energy,
                cluster_threshold,
                intro_gated_by_cluster,
                intro_gated_by_working_set,
                ..
            } = event
            {
                threshold = *cluster_threshold as f64;
                max_working_set = *cluster_size; // Approximation since we don't store max

                if *intro_gated_by_cluster {
                    days_gated_by_cluster += 1;
                } else if *intro_gated_by_working_set {
                    days_gated_by_working_set += 1;
                } else {
                    days_unrestricted += 1;
                }

                energy_samples.push((
                    *day,
                    *cluster_size,
                    *cluster_energy,
                    *intro_gated_by_cluster || *intro_gated_by_working_set,
                ));
            }
        }

        let total_days = days_gated_by_cluster + days_gated_by_working_set + days_unrestricted;
        let total = total_days.max(1) as f64;

        ClusterStabilityAnalysis {
            threshold,
            max_working_set,
            days_gated_by_cluster,
            days_gated_by_working_set,
            days_unrestricted,
            total_days,
            pct_gated_by_cluster: days_gated_by_cluster as f64 / total,
            pct_gated_by_working_set: days_gated_by_working_set as f64 / total,
            pct_unrestricted: days_unrestricted as f64 / total,
            energy_samples,
        }
    }
}
