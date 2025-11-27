use crate::script::VideoScript;

/// Timeline for managing scene playback
pub struct Timeline {
    fps: u32,
    total_frames: u32,
    scenes: Vec<SceneSegment>,
}

#[derive(Debug, Clone)]
struct SceneSegment {
    scene_id: String,
    start_frame: u32,
    end_frame: u32,
}

impl Timeline {
    /// Create timeline from video script
    pub fn from_script(script: &VideoScript) -> Self {
        let fps = script.metadata.fps;
        let total_duration = script.metadata.duration;
        let total_frames = (total_duration * fps as f32) as u32;

        let mut segments = Vec::new();
        let mut current_frame = 0;

        for scene in &script.scenes {
            let scene_frames = (scene.duration * fps as f32) as u32;
            segments.push(SceneSegment {
                scene_id: scene.id.clone(),
                start_frame: current_frame,
                end_frame: current_frame + scene_frames,
            });
            current_frame += scene_frames;
        }

        Self {
            fps,
            total_frames,
            scenes: segments,
        }
    }

    /// Get scene at given frame number
    pub fn get_scene_at_frame(&self, frame: u32) -> Option<&str> {
        for segment in &self.scenes {
            if frame >= segment.start_frame && frame < segment.end_frame {
                return Some(&segment.scene_id);
            }
        }
        None
    }

    /// Get total frame count
    pub fn total_frames(&self) -> u32 {
        self.total_frames
    }

    /// Get FPS
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Convert frame number to time in seconds
    pub fn frame_to_time(&self, frame: u32) -> f32 {
        frame as f32 / self.fps as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Layer, Metadata, Resolution, Scene};
    use std::path::PathBuf;

    #[test]
    fn test_timeline_creation() {
        let script = create_test_script();
        let timeline = Timeline::from_script(&script);

        assert_eq!(timeline.fps(), 30);
        assert_eq!(timeline.total_frames(), 300); // 10 seconds at 30fps
    }

    #[test]
    fn test_get_scene_at_frame() {
        let script = create_test_script();
        let timeline = Timeline::from_script(&script);

        // First scene: 0-150 frames (5 seconds)
        assert_eq!(timeline.get_scene_at_frame(0), Some("scene1"));
        assert_eq!(timeline.get_scene_at_frame(100), Some("scene1"));

        // Second scene: 150-300 frames
        assert_eq!(timeline.get_scene_at_frame(150), Some("scene2"));
        assert_eq!(timeline.get_scene_at_frame(200), Some("scene2"));
    }

    fn create_test_script() -> VideoScript {
        VideoScript {
            metadata: Metadata {
                title: "Test".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: 10.0,
                description: None,
            },
            scenes: vec![
                Scene {
                    id: "scene1".into(),
                    duration: 5.0,
                    layers: vec![Layer::Image {
                        source: PathBuf::from("test.png"),
                        effects: vec![],
                        transform: Default::default(),
                    }],
                    transition: None,
                },
                Scene {
                    id: "scene2".into(),
                    duration: 5.0,
                    layers: vec![Layer::Image {
                        source: PathBuf::from("test2.png"),
                        effects: vec![],
                        transform: Default::default(),
                    }],
                    transition: None,
                },
            ],
            audio: None,
        }
    }
}
