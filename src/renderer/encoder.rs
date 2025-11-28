use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Handles video encoding using external FFmpeg process
pub struct VideoEncoder;

impl VideoEncoder {
    /// Check if FFmpeg is available
    pub fn is_available() -> bool {
        Command::new("ffmpeg").arg("-version").output().is_ok()
    }

    /// Encode a sequence of frames to a video file
    ///
    /// # Arguments
    /// * `frame_pattern` - Pattern for input frames (e.g., "output/frame_%d.ppm")
    /// * `output_path` - Path for the output video (e.g., "output.mp4")
    /// * `fps` - Frames per second
    /// * `width` - Video width
    /// * `height` - Video height
    pub fn encode(
        frame_pattern: &str,
        output_path: &Path,
        fps: u32,
        width: u32,
        height: u32,
        audio_path: Option<&Path>,
    ) -> Result<()> {
        if !Self::is_available() {
            anyhow::bail!("FFmpeg not found. Please install ffmpeg to enable video encoding.");
        }

        println!("🎥 Encoding video to {}...", output_path.display());

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y") // Overwrite output
            .arg("-f")
            .arg("image2") // Input format
            .arg("-framerate")
            .arg(fps.to_string())
            .arg("-i")
            .arg(frame_pattern);

        if let Some(audio) = audio_path {
            cmd.arg("-i").arg(audio);
        }

        cmd.arg("-c:v")
            .arg("libx264") // Video codec
            .arg("-pix_fmt")
            .arg("yuv420p") // Pixel format for compatibility
            .arg("-s")
            .arg(format!("{}x{}", width, height));

        if audio_path.is_some() {
            cmd.arg("-c:a")
                .arg("aac") // Audio codec
                .arg("-shortest"); // Finish when shortest stream ends (video)
        }

        let status = cmd
            .arg(output_path)
            .status()
            .context("Failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("FFmpeg encoding failed");
        }

        Ok(())
    }
}
