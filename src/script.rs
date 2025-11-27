use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main video script structure that defines the entire video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoScript {
    pub metadata: Metadata,
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub audio: Option<AudioConfig>,
}

/// Video metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub resolution: Resolution,
    pub fps: u32,
    pub duration: f32,
    #[serde(default)]
    pub description: Option<String>,
}

/// Video resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Resolution {
    Named(String), // e.g., "1920x1080", "1280x720"
    Dimensions { width: u32, height: u32 },
}

impl Resolution {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Resolution::Named(s) => {
                let parts: Vec<&str> = s.split('x').collect();
                if parts.len() == 2 {
                    let width = parts[0].parse().unwrap_or(1920);
                    let height = parts[1].parse().unwrap_or(1080);
                    (width, height)
                } else {
                    (1920, 1080) // Default to 1080p
                }
            }
            Resolution::Dimensions { width, height } => (*width, *height),
        }
    }
}

/// A scene in the video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub duration: f32,
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub transition: Option<Transition>,
}

/// A layer within a scene (can be video, image, text, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Layer {
    #[serde(rename = "video")]
    Video {
        source: PathBuf,
        #[serde(default)]
        effects: Vec<Effect>,
        #[serde(default)]
        transform: Transform,
    },
    #[serde(rename = "image")]
    Image {
        source: PathBuf,
        #[serde(default)]
        effects: Vec<Effect>,
        #[serde(default)]
        transform: Transform,
    },
    #[serde(rename = "text")]
    Text {
        content: String,
        font: PathBuf,
        font_size: f32,
        color: Color,
        #[serde(default)]
        position: Position,
        #[serde(default)]
        effects: Vec<Effect>,
    },
}

/// Transform for positioning and scaling layers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transform {
    #[serde(default)]
    pub position: Position,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn default_opacity() -> f32 {
    1.0
}

/// Position in the frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(default = "default_alpha")]
    pub a: u8,
}

fn default_alpha() -> u8 {
    255
}

/// Visual effects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    FadeIn,
    FadeOut,
    Blur { radius: f32 },
    ColorGrade { adjustment: String },
}

/// Transition between scenes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transition {
    Cut,
    Fade { duration: f32 },
    Dissolve { duration: f32 },
    Wipe { duration: f32, direction: String },
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub tracks: Vec<AudioTrack>,
}

/// Individual audio track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub source: PathBuf,
    #[serde(default)]
    pub track_type: AudioTrackType,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub start_time: f32,
}

fn default_volume() -> f32 {
    1.0
}

/// Type of audio track
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioTrackType {
    #[default]
    Music,
    Voiceover,
    SoundEffect,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_parsing() {
        let res = Resolution::Named("1920x1080".to_string());
        assert_eq!(res.dimensions(), (1920, 1080));

        let res = Resolution::Dimensions {
            width: 1280,
            height: 720,
        };
        assert_eq!(res.dimensions(), (1280, 720));
    }

    #[test]
    fn test_script_deserialization() {
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
                    "duration": 5.0,
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

        let script: Result<VideoScript, _> = serde_json::from_str(json);
        assert!(script.is_ok());
        let script = script.unwrap();
        assert_eq!(script.metadata.title, "Test Video");
        assert_eq!(script.scenes.len(), 1);
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_scale(), 1.0);
        assert_eq!(default_opacity(), 1.0);
        assert_eq!(default_alpha(), 255);
        assert_eq!(default_volume(), 1.0);
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_resolution_invalid_format() {
        let res = Resolution::Named("invalid".to_string());
        assert_eq!(res.dimensions(), (1920, 1080)); // Should default

        let res = Resolution::Named("1920".to_string());
        assert_eq!(res.dimensions(), (1920, 1080)); // Should default

        let res = Resolution::Named("".to_string());
        assert_eq!(res.dimensions(), (1920, 1080)); // Should default
    }

    #[test]
    fn test_transform_defaults_serde() {
        // Test that serde uses our default functions
        let json = r#"{}"#;
        let transform: Transform = serde_json::from_str(json).unwrap();
        assert_eq!(transform.scale, 1.0); // Uses default_scale
        assert_eq!(transform.opacity, 1.0); // Uses default_opacity
        assert_eq!(transform.rotation, 0.0); // Uses serde default (0.0)
        assert_eq!(transform.position.x, 0); // Uses Position::default()
        assert_eq!(transform.position.y, 0);
    }

    #[test]
    fn test_layer_deserialization() {
        // Test Image layer
        let json = r#"{"type": "image", "source": "test.png"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        match layer {
            Layer::Image { .. } => (),
            _ => panic!("Expected Image layer"),
        }

        // Test Video layer
        let json = r#"{"type": "video", "source": "test.mp4"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        match layer {
            Layer::Video { .. } => (),
            _ => panic!("Expected Video layer"),
        }

        // Test Text layer
        let json = r#"{
            "type": "text",
            "content": "Hello",
            "font": "font.ttf",
            "font_size": 24.0,
            "color": {"r": 255, "g": 255, "b": 255}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        match layer {
            Layer::Text { .. } => (),
            _ => panic!("Expected Text layer"),
        }
    }

    #[test]
    fn test_audio_track_defaults() {
        let json = r#"{"source": "music.mp3"}"#;
        let track: AudioTrack = serde_json::from_str(json).unwrap();
        assert_eq!(track.volume, 1.0);
        assert_eq!(track.start_time, 0.0);
        match track.track_type {
            AudioTrackType::Music => (),
            _ => panic!("Expected Music as default track type"),
        }
    }
}
