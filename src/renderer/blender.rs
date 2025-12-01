use crate::script::{Layer, VideoScript};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

pub struct BlenderRenderer {
    script: VideoScript,
    output_dir: PathBuf,
    cache_dir: PathBuf,
    parallel_jobs: usize,
}

impl BlenderRenderer {
    pub fn new(script: VideoScript, output_dir: PathBuf) -> Self {
        let cache_dir = PathBuf::from(".cache/blender");
        let parallel_jobs = std::cmp::min(num_cpus::get(), 2).max(1);
        Self {
            script,
            output_dir,
            cache_dir,
            parallel_jobs,
        }
    }

    /// Generate the Python script for Blender
    fn generate_python_script(&self, start_frame: u32, end_frame: u32) -> String {
        let mut py = String::new();

        // Imports and setup
        py.push_str("import bpy\n");
        py.push_str("import math\n\n");

        // Clear existing scene
        py.push_str("# Clear scene\n");
        py.push_str("bpy.ops.wm.read_factory_settings(use_empty=True)\n\n");

        // Render settings
        let (width, height) = self.script.metadata.resolution.dimensions();
        py.push_str("scene = bpy.context.scene\n");
        py.push_str(&format!("scene.render.resolution_x = {}\n", width));
        py.push_str(&format!("scene.render.resolution_y = {}\n", height));
        py.push_str(&format!(
            "scene.render.fps = {}\n",
            self.script.metadata.fps
        ));

        // Parse args for start/end frame override
        py.push_str("import sys\n");
        py.push_str("argv = sys.argv\n");
        py.push_str("if \"--\" in argv:\n");
        py.push_str("    args = argv[argv.index(\"--\") + 1:]\n");
        py.push_str("    if \"--start\" in args:\n");
        py.push_str("        scene.frame_start = int(args[args.index(\"--start\") + 1])\n");
        py.push_str("    if \"--end\" in args:\n");
        py.push_str("        scene.frame_end = int(args[args.index(\"--end\") + 1])\n");
        py.push_str("else:\n");
        py.push_str(&format!("    scene.frame_start = {}\n", start_frame));
        py.push_str(&format!("    scene.frame_end = {}\n", end_frame));

        py.push_str("scene.render.image_settings.file_format = 'PNG'\n");

        // Camera setup
        py.push_str("\n# Camera setup\n");
        py.push_str("cam_data = bpy.data.cameras.new(name='Camera')\n");
        py.push_str("cam_obj = bpy.data.objects.new(name='Camera', object_data=cam_data)\n");
        py.push_str("scene.collection.objects.link(cam_obj)\n");
        py.push_str("scene.camera = cam_obj\n");
        py.push_str("cam_obj.location = (0, 0, 10)\n"); // Orthographic setup usually needs specific placement
        py.push_str("cam_data.type = 'ORTHO'\n");
        py.push_str(&format!(
            "cam_data.ortho_scale = {}\n",
            width as f32 / 100.0
        )); // Adjust scale mapping

        // Process scenes and layers
        let mut _current_frame = 0;
        for scene in &self.script.scenes {
            let scene_duration_frames = (scene.duration * self.script.metadata.fps as f32) as u32;

            for (layer_idx, layer) in scene.layers.iter().enumerate() {
                match layer {
                    Layer::Image {
                        source: _source,
                        transform,
                        ..
                    } => {
                        let name = format!("Image_{}_{}", scene.id, layer_idx);
                        py.push_str(&format!("\n# Layer: {}\n", name));
                        // In a real implementation, we'd load the image to a plane
                        // For now, creating a placeholder plane
                        py.push_str("bpy.ops.mesh.primitive_plane_add(size=1)\n");
                        py.push_str("obj = bpy.context.active_object\n");
                        py.push_str(&format!("obj.name = '{}'\n", name));

                        // Apply transform (simplified)
                        // Blender coords are different, would need mapping
                        py.push_str(&format!(
                            "obj.location.x = {}\n",
                            transform.position.x as f32 / 100.0
                        ));
                        py.push_str(&format!(
                            "obj.location.y = {}\n",
                            transform.position.y as f32 / 100.0
                        ));
                    }
                    Layer::Text { content, .. } => {
                        let name = format!("Text_{}_{}", scene.id, layer_idx);
                        py.push_str(&format!("\n# Layer: {}\n", name));
                        py.push_str("bpy.ops.object.text_add()\n");
                        py.push_str("obj = bpy.context.active_object\n");
                        py.push_str(&format!("obj.name = '{}'\n", name));
                        py.push_str(&format!("obj.data.body = '{}'\n", content));
                    }
                    _ => {}
                }
            }
            _current_frame += scene_duration_frames;
        }

        py.push_str("\n# Render animation\n");
        py.push_str("bpy.ops.render.render(animation=True)\n");

        py
    }

    /// Calculate hash of the generation logic/script
    fn calculate_hash(&self, python_script: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(python_script);
        format!("{:x}", hasher.finalize())
    }

    /// Render the video using Blender
    pub fn render(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.output_dir)?;

        let total_frames = (self.script.metadata.duration * self.script.metadata.fps as f32) as u32;
        let python_script = self.generate_python_script(0, total_frames);
        let script_hash = self.calculate_hash(&python_script);

        let cache_file = self.cache_dir.join(format!("{}.py", script_hash));
        let hash_file = self.cache_dir.join("last_render.sha256");

        // Check cache
        if cache_file.exists() && hash_file.exists() {
            let last_hash = fs::read_to_string(&hash_file).unwrap_or_default();
            if last_hash.trim() == script_hash {
                println!("✨ Cache hit! Skipping Blender rendering.");
                return Ok(());
            }
        }

        println!("🎨 Starting Blender rendering...");

        // Write script to file
        fs::write(&cache_file, &python_script)?;

        println!(
            "🚀 Launching {} parallel Blender jobs...",
            self.parallel_jobs
        );

        let frames_per_job = (total_frames as f32 / self.parallel_jobs as f32).ceil() as u32;
        let mut handles = vec![];
        let completed_frames = Arc::new(Mutex::new(0));
        let start_time = Instant::now();

        // Safety Vault: Memory Monitor
        let _monitor_handle = thread::spawn(|| {
            let mut sys = System::new_all();
            loop {
                sys.refresh_memory();
                let used_memory = sys.used_memory();
                let total_memory = sys.total_memory();
                let usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

                if usage_percent > 99.0 {
                    eprintln!(
                        "🚨 CRITICAL: Memory usage at {:.1}%! Killing process to prevent crash.",
                        usage_percent
                    );
                    std::process::exit(1);
                }

                thread::sleep(Duration::from_secs(1));
            }
        });

        for i in 0..self.parallel_jobs {
            let start = i as u32 * frames_per_job;
            let end = ((i + 1) as u32 * frames_per_job).min(total_frames);

            if start >= end {
                break;
            }

            let cache_file = cache_file.clone();
            let output_dir = self.output_dir.clone();
            let completed = Arc::clone(&completed_frames);

            let handle = thread::spawn(move || -> Result<()> {
                let mut child = Command::new("blender")
                    .arg("-b")
                    .arg("-P")
                    .arg(&cache_file)
                    .arg("--")
                    .arg("--start")
                    .arg(start.to_string())
                    .arg("--end")
                    .arg(end.to_string())
                    .arg("--output")
                    .arg(output_dir.join("frame_").to_str().unwrap())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .context("Failed to spawn Blender process")?;

                // Monitor progress
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        if line.contains("Saved:") {
                            let mut count = completed.lock().unwrap();
                            *count += 1;
                        }
                    }
                }

                let status = child.wait()?;
                if !status.success() {
                    anyhow::bail!("Blender job failed");
                }
                Ok(())
            });
            handles.push(handle);
        }

        // Wait for all jobs
        let mut success = true;
        for handle in handles {
            if let Err(e) = handle.join().unwrap() {
                println!("❌ Job failed: {}", e);
                success = false;
            }
        }

        if success {
            // Update cache
            fs::write(&hash_file, &script_hash)?;
            let duration = start_time.elapsed();
            println!(
                "✅ Blender rendering complete in {:.2}s",
                duration.as_secs_f32()
            );
        } else {
            anyhow::bail!("One or more Blender jobs failed");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Layer, Metadata, Resolution, Scene};

    #[test]
    fn test_generate_python_script() {
        let script = VideoScript {
            metadata: Metadata {
                title: "Test".into(),
                resolution: Resolution::Named("1920x1080".into()),
                fps: 30,
                duration: 5.0,
                description: None,
            },
            scenes: vec![Scene {
                id: "scene1".into(),
                duration: 5.0,
                layers: vec![],
                transition: None,
            }],
            audio: None,
        };

        let renderer = BlenderRenderer::new(script, PathBuf::from("output"));
        let py_script = renderer.generate_python_script(0, 150);

        assert!(py_script.contains("import bpy"));
        assert!(py_script.contains("scene.render.resolution_x = 1920"));
        assert!(py_script.contains("scene.render.resolution_y = 1080"));
        assert!(py_script.contains("scene.frame_end = 150"));
    }
}
