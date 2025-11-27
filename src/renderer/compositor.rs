use crate::renderer::FrameBuffer;
use crate::script::Transform;

/// Layer compositor
pub struct Compositor;

impl Compositor {
    /// Fill rectangle with color
    pub fn fill_rect(
        buffer: &mut FrameBuffer,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) {
        let (buf_width, buf_height) = buffer.dimensions();

        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx as i32;
                let py = y + dy as i32;

                if px >= 0 && py >= 0 && (px as u32) < buf_width && (py as u32) < buf_height {
                    buffer.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }

    /// Draw text (placeholder - simple rectangle for now)
    pub fn draw_text_placeholder(
        buffer: &mut FrameBuffer,
        text: &str,
        x: i32,
        y: i32,
        color: [u8; 4],
    ) {
        // Placeholder: draw a colored rectangle representing text
        let width = (text.len() as u32 * 8).min(200);
        let height = 16;
        Self::fill_rect(buffer, x, y, width, height, color);
    }

    /// Apply transform to coordinates
    pub fn apply_transform(x: i32, y: i32, transform: &Transform) -> (i32, i32) {
        // Apply position offset
        let tx = x + transform.position.x;
        let ty = y + transform.position.y;

        // TODO: Apply scale and rotation
        // For now, just position offset

        (tx, ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::Position;

    #[test]
    fn test_fill_rect() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.clear([0, 0, 0, 255]);

        Compositor::fill_rect(&mut fb, 10, 10, 20, 20, [255, 0, 0, 255]);

        // Inside rectangle should be red
        assert_eq!(fb.get_pixel(15, 15), Some([255, 0, 0, 255]));

        // Outside should be black
        assert_eq!(fb.get_pixel(5, 5), Some([0, 0, 0, 255]));
    }

    #[test]
    fn test_apply_transform() {
        let transform = Transform {
            position: Position { x: 100, y: 50 },
            scale: 1.0,
            rotation: 0.0,
            opacity: 1.0,
        };

        let (tx, ty) = Compositor::apply_transform(10, 20, &transform);
        assert_eq!((tx, ty), (110, 70));
    }
}
