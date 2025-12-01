use crate::script::{Scene, SceneType, VideoScript};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct PacingAlert {
    pub scene_index: usize,
    pub wpm: f32,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct RetentionWarning {
    pub scene_index: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct NarrativeReport {
    pub structure_valid: bool,
    pub structure_errors: Vec<String>,
    pub pacing_alerts: Vec<PacingAlert>,
    pub retention_warnings: Vec<RetentionWarning>,
    pub score: u32,
}

pub struct NarrativeAnalyzer;

impl NarrativeAnalyzer {
    pub fn analyze(script: &VideoScript) -> NarrativeReport {
        let (structure_valid, structure_errors) = Self::validate_structure(script);
        let pacing_alerts = Self::analyze_pacing(script);
        let retention_warnings = Self::analyze_visual_density(script);

        let mut score: i32 = 100;
        if !structure_valid {
            score = score.saturating_sub(30);
        }
        score = score.saturating_sub((pacing_alerts.len() * 10) as i32);
        score = score.saturating_sub((retention_warnings.len() * 5) as i32);

        NarrativeReport {
            structure_valid,
            structure_errors,
            pacing_alerts,
            retention_warnings,
            score: score.max(0) as u32,
        }
    }

    fn validate_structure(script: &VideoScript) -> (bool, Vec<String>) {
        let mut has_hook = false;
        let mut has_body = false;
        let mut has_payoff = false;
        let mut errors = Vec::new();

        for scene in &script.scenes {
            match scene.scene_type {
                SceneType::Hook => has_hook = true,
                SceneType::Body => has_body = true,
                SceneType::Payoff => has_payoff = true,
            }
        }

        if !has_hook {
            errors.push("Missing 'Hook' scene".to_string());
        }
        if !has_body {
            errors.push("Missing 'Body' scene".to_string());
        }
        if !has_payoff {
            errors.push("Missing 'Payoff' scene".to_string());
        }

        (errors.is_empty(), errors)
    }

    fn analyze_pacing(script: &VideoScript) -> Vec<PacingAlert> {
        let mut alerts = Vec::new();

        for (i, scene) in script.scenes.iter().enumerate() {
            let word_count = Self::count_words(scene);
            let duration_min = scene.duration / 60.0;

            if duration_min == 0.0 {
                continue;
            }

            let wpm = word_count as f32 / duration_min;
            let (min_wpm, max_wpm) = match scene.scene_type {
                SceneType::Hook => (140.0, 170.0),
                SceneType::Body => (130.0, 150.0),
                SceneType::Payoff => (120.0, 140.0),
            };

            if wpm < min_wpm {
                alerts.push(PacingAlert {
                    scene_index: i,
                    wpm,
                    message: format!(
                        "Scene {} is too slow ({:.0} WPM). Target: {:.0}-{:.0}",
                        i + 1,
                        wpm,
                        min_wpm,
                        max_wpm
                    ),
                });
            } else if wpm > max_wpm {
                alerts.push(PacingAlert {
                    scene_index: i,
                    wpm,
                    message: format!(
                        "Scene {} is too fast ({:.0} WPM). Target: {:.0}-{:.0}",
                        i + 1,
                        wpm,
                        min_wpm,
                        max_wpm
                    ),
                });
            }
        }

        alerts
    }

    fn analyze_visual_density(script: &VideoScript) -> Vec<RetentionWarning> {
        let mut warnings = Vec::new();

        for (i, scene) in script.scenes.iter().enumerate() {
            // Rule: Scenes longer than 10s should have multiple visual layers
            if scene.duration > 10.0 && scene.layers.len() < 2 {
                warnings.push(RetentionWarning {
                    scene_index: i,
                    message: format!("Scene {} is long ({:.1}s) but has low visual density. Consider adding more layers.", i + 1, scene.duration),
                });
            }
        }

        warnings
    }

    fn count_words(scene: &Scene) -> usize {
        let mut count = 0;
        for layer in &scene.layers {
            if let crate::script::Layer::Text { content, .. } = layer {
                count += content.unicode_words().count();
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Layer, Metadata, Resolution, Scene, SceneType};

    fn create_test_script(scenes: Vec<Scene>) -> VideoScript {
        VideoScript {
            metadata: Metadata {
                title: "Test".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: 0.0,
                description: None,
                citations: vec![],
            },
            scenes,
            audio: None,
        }
    }

    fn create_scene(scene_type: SceneType, duration: f32, text: &str) -> Scene {
        Scene {
            id: "test".into(),
            scene_type,
            duration,
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
        }
    }

    #[test]
    fn test_structure_validation() {
        let script = create_test_script(vec![
            create_scene(SceneType::Hook, 5.0, "Hook"),
            create_scene(SceneType::Body, 5.0, "Body"),
        ]);
        let report = NarrativeAnalyzer::analyze(&script);
        assert!(!report.structure_valid);
        assert!(report
            .structure_errors
            .contains(&"Missing 'Payoff' scene".to_string()));
    }

    #[test]
    fn test_pacing_analysis() {
        // Hook target: 140-170 WPM.
        // 10 words in 2 seconds = 300 WPM (Too fast)
        let script = create_test_script(vec![
            create_scene(
                SceneType::Hook,
                2.0,
                "One two three four five six seven eight nine ten",
            ),
            create_scene(SceneType::Body, 5.0, "Body"),
            create_scene(SceneType::Payoff, 5.0, "Payoff"),
        ]);
        let report = NarrativeAnalyzer::analyze(&script);
        assert!(!report.pacing_alerts.is_empty());
        assert!(report.pacing_alerts[0].wpm > 170.0);
    }
}
