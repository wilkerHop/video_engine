use crate::assets::AssetLoader;
use crate::renderer::{Compositor, FrameBuffer, GpuRenderer, Timeline};
use crate::script::{Layer, VideoScript};
use anyhow::Result;
use image::GenericImageView;

/// Main rendering engine
pub struct RenderEngine {
    script: VideoScript,
    timeline: Timeline,
    frame_buffer: FrameBuffer,
    #[allow(dead_code)]
    gpu_renderer: Option<GpuRenderer>,
    texture_cache:
        std::collections::HashMap<std::path::PathBuf, (std::sync::Arc<wgpu::BindGroup>, u32, u32)>,
}

impl RenderEngine {
    /// Create new render engine from script
    pub fn new(script: VideoScript, use_gpu: bool) -> Self {
        let (width, height) = script.metadata.resolution.dimensions();
        let timeline = Timeline::from_script(&script);
        let frame_buffer = FrameBuffer::new(width, height);

        // Try to initialize GPU renderer (optional - falls back to CPU if fails)
        let gpu_renderer = if use_gpu {
            pollster::block_on(async { GpuRenderer::new(width, height).await.ok() })
        } else {
            None
        };

        if gpu_renderer.is_some() {
            println!("✨ GPU renderer initialized successfully");
        } else {
            println!(
                "ℹ️  Using CPU rendering (GPU unavailable, initialization failed, or disabled)"
            );
        }

        Self {
            script,
            timeline,
            frame_buffer,
            gpu_renderer,
            texture_cache: std::collections::HashMap::new(),
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
                    self.render_layer(layer, _asset_loader)?;
                }

                // Flush GPU commands after rendering all layers
                self.flush_gpu()?;
            }
        }

        Ok(())
    }

    /// Flush GPU commands if available
    fn flush_gpu(&mut self) -> Result<()> {
        if let Some(gpu) = &mut self.gpu_renderer {
            gpu.flush(&mut self.frame_buffer)?;
        }
        Ok(())
    }

    /// Render a single layer
    fn render_layer(&mut self, layer: &Layer, asset_loader: &AssetLoader) -> Result<()> {
        match layer {
            Layer::Image {
                source, transform, ..
            } => {
                let (x, y) = Compositor::apply_transform(0, 0, transform);
                let color = [255, 255, 255, 255];

                if let Some(gpu) = &mut self.gpu_renderer {
                    // Load texture if not in cache
                    if !self.texture_cache.contains_key(source) {
                        // Resolve path using AssetLoader (hacky access to private method or just join)
                        // AssetLoader::resolve_path is private. But we can use base_path.
                        let full_path = if source.is_absolute() {
                            source.clone()
                        } else {
                            asset_loader.base_path().join(source)
                        };

                        if full_path.exists() {
                            if let Ok(img) = image::open(&full_path) {
                                let dims = img.dimensions();
                                let bind_group = gpu.create_texture(&img);
                                self.texture_cache
                                    .insert(source.clone(), (bind_group, dims.0, dims.1));
                            } else {
                                println!(
                                    "Failed to load image for texture: {}",
                                    full_path.display()
                                );
                            }
                        }
                    }

                    if let Some((bind_group, w, h)) = self.texture_cache.get(source) {
                        // Apply scale from transform
                        let scale = transform.scale;
                        let draw_w = (*w as f32 * scale) as u32;
                        let draw_h = (*h as f32 * scale) as u32;

                        gpu.draw_texture(bind_group.clone(), x, y, draw_w, draw_h, color)?;
                    } else {
                        // Fallback to colored rect if texture failed
                        gpu.fill_rect(
                            &mut self.frame_buffer,
                            x,
                            y,
                            100,
                            100,
                            [100, 100, 200, 255],
                        )?;
                    }
                } else {
                    Compositor::fill_rect(
                        &mut self.frame_buffer,
                        x,
                        y,
                        100,
                        100,
                        [100, 100, 200, 255],
                    );
                }
            }
            Layer::Video { transform, .. } => {
                // Placeholder: draw colored rectangle for video
                let (x, y) = Compositor::apply_transform(0, 0, transform);
                let color = [200, 100, 100, 255];

                if let Some(gpu) = &self.gpu_renderer {
                    gpu.fill_rect(&mut self.frame_buffer, x, y, 100, 100, color)?;
                } else {
                    Compositor::fill_rect(&mut self.frame_buffer, x, y, 100, 100, color);
                }
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
        let engine = RenderEngine::new(script, false); // Default to CPU for basic test
        assert_eq!(engine.timeline().total_frames(), 600);
    }

    fn create_test_script() -> VideoScript {
        VideoScript {
            metadata: Metadata {
                title: "Test".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 60,
                duration: 10.0,
                description: None,
                citations: vec![],
            },
            scenes: vec![Scene {
                id: "test".into(),
                duration: 5.0,
                scene_type: Default::default(),
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
        let engine = RenderEngine::new(script, true); // Try GPU

        // Engine should be created successfully regardless of GPU availability
        assert_eq!(engine.timeline().total_frames(), 600);

        // GPU renderer field exists (even if None)
        // This test verifies the integration compiles and runs
    }

    #[test]
    fn test_render_frame_with_gpu() {
        let script = create_test_script();
        let mut engine = RenderEngine::new(script, true); // Try GPU
        let mut asset_loader = AssetLoader::new(".");

        // Should not panic even if GPU is not available (fallback to CPU)
        // If GPU is available, it exercises the flush() logic
        engine.render_frame(0, &mut asset_loader).unwrap();
    }
}
