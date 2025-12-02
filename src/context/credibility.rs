use crate::analysis::credibility::CredibilityReport;
use crate::script::VideoScript;

pub struct CredibilityContext;

impl CredibilityContext {
    pub fn run(script: &VideoScript) -> CredibilityReport {
        // Pillar 3: Credibility (Trustworthy) - Analysis
        println!("\n🛡️ Analyzing Credibility...");
        let report = crate::analysis::credibility::CredibilityAnalyzer::analyze(script);

        println!("   Score: {}/100", report.score);

        if !report.claims.is_empty() {
            println!("   🔍 Detected {} claims:", report.claims.len());
            for claim in &report.claims {
                let status = if claim.verified {
                    "✅ Verified"
                } else {
                    "⚠️ Unverified"
                };
                println!("      - [{}] \"{}\" ({})", status, claim.text, claim.reason);
            }
        } else {
            println!("   ✅ No specific claims detected");
        }

        if !report.citations.is_empty() {
            println!("   📚 Citations:");
            for citation in &report.citations {
                println!("      - {}", citation);
            }
        } else {
            println!("   ⚠️  No citations provided");
        }

        println!("\n   ✅ Quality Checklist:");
        for item in &report.checklist {
            let icon = if item.passed { "✓" } else { "❌" };
            println!("      {} [{}] {}", icon, item.category, item.message);
        }

        report
    }
}
