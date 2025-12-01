use crate::script::VideoScript;
use regex::Regex;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Claim {
    pub text: String,
    pub scene_index: usize,
    pub verified: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistItem {
    pub passed: bool,
    pub message: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredibilityReport {
    pub score: u32,
    pub claims: Vec<Claim>,
    pub citations: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
}

pub struct CredibilityAnalyzer;

impl CredibilityAnalyzer {
    pub fn analyze(script: &VideoScript) -> CredibilityReport {
        let claims = Self::detect_claims(script);
        let citations = script.metadata.citations.clone();

        let verified_claims = claims.iter().filter(|c| c.verified).count();
        let total_claims = claims.len();

        let score = if total_claims == 0 {
            100
        } else {
            let base_score = (verified_claims as f32 / total_claims as f32) * 100.0;
            // Penalize for unverified claims
            let unverified_count = total_claims - verified_claims;
            (base_score as i32 - (unverified_count as i32 * 10)).max(0) as u32
        };

        let checklist = Self::generate_checklist(script, &citations);

        CredibilityReport {
            score,
            claims,
            citations,
            checklist,
        }
    }

    fn generate_checklist(script: &VideoScript, citations: &[String]) -> Vec<ChecklistItem> {
        let mut items = Vec::new();

        // 1. Citation Format Check
        if citations.is_empty() {
            items.push(ChecklistItem {
                passed: false,
                category: "Citations".to_string(),
                message: "No citations provided. Add sources to build trust.".to_string(),
            });
        } else {
            for citation in citations {
                let is_url = citation.starts_with("http");
                let is_academic = citation.contains('[') && citation.contains(']'); // Naive check for [Author, Year]

                if !is_url && !is_academic && citation.len() < 10 {
                    items.push(ChecklistItem {
                        passed: false,
                        category: "Citations".to_string(),
                        message: format!(
                            "Citation '{}' is too vague. Use a URL or academic format.",
                            citation
                        ),
                    });
                }
            }
            items.push(ChecklistItem {
                passed: true,
                category: "Citations".to_string(),
                message: format!("{} citations provided.", citations.len()),
            });
        }

        // 2. Weasel Word Detection
        let weasel_regex = Regex::new(
            r"(?i)\b(many people|some people|it is believed|studies show|experts agree)\b",
        )
        .unwrap();
        let mut weasel_count = 0;

        for scene in &script.scenes {
            for layer in &scene.layers {
                if let crate::script::Layer::Text { content, .. } = layer {
                    if weasel_regex.is_match(content) {
                        weasel_count += 1;
                    }
                }
            }
        }

        if weasel_count > 0 {
            items.push(ChecklistItem {
                passed: false,
                category: "Clarity".to_string(),
                message: format!(
                    "Detected {} instance(s) of weasel words (e.g., 'many people'). Be specific.",
                    weasel_count
                ),
            });
        } else {
            items.push(ChecklistItem {
                passed: true,
                category: "Clarity".to_string(),
                message: "No vague 'weasel words' detected.".to_string(),
            });
        }

        // 3. Tone Consistency (Hype Check)
        let hype_regex = Regex::new(
            r"(?i)\b(amazing|incredible|revolutionary|game-changing|miracle|best ever)\b",
        )
        .unwrap();
        let mut hype_count = 0;
        let mut total_words = 0;

        for scene in &script.scenes {
            for layer in &scene.layers {
                if let crate::script::Layer::Text { content, .. } = layer {
                    hype_count += hype_regex.find_iter(content).count();
                    total_words += content.split_whitespace().count();
                }
            }
        }

        if total_words > 0 {
            let hype_ratio = hype_count as f32 / total_words as f32;
            if hype_ratio > 0.05 {
                // >5% hype words
                items.push(ChecklistItem {
                    passed: false,
                    category: "Tone".to_string(),
                    message:
                        "High hype factor detected. Tone down superlatives for better credibility."
                            .to_string(),
                });
            } else {
                items.push(ChecklistItem {
                    passed: true,
                    category: "Tone".to_string(),
                    message: "Tone appears professional and balanced.".to_string(),
                });
            }
        }

        items
    }

    fn detect_claims(script: &VideoScript) -> Vec<Claim> {
        let mut claims = Vec::new();

        // Regex patterns for claim detection
        let stat_regex = Regex::new(r"\d+%|\d+ out of \d+|\d+x faster").unwrap();
        let superlative_regex =
            Regex::new(r"(?i)\b(best|fastest|first|only|proven|guaranteed)\b").unwrap();
        let absolute_regex = Regex::new(r"(?i)\b(always|never|everyone|nobody)\b").unwrap();

        for (i, scene) in script.scenes.iter().enumerate() {
            for layer in &scene.layers {
                if let crate::script::Layer::Text { content, .. } = layer {
                    let mut is_claim = false;
                    let mut reason = String::new();

                    if stat_regex.is_match(content) {
                        is_claim = true;
                        reason = "Contains statistics".to_string();
                    } else if superlative_regex.is_match(content) {
                        is_claim = true;
                        reason = "Uses superlative language".to_string();
                    } else if absolute_regex.is_match(content) {
                        is_claim = true;
                        reason = "Uses absolute terms".to_string();
                    }

                    if is_claim {
                        // Check if this claim is covered by any citation
                        // This is a naive check: if we have citations, we assume claims are verified for now
                        // A more advanced version would link specific citations to specific claims
                        let verified = !script.metadata.citations.is_empty();

                        claims.push(Claim {
                            text: content.clone(),
                            scene_index: i,
                            verified,
                            reason,
                        });
                    }
                }
            }
        }

        claims
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Layer, Metadata, Resolution, Scene, SceneType};

    fn create_test_script(citations: Vec<String>, text: &str) -> VideoScript {
        VideoScript {
            metadata: Metadata {
                title: "Test".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: 0.0,
                description: None,
                citations,
            },
            scenes: vec![Scene {
                id: "test".into(),
                duration: 5.0,
                scene_type: SceneType::Body,
                layers: vec![Layer::Text {
                    content: text.into(),
                    font: "font.ttf".into(),
                    font_size: 24.0,
                    color: crate::script::Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                    position: crate::script::Position { x: 0, y: 0 },
                    effects: vec![],
                }],
                transition: None,
            }],
            audio: None,
        }
    }

    #[test]
    fn test_claim_detection_stats() {
        let script = create_test_script(vec![], "Rust is 10x faster than Python");
        let report = CredibilityAnalyzer::analyze(&script);
        assert_eq!(report.claims.len(), 1);
        assert_eq!(report.claims[0].reason, "Contains statistics");
        assert!(!report.claims[0].verified); // No citations
        assert!(report.score < 100);
    }

    #[test]
    fn test_claim_detection_superlative() {
        let script = create_test_script(vec![], "The best programming language");
        let report = CredibilityAnalyzer::analyze(&script);
        assert_eq!(report.claims.len(), 1);
        assert_eq!(report.claims[0].reason, "Uses superlative language");
    }

    #[test]
    fn test_claim_verification() {
        let script =
            create_test_script(vec!["Benchmark results 2024".into()], "Rust is 10x faster");
        let report = CredibilityAnalyzer::analyze(&script);
        assert_eq!(report.claims.len(), 1);
        assert!(report.claims[0].verified); // Has citations
        assert_eq!(report.score, 100);
    }

    #[test]
    fn test_checklist_citations() {
        // Test with no citations
        let script = create_test_script(vec![], "Some content");
        let report = CredibilityAnalyzer::analyze(&script);
        let citation_check = report
            .checklist
            .iter()
            .find(|i| i.category == "Citations")
            .unwrap();
        assert!(!citation_check.passed);

        // Test with vague citation
        let script = create_test_script(vec!["Google".into()], "Some content");
        let report = CredibilityAnalyzer::analyze(&script);
        let citation_check = report
            .checklist
            .iter()
            .find(|i| i.category == "Citations")
            .unwrap();
        assert!(!citation_check.passed);
        assert!(citation_check.message.contains("too vague"));

        // Test with valid URL
        let script = create_test_script(vec!["https://rust-lang.org".into()], "Some content");
        let report = CredibilityAnalyzer::analyze(&script);
        let citation_check = report
            .checklist
            .iter()
            .find(|i| i.category == "Citations")
            .unwrap();
        assert!(citation_check.passed);
    }

    #[test]
    fn test_checklist_weasel_words() {
        let script = create_test_script(vec![], "Many people say this is good.");
        let report = CredibilityAnalyzer::analyze(&script);
        let clarity_check = report
            .checklist
            .iter()
            .find(|i| i.category == "Clarity")
            .unwrap();
        assert!(!clarity_check.passed);
        assert!(clarity_check.message.contains("weasel words"));
    }

    #[test]
    fn test_checklist_tone() {
        let script = create_test_script(
            vec![],
            "This is the most amazing, incredible, revolutionary product ever!",
        );
        let report = CredibilityAnalyzer::analyze(&script);
        let tone_check = report
            .checklist
            .iter()
            .find(|i| i.category == "Tone")
            .unwrap();
        assert!(!tone_check.passed);
        assert!(tone_check.message.contains("High hype factor"));
    }
}
