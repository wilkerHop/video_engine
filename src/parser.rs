use crate::script::VideoScript;
use anyhow::{Context, Result};
use std::path::Path;

/// Script parser that handles JSON/TOML video scripts
pub struct ScriptParser;

impl ScriptParser {
    /// Parse a JSON script file
    pub fn parse_json(path: &Path) -> Result<VideoScript> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read script file: {}", path.display()))?;

        let script: VideoScript = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON script: {}", path.display()))?;

        Self::validate_script(&script)?;

        Ok(script)
    }

    /// Validate the script structure
    fn validate_script(script: &VideoScript) -> Result<()> {
        // Validate metadata
        if script.metadata.title.is_empty() {
            anyhow::bail!("Script title cannot be empty");
        }

        if script.metadata.fps == 0 {
            anyhow::bail!("FPS must be greater than 0");
        }

        if script.metadata.duration <= 0.0 {
            anyhow::bail!("Duration must be positive");
        }

        // Validate scenes
        if script.scenes.is_empty() {
            anyhow::bail!("Script must contain at least one scene");
        }

        for (idx, scene) in script.scenes.iter().enumerate() {
            if scene.id.is_empty() {
                anyhow::bail!("Scene {} has empty ID", idx);
            }

            if scene.duration <= 0.0 {
                anyhow::bail!("Scene '{}' duration must be positive", scene.id);
            }

            if scene.layers.is_empty() {
                anyhow::bail!("Scene '{}' must have at least one layer", scene.id);
            }
        }

        // Validate total duration matches scenes
        let total_scene_duration: f32 = script.scenes.iter().map(|s| s.duration).sum();
        let duration_diff = (total_scene_duration - script.metadata.duration).abs();

        if duration_diff > 0.1 {
            eprintln!(
                "Warning: Total scene duration ({:.2}s) differs from metadata duration ({:.2}s)",
                total_scene_duration, script.metadata.duration
            );
        }

        Ok(())
    }

    /// Get a summary of the script structure
    pub fn summarize(script: &VideoScript) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Title: {}\n", script.metadata.title));
        summary.push_str(&format!(
            "Resolution: {}x{}\n",
            script.metadata.resolution.dimensions().0,
            script.metadata.resolution.dimensions().1
        ));
        summary.push_str(&format!("FPS: {}\n", script.metadata.fps));
        summary.push_str(&format!("Duration: {:.2}s\n", script.metadata.duration));
        summary.push_str(&format!("Scenes: {}\n", script.scenes.len()));

        for (idx, scene) in script.scenes.iter().enumerate() {
            summary.push_str(&format!(
                "  Scene {}: '{}' ({:.2}s, {} layers)\n",
                idx + 1,
                scene.id,
                scene.duration,
                scene.layers.len()
            ));
        }

        if let Some(audio) = &script.audio {
            summary.push_str(&format!("Audio tracks: {}\n", audio.tracks.len()));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"
        {
            "metadata": {
                "title": "Test Video",
                "resolution": "1920x1080",
                "fps": 60,
                "duration": 10.0
            },
            "scenes": [
                {
                    "id": "scene1",
                    "duration": 10.0,
                    "layers": [
                        {
                            "type": "image",
                            "source": "test.png"
                        }
                    ]
                }
            ]
        }
        "#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let result = ScriptParser::parse_json(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"
        {
            "metadata": {
                "title": "",
                "resolution": "1920x1080",
                "fps": 0,
                "duration": -1.0
            },
            "scenes": []
        }
        "#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let result = ScriptParser::parse_json(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_summarize() {
        let json = r#"
        {
            "metadata": {
                "title": "My Video",
                "resolution": "1920x1080",
                "fps": 30,
                "duration": 5.0
            },
            "scenes": [
                {
                    "id": "intro",
                    "duration": 5.0,
                    "layers": [
                        {
                            "type": "image",
                            "source": "bg.png"
                        }
                    ]
                }
            ]
        }
        "#;

        let script: VideoScript = serde_json::from_str(json).unwrap();
        let summary = ScriptParser::summarize(&script);
        assert!(summary.contains("My Video"));
        assert!(summary.contains("1920x1080"));
        assert!(summary.contains("30"));
    }

    #[test]
    fn test_validate_script_edge_cases() {
        // Test scene with no layers
        let json = r#"{
            "metadata": {"title": "Test", "resolution": "1920x1080", "fps": 30, "duration": 5.0},
            "scenes": [{"id": "s1", "duration": 5.0, "layers": []}]
        }"#;
        let script: VideoScript = serde_json::from_str(json).unwrap();
        let result = ScriptParser::validate_script(&script);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have at least one layer"));
    }

    #[test]
    fn test_validate_script_duration_mismatch() {
        // Test duration warning (should succeed but log warning)
        let json = r#"{
            "metadata": {"title": "Test", "resolution": "1920x1080", "fps": 30, "duration": 10.0},
            "scenes": [{"id": "s1", "duration": 5.0, "layers": [{"type": "image", "source": "t.png"}]}]
        }"#;
        let script: VideoScript = serde_json::from_str(json).unwrap();
        // Should succeed but print warning to stderr
        let result = ScriptParser::validate_script(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_summarize_with_audio() {
        let json = r#"{
            "metadata": {"title": "Test", "resolution": "1280x720", "fps": 24, "duration": 3.0},
            "scenes": [{"id": "s1", "duration": 3.0, "layers": [{"type": "image", "source": "t.png"}]}],
            "audio": {"tracks": [{"source": "music.mp3"}]}
        }"#;
        let script: VideoScript = serde_json::from_str(json).unwrap();
        let summary = ScriptParser::summarize(&script);
        assert!(summary.contains("1280x720"));
        assert!(summary.contains("24"));
        assert!(summary.contains("Audio tracks: 1"));
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = ScriptParser::parse_json(Path::new("/nonexistent/file.json"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read script file"));
    }
}
