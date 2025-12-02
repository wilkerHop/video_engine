use crate::script::VideoScript;
use crate::AssetLoader;
use anyhow::Result;
use std::path::Path;

pub struct PerformanceContext;

impl PerformanceContext {
    pub fn run(
        script: &VideoScript,
        loader: &mut AssetLoader,
        output_dir: &Path,
        use_blender: bool,
        use_gpu: bool,
    ) -> Result<()> {
        // 1. Rendering
        println!("\n🎬 Rendering frames...");

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        if use_blender {
            println!("🎨 Using Blender Backend");
            let renderer =
                crate::renderer::BlenderRenderer::new(script.clone(), output_dir.to_path_buf());
            renderer.render()?;
        } else {
            println!("🎨 Using Native Engine (CPU/GPU)");
            let mut engine = crate::renderer::RenderEngine::new(script.clone(), use_gpu);
            engine.render(output_dir, loader)?;
        }

        // 2. Audio Processing
        let mut audio_path_opt = None;
        if let Some(audio_config) = &script.audio {
            println!("\n🎵 Processing audio...");
            let mut mixer = crate::AudioMixer::new(44100, 2);

            for track in &audio_config.tracks {
                println!("  Loading track: {}", track.source.display());
                // Resolve path relative to script (using loader's base path would be better, but script paths are relative to script file)
                // We need the base path here. Loader has it.
                let base_path = loader.base_path();
                let track_path = if track.source.is_absolute() {
                    track.source.clone()
                } else {
                    base_path.join(&track.source)
                };

                match crate::AudioDecoder::decode(&track_path) {
                    Ok((samples, rate, channels)) => {
                        mixer.add_track(samples, rate, channels, track.start_time, track.volume);
                    }
                    Err(e) => println!("  ⚠️  Failed to load audio track: {}", e),
                }
            }

            let mixed_audio = mixer.mix(script.metadata.duration);
            let output_audio = output_dir.join("audio.wav");
            if let Err(e) = mixer.export(&output_audio, &mixed_audio) {
                println!("  ⚠️  Failed to export mixed audio: {}", e);
            } else {
                println!("  ✓ Mixed audio exported to: {}", output_audio.display());
                audio_path_opt = Some(output_audio);
            }
        }

        // 3. Video Encoding
        if crate::renderer::VideoEncoder::is_available() {
            let output_video = Path::new("output.mp4");
            let frame_pattern = if use_blender {
                output_dir.join("frame_%04d.png")
            } else {
                output_dir.join("frame_%d.ppm")
            };

            crate::renderer::VideoEncoder::encode(
                frame_pattern.to_str().unwrap(),
                output_video,
                script.metadata.fps,
                script.metadata.resolution.dimensions().0,
                script.metadata.resolution.dimensions().1,
                audio_path_opt.as_deref(),
            )?;

            println!("✨ Video created successfully: {}", output_video.display());
        } else {
            println!("⚠️  FFmpeg not found. Skipping video encoding.");
            println!("   Frames are saved in: {}", output_dir.display());
            println!("\n💡 To enable video generation, install FFmpeg:");
            if cfg!(target_os = "macos") {
                println!("   brew install ffmpeg");
            } else if cfg!(target_os = "windows") {
                println!("   choco install ffmpeg");
            } else if cfg!(target_os = "linux") {
                println!("   sudo apt-get install ffmpeg");
            } else {
                println!("   Install FFmpeg from https://ffmpeg.org/download.html");
            }
        }

        Ok(())
    }
}
