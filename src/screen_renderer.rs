use std::fmt::Debug;
use backend::Colour;
use iced::widget::shader::wgpu::util::DeviceExt;
use iced::widget::shader::wgpu::*;
use iced::widget::shader::Primitive;
use iced::Rectangle;
use util::BufferInitDescriptor;

pub const SCREEN_SIZE: u32 = 450;

#[derive(Debug)]
pub struct Screen
{
    colours: Vec<Colour>,
    exposure: f32
}

impl Screen
{
    pub fn new(colours: Vec<Colour>, exposure: f32) -> Self
    {
        return Self { colours, exposure };
    }
}

impl Primitive for Screen
{
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        // custom pipelines go here
        storage: &mut iced::widget::shader::Storage,
        _bounds: &Rectangle,
        _viewport: &iced::widget::shader::Viewport)
    {
        let pipe = storage.get_mut::<ScreenPipe>();
        let pipe = match pipe
        {
            Some(lp) => lp,
            None =>
            {
                let lp = ScreenPipe::new(device, format);
                storage.store(lp);
                storage.get_mut::<ScreenPipe>().unwrap()
            },
        };
        
        let size = self.colours.len() as u32;
        
        queue.write_buffer(&pipe.uniform_buffer, 0,
            bytemuck::cast_slice(&[self.exposure]));
        
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            ImageCopyTexture {
                texture: &pipe.texture,
                mip_level: 0,
                aspect: TextureAspect::All,
                origin: Origin3d::ZERO
            },
            // The actual pixel data
            &bytemuck::cast_slice(&self.colours),
            // The layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16 * size),
                rows_per_image: None,
            },
            Extent3d { width: size, height: 1, depth_or_array_layers: 1 });
        
        // pipe.write_vertex(device, queue, &self.data);
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &Rectangle<u32>)
    {
        let pipe = storage.get::<ScreenPipe>();
        match pipe
        {
            Some(pipe) =>
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("lines.render"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &target,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None
                });
                
                render_pass.set_scissor_rect(clip_bounds.x, clip_bounds.y, clip_bounds.width, clip_bounds.height);
                render_pass.set_viewport(clip_bounds.x as f32,
                    clip_bounds.y as f32,
                    clip_bounds.width as f32,
                    clip_bounds.height as f32, 0.0, 1.0);
                render_pass.set_pipeline(&pipe.render_pipeline);
                render_pass.set_bind_group(0, &pipe.bind_group, &[]);
                
                render_pass.set_vertex_buffer(0, pipe.vertex_buffer.slice(..));
                
                render_pass.draw(0..4, 0..1);
            },
            None => {},
        };
    }
}

struct ScreenPipe
{
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    texture: Texture,
    // sampler: Sampler,
    bind_group: BindGroup
}

impl ScreenPipe
{
    pub fn new(
        device: &iced::widget::shader::wgpu::Device,
        format: iced::widget::shader::wgpu::TextureFormat) -> Self
    {
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("screen.uniform"),
            contents: bytemuck::cast_slice(&[0.0]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let texture = device.create_texture(&TextureDescriptor {
                size: Extent3d { width: SCREEN_SIZE, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D1,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label: Some("screen.colours"),
                view_formats: &[]
            }
        );
        
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("screen.uniform.bind"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D1,
                        sample_type: TextureSampleType::Float { filterable: false }
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ]
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("screen.uniform.group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ]
        });
        
        let shader = device.create_shader_module(include_wgsl!("screen_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("screen.rp.lay"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });
        
        let vertex_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("screen.verts"),
                contents: bytemuck::cast_slice(&[
                    // pos              uv
                    -1.0f32, 1.0f32,    0.0f32, 1.0f32,
                    1.0f32, 1.0f32,     1.0f32, 1.0f32,
                    -1.0f32, -1.0f32,   0.0f32, 0.0f32,
                    1.0f32, -1.0f32,    1.0f32, 0.0f32
                ]),
                usage: BufferUsages::VERTEX
            }
        );
        
        let buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 4]>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2]
        };
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("screen.pipe"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                // specify vertex buffer layout
                buffers: &[buffer_layout]
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })]
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                front_face: FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None
        });
        
        return Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            texture,
            // sampler,
            bind_group
        };
    }
}