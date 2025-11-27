use anyhow::Result;
use interstellar_triangulum::{AssetLoader, ScriptParser};
use std::path::Path;

fn main() -> Result<()> {
    println!("🎬 Video Engine - Digital Artisan PoC\n");

    // Example: Parse a script file
    let example_script = Path::new("examples/simple.json");

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

        println!("\n📊 Asset Statistics:");
        println!("  {}", loader.stats());
    } else {
        println!("ℹ️  No example script found at {}", example_script.display());
        println!("   Create an example script to test the engine.");
        println!("\n💡 See examples/simple.json for reference format");
    }

    Ok(())
}
