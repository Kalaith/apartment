//! Markdown rendering for the headless balance harness.

use super::{strategies, summarize, StrategySummary, TierSummary};
use std::fmt::Write;

pub(super) fn format_report(tiers: &[TierSummary], seeds: u64) -> String {
    let mut out = String::new();
    writeln!(out, "# Second Story — Representative Balance Report\n").unwrap();
    writeln!(
        out,
        "All {} campaign templates, {} deterministic seeds per strategy, and the live 36-month rules. Runs stop at terminal outcomes.\n",
        tiers.len(), seeds
    )
    .unwrap();

    for tier in tiers {
        writeln!(
            out,
            "## {} — {} (starts ${})\n",
            tier.template.name, tier.template.difficulty, tier.starting_cash
        )
        .unwrap();
        writeln!(out, "| Strategy | Score | Cash | Operating net/mo | Occ. | Happy | Cond. | Apps | Leaves | Invest $ / payback | Missions / grants | Condos | Buildings | V/A/B |\n|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|").unwrap();
        for summary in &tier.strategies {
            let payback = if summary.mean_payback_months.is_nan() {
                "—".to_string()
            } else {
                format!("{:.1} mo", summary.mean_payback_months)
            };
            writeln!(
                out,
                "| {} | {} | {} | {} | {:.0}% | {} | {} | {:.1} | {:.1} | {} / {} | {:.1} / {} | {:.1} | {:.1} | {}/{}/{} |",
                summary.name,
                summary.mean_score,
                summary.mean_final_balance,
                summary.mean_steady_net,
                summary.mean_end_occupancy * 100.0,
                summary.mean_end_happiness,
                summary.mean_end_condition,
                summary.mean_applications,
                summary.mean_departures,
                summary.mean_investment_spend,
                payback,
                summary.mean_missions_completed,
                summary.mean_mission_cash,
                summary.mean_condos_sold,
                summary.mean_buildings_bought,
                summary.victory_count,
                summary.all_left_count,
                summary.bankruptcy_count,
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    out
}

/// Run the full harness and produce the balance report string.
pub(super) fn generate_report(seeds: u64) -> String {
    let policies = strategies();
    let templates = crate::data::templates::load_templates()
        .map(|templates| templates.templates)
        .unwrap_or_default();
    let tiers = templates
        .into_iter()
        .map(|template| {
            let mut config = crate::data::config::load_config();
            let starting_cash = config.apply_difficulty(&template.difficulty);
            let summaries: Vec<StrategySummary> = policies
                .iter()
                .map(|strategy| summarize(&template, strategy, seeds))
                .collect();
            TierSummary {
                template,
                starting_cash,
                strategies: summaries,
            }
        })
        .collect::<Vec<_>>();
    format_report(&tiers, seeds)
}
