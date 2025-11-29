use crate::assets::AssetLoader;
use crate::renderer::{Compositor, FrameBuffer, GpuRenderer, Timeline};
use crate::script::{Layer, VideoScript};
use anyhow::Result;

/// Main rendering engine
pub struct RenderEngine {
    script: VideoScript,
    timeline: Timeline,
    frame_buffer: FrameBuffer,
    #[allow(dead_code)]
    gpu_renderer: Option<GpuRenderer>,
}

impl RenderEngine {
    /// Create new render engine from script
    pub fn new(script: VideoScript) -> Self {
        let (width, height) = script.metadata.resolution.dimensions();
        let timeline = Timeline::from_script(&script);
        let frame_buffer = FrameBuffer::new(width, height);

        // Try to initialize GPU renderer (optional - falls back to CPU if fails)
        let gpu_renderer = pollster::block_on(async { GpuRenderer::new(width, height).await.ok() });

        if gpu_renderer.is_some() {
            println!("✨ GPU renderer initialized successfully");
        } else {
            println!("ℹ️  Using CPU rendering (GPU unavailable or initialization failed)");
        }

        Self {
            script,
            timeline,
            frame_buffer,
            gpu_renderer,
        }
    }

    /// Render a single frame
    pub fn render_frame(
        &mut self,
        frame_number: u32,
        _asset_loader: &mut AssetLoader,
    ) -> Result<()> {
        // Clear frame
        self.frame_buffer.clear([0, 0, 0, 255]);

        // Get current scene ID
        if let Some(scene_id) = self.timeline.get_scene_at_frame(frame_number) {
            // Clone the scene ID to avoid borrow checker issues
            let scene_id = scene_id.to_string();

            // Find and render the scene
            if let Some(scene) = self.script.scenes.iter().find(|s| s.id == scene_id) {
                // Collect layers to avoid borrowing issues
                let layers: Vec<_> = scene.layers.clone();

                // Render each layer
                for layer in &layers {
                    self.render_layer(layer)?;
                }
            }
        }

        Ok(())
    }

    /// Render a single layer
    fn render_layer(&mut self, layer: &Layer) -> Result<()> {
        match layer {
            Layer::Image { transform, .. } => {
                // Placeholder: draw colored rectangle for image
                let (x, y) = Compositor::apply_transform(0, 0, transform);
                Compositor::fill_rect(&mut self.frame_buffer, x, y, 100, 100, [100, 100, 200, 255]);
            }
            Layer::Video { transform, .. } => {
                // Placeholder: draw colored rectangle for video
                let (x, y) = Compositor::apply_transform(0, 0, transform);
                Compositor::fill_rect(&mut self.frame_buffer, x, y, 100, 100, [200, 100, 100, 255]);
            }
            Layer::Text {
                content,
                position,
                color,
                ..
            } => {
                let rgba = [color.r, color.g, color.b, color.a];
                Compositor::draw_text_placeholder(
                    &mut self.frame_buffer,
                    content,
                    position.x,
                    position.y,
                    rgba,
                );
            }
        }

        Ok(())
    }

    /// Save current frame as PPM
    pub fn save_frame(&self, path: &str) -> Result<()> {
        self.frame_buffer.save_ppm(path)
    }

    /// Render all frames to the output directory
    pub fn render(
        &mut self,
        output_dir: &std::path::Path,
        asset_loader: &mut AssetLoader,
    ) -> Result<()> {
        let total_frames = self.timeline.total_frames();

        for frame in 0..total_frames {
            if frame % 30 == 0 {
                println!("  Rendering frame {}/{}", frame, total_frames);
            }

            self.render_frame(frame, asset_loader)?;

            let filename = format!("frame_{}.ppm", frame);
            let path = output_dir.join(filename);
            self.save_frame(path.to_str().unwrap())?;
        }

        Ok(())
    }

    /// Get timeline
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::*;
    use std::path::PathBuf;

    #[test]
    fn test_render_engine_creation() {
        let script = create_test_script();
        let engine = RenderEngine::new(script);
        assert_eq!(engine.timeline().total_frames(), 300);
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
            scenes: vec![Scene {
                id: "scene1".into(),
                duration: 10.0,
                layers: vec![Layer::Image {
                    source: PathBuf::from("test.png"),
                    effects: vec![],
                    transform: Default::default(),
                }],
                transition: None,
            }],
            audio: None,
        }
    }

    #[test]
    fn test_gpu_renderer_integration() {
        let script = create_test_script();
        let engine = RenderEngine::new(script);

        // Engine should be created successfully regardless of GPU availability
        assert_eq!(engine.timeline().total_frames(), 300);

        // GPU renderer field exists (even if None)
        // This test verifies the integration compiles and runs
    }
}
