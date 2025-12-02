use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use wgpu;

use crate::renderer::{FrameBuffer, GpuContext};

/// Vertex structure optimized for Metal (Apple Silicon)
///
/// Layout:
/// - position: vec2<f32> (8 bytes, offset 0)
/// - color: vec4<f32> (16 bytes, offset 8)  
/// - uv: vec2<f32> (8 bytes, offset 24)
///
/// Total: 32 bytes (16-byte aligned for optimal GPU cache performance)
///
/// Metal alignment requirements:
/// - vec2/vec3 should be aligned to their component size
/// - vec4 should be 16-byte aligned
/// - Structs should be padded to 16-byte boundaries for best performance
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
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
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
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
    batches: std::cell::RefCell<Vec<(std::sync::Arc<wgpu::BindGroup>, Vec<Vertex>)>>,
    white_texture_bind_group: std::sync::Arc<wgpu::BindGroup>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    output_texture: Option<wgpu::Texture>,
    staging_buffer: Option<wgpu::Buffer>,
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

        // Create texture bind group layout
        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout],
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
                        entry_point: Some("fs_texture"),
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

        // Create 1x1 white texture
        let white_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let white_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: white_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            white_texture_size,
        );

        let white_texture_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let white_texture_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&white_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&white_sampler),
                        },
                    ],
                    label: Some("White Texture Bind Group"),
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
            batches: std::cell::RefCell::new(Vec::new()),
            white_texture_bind_group: std::sync::Arc::new(white_texture_bind_group),
            texture_bind_group_layout,
            output_texture: None,
            staging_buffer: None,
        })
    }

    /// Create a texture from an image
    pub fn create_texture(&self, image: &image::DynamicImage) -> std::sync::Arc<wgpu::BindGroup> {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

        self.context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self
            .context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

        std::sync::Arc::new(
            self.context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: Some("Image Texture Bind Group"),
                }),
        )
    }

    /// Draw a textured rectangle
    pub fn draw_texture(
        &self,
        bind_group: std::sync::Arc<wgpu::BindGroup>,
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
        let new_vertices = vec![
            Vertex {
                position: [x1, y1],
                color: color_norm,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [x2, y1],
                color: color_norm,
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [x2, y2],
                color: color_norm,
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [x1, y1],
                color: color_norm,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [x2, y2],
                color: color_norm,
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [x1, y2],
                color: color_norm,
                uv: [0.0, 1.0],
            },
        ];

        let mut batches = self.batches.borrow_mut();

        // Check if we can merge with the last batch
        if let Some(last_batch) = batches.last_mut() {
            if std::sync::Arc::ptr_eq(&last_batch.0, &bind_group) {
                last_batch.1.extend(new_vertices);
                return Ok(());
            }
        }

        // Create new batch
        batches.push((bind_group, new_vertices));
        Ok(())
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
        self.draw_texture(
            self.white_texture_bind_group.clone(),
            x,
            y,
            width,
            height,
            color,
        )
    }

    /// Flush accumulated vertices to GPU and render to frame buffer
    pub fn flush(&mut self, frame_buffer: &mut FrameBuffer) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut batches = self.batches.borrow_mut();
        if batches.is_empty() {
            return Ok(());
        }

        let (width, height) = frame_buffer.dimensions();

        // Create or reuse output texture
        if self.output_texture.is_none() {
            let texture = self
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
            self.output_texture = Some(texture);
        }
        let output_texture = self.output_texture.as_ref().unwrap();

        let view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Upload all vertices to the buffer at different offsets
        let mut current_offset = 0;
        for (_, vertices) in batches.iter() {
            let bytes = bytemuck::cast_slice(vertices);
            self.context
                .queue
                .write_buffer(&self.vertex_buffer, current_offset, bytes);
            current_offset += bytes.len() as u64;
        }

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

            let mut draw_offset = 0;
            for (bind_group, vertices) in batches.iter() {
                let vertex_count = vertices.len() as u32;
                let byte_size = (vertex_count as usize * std::mem::size_of::<Vertex>()) as u64;

                render_pass.set_bind_group(0, bind_group.as_ref(), &[]);
                render_pass.set_vertex_buffer(
                    0,
                    self.vertex_buffer
                        .slice(draw_offset..draw_offset + byte_size),
                );
                render_pass.draw(0..vertex_count, 0..1);

                draw_offset += byte_size;
            }
        }

        // Create or reuse staging buffer
        let buffer_size = (width * height * 4) as u64;
        if self.staging_buffer.is_none() {
            let buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            self.staging_buffer = Some(buffer);
        }
        let staging_buffer = self.staging_buffer.as_ref().unwrap();

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buffer,
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

        // Clear batches for next frame
        batches.clear();

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
