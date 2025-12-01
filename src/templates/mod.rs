use crate::script::{
    Color, Layer, Metadata, Position, Resolution, Scene, SceneType, VideoScript,
};
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum TemplateType {
    Explainer,
    Tutorial,
    Storytelling,
}

pub struct ScriptTemplate;

impl ScriptTemplate {
    pub fn generate(template_type: TemplateType, duration: f32) -> VideoScript {
        match template_type {
            TemplateType::Explainer => Self::generate_explainer(duration),
            TemplateType::Tutorial => Self::generate_tutorial(duration),
            TemplateType::Storytelling => Self::generate_storytelling(duration),
        }
    }

    fn generate_explainer(total_duration: f32) -> VideoScript {
        let hook_duration = (total_duration * 0.15).max(3.0);
        let payoff_duration = (total_duration * 0.10).max(3.0);
        let body_duration = total_duration - hook_duration - payoff_duration;

        VideoScript {
            metadata: Metadata {
                title: "Explainer Video".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: total_duration,
                description: Some("Generated explainer template".into()),
                citations: vec![],
            },
            scenes: vec![
                Self::create_scene("Hook", SceneType::Hook, hook_duration, "Hook: Grab Attention"),
                Self::create_scene("Body", SceneType::Body, body_duration, "Body: Explain Concept"),
                Self::create_scene("Payoff", SceneType::Payoff, payoff_duration, "Payoff: Call to Action"),
            ],
            audio: None,
        }
    }

    fn generate_tutorial(total_duration: f32) -> VideoScript {
        let intro_duration = (total_duration * 0.10).max(3.0);
        let recap_duration = (total_duration * 0.15).max(5.0);
        let steps_duration = total_duration - intro_duration - recap_duration;

        VideoScript {
            metadata: Metadata {
                title: "Tutorial Video".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: total_duration,
                description: Some("Generated tutorial template".into()),
                citations: vec![],
            },
            scenes: vec![
                Self::create_scene("Intro", SceneType::Hook, intro_duration, "Intro: What we'll build"),
                Self::create_scene("Steps", SceneType::Body, steps_duration, "Steps: Step-by-step guide"),
                Self::create_scene("Recap", SceneType::Payoff, recap_duration, "Recap: Summary & Next Steps"),
            ],
            audio: None,
        }
    }

    fn generate_storytelling(total_duration: f32) -> VideoScript {
        let setup_duration = total_duration * 0.20;
        let conflict_duration = total_duration * 0.50;
        let resolution_duration = total_duration * 0.30;

        VideoScript {
            metadata: Metadata {
                title: "Story Video".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: total_duration,
                description: Some("Generated storytelling template".into()),
                citations: vec![],
            },
            scenes: vec![
                Self::create_scene("Setup", SceneType::Hook, setup_duration, "Setup: The World & Characters"),
                Self::create_scene("Conflict", SceneType::Body, conflict_duration, "Conflict: The Challenge"),
                Self::create_scene("Resolution", SceneType::Payoff, resolution_duration, "Resolution: The Change"),
            ],
            audio: None,
        }
    }

    fn create_scene(id: &str, scene_type: SceneType, duration: f32, text: &str) -> Scene {
        Scene {
            id: id.into(),
            scene_type,
            duration,
            layers: vec![Layer::Text {
                content: text.into(),
                font: "assets/fonts/Inter-Bold.ttf".into(),
                font_size: 60.0,
                color: Color { r: 255, g: 255, b: 255, a: 255 },
                position: Position { x: 960, y: 540 },
                effects: vec![],
            }],
            transition: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_explainer() {
        let script = ScriptTemplate::generate(TemplateType::Explainer, 60.0);
        assert_eq!(script.scenes.len(), 3);
        assert_eq!(script.scenes[0].scene_type, SceneType::Hook);
        assert_eq!(script.scenes[2].scene_type, SceneType::Payoff);
        assert!((script.scenes[0].duration - 9.0).abs() < 0.1); // 15% of 60 = 9
    }

    #[test]
    fn test_generate_tutorial() {
        let script = ScriptTemplate::generate(TemplateType::Tutorial, 100.0);
        assert_eq!(script.scenes.len(), 3);
        assert_eq!(script.scenes[0].scene_type, SceneType::Hook); // Intro maps to Hook
    }
}
