pub mod analysis;
pub mod assets;
pub mod audio;
pub mod config;
pub mod context;
pub mod parser;
pub mod renderer;
pub mod script;
pub mod templates;

pub use assets::AssetLoader;
pub use audio::{AudioDecoder, AudioMixer};
pub use parser::ScriptParser;
pub use renderer::{Compositor, FrameBuffer, RenderEngine, Timeline};
pub use script::VideoScript;
