use anyhow::Result;
use interstellar_triangulum::{AssetLoader, ScriptParser};
use std::path::Path;

fn main() -> Result<()> {
    println!("🎬 Video Engine - Digital Artisan PoC\n");

    // Parse script file from args or use default
    let args: Vec<String> = std::env::args().collect();
    let default_path = "examples/simple.json".to_string();
    let script_path = args.get(1).unwrap_or(&default_path);
    let example_script = Path::new(script_path);

    if example_script.exists() {
        println!("Parsing script: {}", example_script.display());
        let script = ScriptParser::parse_json(example_script)?;

        println!("\n📋 Script Summary:");
        println!("{}", ScriptParser::summarize(&script));

        // Load assets
        let base_path = example_script.parent().unwrap_or_else(|| Path::new("."));
        let mut loader = AssetLoader::new(base_path);

        println!("\n🎨 Loading assets...");
        for scene in &script.scenes {
            for layer in &scene.layers {
                match layer {
                    interstellar_triangulum::script::Layer::Image { source, .. } => {
                        match loader.load_image(source) {
                            Ok(img) => println!(
                                "  ✓ Loaded image: {} ({}x{})",
                                source.display(),
                                img.width,
                                img.height
                            ),
                            Err(e) => {
                                println!("  ✗ Failed to load image {}: {}", source.display(), e)
                            }
                        }
                    }
                    interstellar_triangulum::script::Layer::Video { source, .. } => {
                        match loader.load_video(source) {
                            Ok(vid) => println!(
                                "  ✓ Loaded video: {} ({}x{}@{}fps)",
                                source.display(),
                                vid.width,
                                vid.height,
                                vid.fps
                            ),
                            Err(e) => {
                                println!("  ✗ Failed to load video {}: {}", source.display(), e)
                            }
                        }
                    }
                    interstellar_triangulum::script::Layer::Text { font, .. } => {
                        match loader.load_font(font) {
                            Ok(_) => println!("  ✓ Loaded font: {}", font.display()),
                            Err(e) => println!("  ✗ Failed to load font {}: {}", font.display(), e),
                        }
                    }
                }
            }
        }

        // Render frames
        println!("\n🎬 Rendering frames...");
        let output_dir = Path::new("output");
        if !output_dir.exists() {
            std::fs::create_dir(output_dir)?;
        }

        let mut engine = interstellar_triangulum::renderer::RenderEngine::new(script.clone());
        engine.render(output_dir, &mut loader)?;

        // Process Audio
        let mut audio_path_opt = None;
        if let Some(audio_config) = &script.audio {
            println!("\n🎵 Processing audio...");
            let mut mixer = interstellar_triangulum::AudioMixer::new(44100, 2);

            for track in &audio_config.tracks {
                println!("  Loading track: {}", track.source.display());
                // Resolve path relative to script
                let track_path = if track.source.is_absolute() {
                    track.source.clone()
                } else {
                    base_path.join(&track.source)
                };

                match interstellar_triangulum::AudioDecoder::decode(&track_path) {
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

        // Encode video if FFmpeg is available
        if interstellar_triangulum::renderer::VideoEncoder::is_available() {
            let output_video = Path::new("output.mp4");
            let frame_pattern = output_dir.join("frame_%d.ppm");

            interstellar_triangulum::renderer::VideoEncoder::encode(
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
        }

        println!("\n📊 Asset Statistics:");
        println!("  {}", loader.stats());
    } else {
        println!(
            "ℹ️  No example script found at {}",
            example_script.display()
        );
        println!("   Create an example script to test the engine.");
        println!("\n💡 See examples/simple.json for reference format");
    }

    Ok(())
}
