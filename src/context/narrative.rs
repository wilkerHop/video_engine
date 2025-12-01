use crate::script::VideoScript;

use crate::analysis::narrative::NarrativeReport;

pub struct NarrativeContext;

impl NarrativeContext {
    pub fn run(script: &VideoScript) -> NarrativeReport {
        // Pillar 2: Narrative (Engaging) - Analysis
        println!("\n📊 Analyzing Narrative Structure...");
        let report = crate::analysis::narrative::NarrativeAnalyzer::analyze(script);

        println!("   Score: {}/100", report.score);

        // Structure validation
        if !report.structure_valid {
            println!("   ❌ Structure Issues:");
            for error in &report.structure_errors {
                println!("      - {}", error);
            }
        } else {
            println!("   ✅ Structure: Valid (Hook → Body → Payoff)");
        }

        // Structure recommendations
        if !report.structure_recommendations.is_empty() {
            println!("   💡 Structure Recommendations:");
            for rec in &report.structure_recommendations {
                let emoji = match rec.severity {
                    crate::analysis::narrative::Severity::Error => "❌",
                    crate::analysis::narrative::Severity::Warning => "⚠️",
                    crate::analysis::narrative::Severity::Info => "ℹ️",
                };
                println!("      {} [{}] {}", emoji, rec.category, rec.message);
            }
        }

        // Pacing alerts
        if !report.pacing_alerts.is_empty() {
            println!("   ⚠️ Pacing Alerts:");
            for alert in &report.pacing_alerts {
                println!("      - {}", alert.message);
            }
        } else {
            println!("   ✅ Pacing: Optimal");
        }

        // Retention warnings
        if !report.retention_warnings.is_empty() {
            println!("   ⚠️ Retention Warnings:");
            for warning in &report.retention_warnings {
                println!("      - {}", warning.message);
            }
        }

        // Advanced Retention Analysis
        println!("\n🎯 Analyzing Retention Metrics...");
        let heatmap = crate::analysis::retention::RetentionAnalyzer::generate_heatmap(script);
        println!(
            "   Overall Retention Score: {:.1}/100",
            heatmap.overall_retention_score
        );

        if !heatmap.critical_moments.is_empty() {
            println!("   ⚠️  Critical Moments (Low Retention):");
            for scene_idx in &heatmap.critical_moments {
                let scene_retention = &heatmap.scene_scores[*scene_idx];
                println!(
                    "      - Scene {}: {:.1} momentum, {:.1} retention",
                    scene_idx + 1,
                    scene_retention.momentum,
                    scene_retention.retention_score
                );
            }
        } else {
            println!("   ✅ No critical retention drop-offs detected");
        }

        let dropoff_predictions =
            crate::analysis::retention::RetentionAnalyzer::predict_dropoff(script);
        if !dropoff_predictions.is_empty() {
            println!("   📉 Drop-off Predictions:");
            for pred in dropoff_predictions.iter().take(3) {
                println!(
                    "      - Scene {}: {:.0}% predicted drop-off ({})",
                    pred.scene_index + 1,
                    pred.predicted_dropoff_percent,
                    pred.reason
                );
            }
        }

        report
    }
}
