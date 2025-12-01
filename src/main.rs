use anyhow::Result;
use clap::{Parser, Subcommand};
use interstellar_triangulum::templates::{ScriptTemplate, TemplateType};
use interstellar_triangulum::{AssetLoader, ScriptParser};
use std::path::Path;

#[derive(Parser)]
#[command(name = "interstellar-triangulum")]
#[command(about = "Digital Artisan Video Engine", long_about = None)]
struct Cli {
    /// Path to the script file to render
    #[arg(value_name = "SCRIPT")]
    script: Option<String>,

    /// Use Blender renderer (pass "blender")
    #[arg(long)]
    renderer: Option<String>,

    /// Export analysis report to file (supports .json, .md)
    #[arg(long)]
    export_report: Option<String>,

    /// Fail if narrative score is below threshold
    #[arg(long)]
    fail_on_low_score: Option<u32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a script template
    Template {
        /// Type of template to generate
        #[arg(value_enum)]
        #[arg(name = "type")]
        template_type: TemplateType,

        /// Total duration in seconds
        #[arg(short, long, default_value_t = 60.0)]
        duration: f32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Template {
        template_type,
        duration,
    }) = cli.command
    {
        let script = ScriptTemplate::generate(template_type, duration);
        println!("{}", serde_json::to_string_pretty(&script)?);
        return Ok(());
    }

    println!("🎬 Video Engine - Digital Artisan PoC\n");

    // Default behavior: Parse script
    let default_path = "examples/simple.json".to_string();
    let script_path = cli.script.unwrap_or(default_path);
    let example_script = Path::new(&script_path);

    if example_script.exists() {
        println!("Parsing script: {}", example_script.display());
        let script = ScriptParser::parse_json(example_script)?;

        println!("\n📋 Script Summary:");
        println!("{}", ScriptParser::summarize(&script));

        // Load assets
        let base_path = example_script.parent().unwrap_or_else(|| Path::new("."));
        let mut loader = AssetLoader::new(base_path);

        // Pillar 2: Narrative (Engaging)
        let narrative_report = interstellar_triangulum::context::narrative::NarrativeContext::run(&script);

        // Pillar 3: Credibility (Trustworthy)
        interstellar_triangulum::context::credibility::CredibilityContext::run(&script);

        // Export Report
        if let Some(path) = cli.export_report {
            let path = Path::new(&path);
            let content = if path.extension().map_or(false, |ext| ext == "json") {
                // JSON Export
                serde_json::to_string_pretty(&narrative_report)?
            } else {
                // Markdown Export
                let mut md = format!(
                    "# Narrative Analysis Report\n\n**Score**: {}/100\n\n## Structure\n- Valid: {}\n- Errors: {:?}\n\n## Recommendations\n",
                    narrative_report.score,
                    narrative_report.structure_valid,
                    narrative_report.structure_errors
                );

                for rec in &narrative_report.structure_recommendations {
                    md.push_str(&format!("- **[{:?}]** {}: {}\n", rec.severity, rec.category, rec.message));
                }
                md
            };
            std::fs::write(path, content)?;
            println!("\n📄 Report exported to: {}", path.display());
        }

        // Fail on low score
        if let Some(threshold) = cli.fail_on_low_score {
            if narrative_report.score < threshold {
                eprintln!(
                    "\n❌ Narrative score {} is below threshold {}",
                    narrative_report.score, threshold
                );
                std::process::exit(1);
            }
        }

        // Pillar 1: Performance (Fast) - Asset Loading & Rendering
        println!("\n🎨 Loading assets...");
        // Pre-load assets for statistics and validation
        for scene in &script.scenes {
            for layer in &scene.layers {
                match layer {
                    interstellar_triangulum::script::Layer::Image { source, .. } => {
                        if let Err(e) = loader.load_image(source) {
                            println!("  ✗ Failed to load image {}: {}", source.display(), e);
                        } else {
                            println!("  ✓ Loaded image: {}", source.display());
                        }
                    }
                    interstellar_triangulum::script::Layer::Video { source, .. } => {
                        if let Err(e) = loader.load_video(source) {
                            println!("  ✗ Failed to load video {}: {}", source.display(), e);
                        } else {
                            println!("  ✓ Loaded video: {}", source.display());
                        }
                    }
                    interstellar_triangulum::script::Layer::Text { font, .. } => {
                        if let Err(e) = loader.load_font(font) {
                            println!("  ✗ Failed to load font {}: {}", font.display(), e);
                        } else {
                            println!("  ✓ Loaded font: {}", font.display());
                        }
                    }
                }
            }
        }

        let output_dir = Path::new("output");
        let use_blender = cli.renderer.as_deref() == Some("blender");

        interstellar_triangulum::context::performance::PerformanceContext::run(
            &script,
            &mut loader,
            output_dir,
            use_blender,
        )?;

        println!("\n📊 Asset Statistics:");
        println!("  {}", loader.stats());
    } else {
        println!(
            "ℹ️  No example script found at {}",
            example_script.display()
        );
        println!("   Create an example script to test the engine.");
        println!("\n💡 See examples/simple.json for reference format");
        println!("   Or generate a template: interstellar-triangulum template explainer");
    }

    Ok(())
}
