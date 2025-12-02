use anyhow::Result;
use std::fs::File;
use std::io::Write;

/// RGBA frame buffer for rendering
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA, 4 bytes per pixel
}

impl FrameBuffer {
    /// Create new frame buffer with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Clear buffer with color
    pub fn clear(&mut self, color: [u8; 4]) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    /// Set pixel at position
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.pixels[idx..idx + 4].copy_from_slice(&color);
        }
    }

    /// Get pixel at position
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            let mut pixel = [0u8; 4];
            pixel.copy_from_slice(&self.pixels[idx..idx + 4]);
            Some(pixel)
        } else {
            None
        }
    }

    /// Alpha blend a color onto the buffer at position
    pub fn blend_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if let Some(bg) = self.get_pixel(x, y) {
            let alpha = color[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;

            let blended = [
                (color[0] as f32 * alpha + bg[0] as f32 * inv_alpha) as u8,
                (color[1] as f32 * alpha + bg[1] as f32 * inv_alpha) as u8,
                (color[2] as f32 * alpha + bg[2] as f32 * inv_alpha) as u8,
                255, // Output alpha is always opaque
            ];

            self.set_pixel(x, y, blended);
        }
    }

    /// Get buffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get raw pixel data
    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// Copy data from a slice into the buffer
    pub fn copy_from_slice(&mut self, data: &[u8]) {
        self.pixels.copy_from_slice(data);
    }

    /// Save as PPM (simple image format)
    pub fn save_ppm(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        // PPM header
        writeln!(writer, "P6")?;
        writeln!(writer, "{} {}", self.width, self.height)?;
        writeln!(writer, "255")?;

        // Write RGB data (skip alpha channel)
        for chunk in self.pixels.chunks_exact(4) {
            writer.write_all(&chunk[0..3])?;
        }

        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_creation() {
        let fb = FrameBuffer::new(1920, 1080);
        assert_eq!(fb.dimensions(), (1920, 1080));
        assert_eq!(fb.pixels.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_clear() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.clear([255, 0, 0, 255]); // Red

        assert_eq!(fb.get_pixel(0, 0), Some([255, 0, 0, 255]));
        assert_eq!(fb.get_pixel(50, 50), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_set_get_pixel() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.set_pixel(10, 20, [100, 150, 200, 255]);

        assert_eq!(fb.get_pixel(10, 20), Some([100, 150, 200, 255]));
        assert_eq!(fb.get_pixel(100, 100), None); // Out of bounds
    }

    #[test]
    fn test_alpha_blending() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.clear([255, 255, 255, 255]); // White background

        // Blend 50% transparent red
        fb.blend_pixel(50, 50, [255, 0, 0, 128]);

        let pixel = fb.get_pixel(50, 50).unwrap();
        // Should be approximately pink (255, 127, 127, 255)
        assert!(pixel[0] == 255);
        assert!(pixel[1] > 120 && pixel[1] < 135);
        assert!(pixel[2] > 120 && pixel[2] < 135);
    }
}
