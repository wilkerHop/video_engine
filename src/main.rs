use anyhow::Result;
use clap::{Parser, Subcommand};
use interstellar_triangulum::config::AppConfig;
use interstellar_triangulum::templates::{ScriptTemplate, TemplateType};
use interstellar_triangulum::{AssetLoader, ScriptParser};
use std::path::Path;

#[derive(Parser)]
#[command(name = "interstellar-triangulum")]
#[command(about = "Digital Artisan Video Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a script to video
    Render {
        /// Path to the script file
        #[arg(value_name = "SCRIPT")]
        script: String,

        /// Renderer engine to use
        #[arg(long)]
        renderer: Option<String>,

        /// Output directory
        #[arg(long)]
        output: Option<String>,

        /// Export analysis report
        #[arg(long)]
        export_report: Option<String>,

        /// Fail on low narrative score
        #[arg(long)]
        fail_on_low_score: Option<u32>,

        /// Force CPU rendering (disable GPU)
        #[arg(long)]
        force_cpu: bool,
    },

    /// Validate script without rendering
    Validate {
        /// Path to the script file
        #[arg(value_name = "SCRIPT")]
        script: String,

        /// Fail on warnings
        #[arg(long)]
        fail_on_warnings: bool,
    },

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

    /// Clean output and cache directories
    Clean,
}

fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load().unwrap_or_default();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Template {
            template_type,
            duration,
        }) => {
            let script = ScriptTemplate::generate(template_type, duration);
            println!("{}", serde_json::to_string_pretty(&script)?);
        }
        Some(Commands::Clean) => {
            let output_dir = &config.renderer.output_dir;
            let cache_dir = Path::new(".cache");
            if output_dir.exists() {
                std::fs::remove_dir_all(output_dir)?;
                println!("🗑️  Cleaned output directory: {}", output_dir.display());
            }
            if cache_dir.exists() {
                std::fs::remove_dir_all(cache_dir)?;
                println!("🗑️  Cleaned cache directory: {}", cache_dir.display());
            }
        }
        Some(Commands::Validate {
            script,
            fail_on_warnings,
        }) => {
            run_validation(&script, fail_on_warnings)?;
        }
        Some(Commands::Render {
            script,
            renderer,
            output,
            export_report,
            fail_on_low_score,
            force_cpu,
        }) => {
            let renderer_engine = renderer.unwrap_or(config.renderer.engine.clone());
            let output_dir = output
                .map(std::path::PathBuf::from)
                .unwrap_or(config.renderer.output_dir.clone());

            run_render(
                &script,
                &renderer_engine,
                &output_dir,
                export_report,
                fail_on_low_score,
                force_cpu,
            )?;
        }
        None => {
            // Default behavior if no subcommand: try to render examples/simple.json
            // This preserves backward compatibility for "cargo run" without args if we wanted,
            // but strictly speaking "cargo run" passes no args.
            // Let's print help instead.
            use clap::CommandFactory;
            Cli::command().print_help()?;
        }
    }

    Ok(())
}

fn run_validation(script_path: &str, fail_on_warnings: bool) -> Result<()> {
    let script_path = Path::new(script_path);
    println!("🔍 Validating script: {}", script_path.display());

    let script = ScriptParser::parse_json(script_path)?;
    println!("\n📋 Script Summary:");
    println!("{}", ScriptParser::summarize(&script));

    // Run Analysis
    let narrative_report =
        interstellar_triangulum::context::narrative::NarrativeContext::run(&script);
    let credibility_report =
        interstellar_triangulum::context::credibility::CredibilityContext::run(&script);

    if fail_on_warnings {
        let has_warnings = !narrative_report.structure_valid
            || !narrative_report.structure_recommendations.is_empty()
            || !narrative_report.pacing_alerts.is_empty()
            || !narrative_report.retention_warnings.is_empty()
            || credibility_report.score < 100; // Strict check

        if has_warnings {
            eprintln!("\n❌ Validation failed due to warnings (strict mode).");
            std::process::exit(1);
        }
    }

    println!("\n✅ Validation complete.");
    Ok(())
}

fn run_render(
    script_path: &str,
    renderer_engine: &str,
    output_dir: &Path,
    export_report: Option<String>,
    fail_on_low_score: Option<u32>,
    force_cpu: bool,
) -> Result<()> {
    let script_path = Path::new(script_path);
    println!("🎬 Video Engine - Digital Artisan PoC\n");
    println!("Parsing script: {}", script_path.display());

    let script = ScriptParser::parse_json(script_path)?;

    println!("\n📋 Script Summary:");
    println!("{}", ScriptParser::summarize(&script));

    // Load assets
    let base_path = script_path.parent().unwrap_or_else(|| Path::new("."));
    let mut loader = AssetLoader::new(base_path);

    // Pillar 2: Narrative (Engaging)
    let narrative_report =
        interstellar_triangulum::context::narrative::NarrativeContext::run(&script);

    // Pillar 3: Credibility (Trustworthy)
    interstellar_triangulum::context::credibility::CredibilityContext::run(&script);

    // Export Report
    if let Some(path) = export_report {
        let path = Path::new(&path);
        let content = if path.extension().is_some_and(|ext| ext == "json") {
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
                md.push_str(&format!(
                    "- **[{:?}]** {}: {}\n",
                    rec.severity, rec.category, rec.message
                ));
            }
            md
        };
        std::fs::write(path, content)?;
        println!("\n📄 Report exported to: {}", path.display());
    }

    // Fail on low score
    if let Some(threshold) = fail_on_low_score {
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

    let use_blender = renderer_engine == "blender";
    let use_gpu = !force_cpu;

    interstellar_triangulum::context::performance::PerformanceContext::run(
        &script,
        &mut loader,
        output_dir,
        use_blender,
        use_gpu,
    )?;

    println!("\n📊 Asset Statistics:");
    println!("  {}", loader.stats());

    Ok(())
}
