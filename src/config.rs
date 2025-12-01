use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub renderer: RendererConfig,
    pub video: VideoConfig,
    pub assets: AssetsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RendererConfig {
    pub engine: String, // "native" or "blender"
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VideoConfig {
    pub default_resolution: String,
    pub default_fps: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AssetsConfig {
    pub base_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            renderer: RendererConfig {
                engine: "native".to_string(),
                output_dir: PathBuf::from("output"),
            },
            video: VideoConfig {
                default_resolution: "1920x1080".to_string(),
                default_fps: 30,
            },
            assets: AssetsConfig {
                base_path: PathBuf::from("."),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .set_default("renderer.engine", "native")?
            .set_default("renderer.output_dir", "output")?
            .set_default("video.default_resolution", "1920x1080")?
            .set_default("video.default_fps", 30)?
            .set_default("assets.base_path", ".")?
            // Load from file if exists
            .add_source(config::File::with_name("interstellar").required(false))
            // Allow env var overrides (e.g. INTERSTELLAR_RENDERER__ENGINE=blender)
            .add_source(config::Environment::with_prefix("INTERSTELLAR").separator("__"));

        builder.build()?.try_deserialize()
    }
}
