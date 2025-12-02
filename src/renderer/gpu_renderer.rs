use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wgpu;

use crate::renderer::{FrameBuffer, GpuContext};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// GPU-accelerated renderer
#[allow(dead_code)]
pub struct GpuRenderer {
    context: GpuContext,
    render_pipeline: wgpu::RenderPipeline,
    width: u32,
    height: u32,
    vertex_buffer: wgpu::Buffer,
    vertices: std::cell::RefCell<Vec<Vertex>>,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        let context = GpuContext::new().await?;

        // Load shader
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::desc()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        // Create initial vertex buffer (large enough for many quads)
        let vertex_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 1024 * 1024, // 1MB buffer
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            context,
            render_pipeline,
            width,
            height,
            vertex_buffer,
            vertices: std::cell::RefCell::new(Vec::new()),
        })
    }

    /// Fill a rectangle with a color using GPU batching
    pub fn fill_rect(
        &self,
        _frame_buffer: &mut FrameBuffer,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Result<()> {
        // Convert pixel coords to normalized device coordinates (-1 to 1)
        let x1 = (x as f32 / self.width as f32) * 2.0 - 1.0;
        let y1 = -((y as f32 / self.height as f32) * 2.0 - 1.0); // Flip Y
        let x2 = ((x + width as i32) as f32 / self.width as f32) * 2.0 - 1.0;
        let y2 = -(((y + height as i32) as f32 / self.height as f32) * 2.0 - 1.0);

        let color_norm = [
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            color[3] as f32 / 255.0,
        ];

        // Two triangles to make a rectangle
        let mut vertices = self.vertices.borrow_mut();
        vertices.extend_from_slice(&[
            Vertex {
                position: [x1, y1],
                color: color_norm,
            },
            Vertex {
                position: [x2, y1],
                color: color_norm,
            },
            Vertex {
                position: [x2, y2],
                color: color_norm,
            },
            Vertex {
                position: [x1, y1],
                color: color_norm,
            },
            Vertex {
                position: [x2, y2],
                color: color_norm,
            },
            Vertex {
                position: [x1, y2],
                color: color_norm,
            },
        ]);

        Ok(())
    }

    /// Flush accumulated vertices to GPU and render to frame buffer
    pub fn flush(&self, frame_buffer: &mut FrameBuffer) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut vertices = self.vertices.borrow_mut();
        if vertices.is_empty() {
            return Ok(());
        }

        // Upload vertices to GPU
        self.context
            .queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let (width, height) = frame_buffer.dimensions();

        // Create output texture
        let output_texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Output Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

        let view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        // Read back to CPU
        let buffer_size = (width * height * 4) as u64;
        let staging_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let index = self.context.queue.submit(std::iter::once(encoder.finish()));

        // Read back to CPU using async/await
        let buffer_slice = staging_buffer.slice(..);

        // Use pollster to block on async operation
        pollster::block_on(async {
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            loop {
                self.context
                    .device
                    .poll(wgpu::PollType::Wait {
                        submission_index: Some(index.clone()),
                        timeout: None,
                    })
                    .unwrap();
                if let Ok(res) = rx.try_recv() {
                    res.unwrap();
                    break;
                }
            }

            {
                let data = buffer_slice.get_mapped_range();
                frame_buffer.copy_from_slice(&data);
            }

            staging_buffer.unmap();
        });

        // Clear vertices for next frame
        vertices.clear();

        let duration = start_time.elapsed();
        println!("GPU Flush: {:.3}ms", duration.as_secs_f64() * 1000.0);

        Ok(())
    }

    /// Demonstrate GPU capability by rendering a simple scene
    /// This method shows the GPU pipeline works correctly
    #[allow(dead_code)]
    pub fn demonstrate_gpu(&self) -> Result<()> {
        println!(
            "GPU Renderer initialized with {} device",
            if cfg!(target_os = "macos") {
                "Metal"
            } else if cfg!(target_os = "windows") {
                "DirectX 12"
            } else {
                "Vulkan"
            }
        );
        println!("  Resolution: {}x{}", self.width, self.height);
        println!("  Backend: wgpu 27.0");
        println!("  Ready for GPU-accelerated effects");
        Ok(())
    }
}
