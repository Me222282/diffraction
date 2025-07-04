use std::fmt::Debug;

use bytemuck::NoUninit;
use iced::widget::shader::wgpu::util::{BufferInitDescriptor, DeviceExt};
use iced::widget::shader::wgpu::*;
use iced::widget::shader::Primitive;
use iced::Rectangle;
use num::Zero;
use zene_structs::{Vector2, Vector4};

use crate::SPECTRUM_SIZE;

pub trait TextureData
{
    const FORMAT: TextureFormat;
}
impl TextureData for f32
{
    const FORMAT: TextureFormat = TextureFormat::R32Float;
}
impl TextureData for [f32; 4]
{
    const FORMAT: TextureFormat = TextureFormat::Rgba32Float;
}

#[derive(Debug)]
pub struct PlotRender<D: TextureData, const ID: usize>
{
    data: Vec<D>,
    foreground: Vector4,
    background: Vector4,
    scale: f32,
    uv_scale: f32,
    uv_offset: f32
}

impl<D: TextureData, const ID: usize> PlotRender<D, ID>
{
    pub fn new(data: Vec<D>, foreground: Vector4,
        background: Vector4, scale: f32, uv_scale: f32, uv_offset: f32) -> Self
    {
        return Self {
            data,
            foreground,
            background,
            scale,
            uv_scale,
            uv_offset
        };
    }
}

impl<D: TextureData, const ID: usize> Primitive for PlotRender<D, ID>
    where D: Debug + Send + Sync + NoUninit + 'static
{
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        // custom pipelines go here
        storage: &mut iced::widget::shader::Storage,
        bounds: &Rectangle,
        _viewport: &iced::widget::shader::Viewport)
    {
        let pipe = storage.get_mut::<PlotPipe<ID>>();
        let pipe = match pipe
        {
            Some(lp) => lp,
            None =>
            {
                let lp = PlotPipe::<ID>::new::<D>(device, format);
                storage.store(lp);
                storage.get_mut::<PlotPipe<ID>>().unwrap()
            },
        };
        
        let size = self.data.len() as u32;
        
        let bh = bounds.height;
        let uni_dat = Uniform {
            foreground: self.foreground,
            background: self.background,
            h_size: Vector2::new(size as f32, bh * self.scale),
            // _pad: [0.0, 0.0],
            uv_trans: Vector2::new(self.uv_scale * bh, self.uv_offset * bh),
        };
        queue.write_buffer(&pipe.uniform_buffer, 0,
            bytemuck::cast_slice(&[uni_dat]));
        
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            ImageCopyTexture {
                texture: &pipe.sample_texture,
                mip_level: 0,
                aspect: TextureAspect::All,
                origin: Origin3d::ZERO
            },
            // The actual pixel data
            &bytemuck::cast_slice(&self.data),
            // The layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size_of::<D>() as u32 * size),
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
        let pipe = storage.get::<PlotPipe<ID>>();
        match pipe
        {
            Some(pipe) =>
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("plot.render"),
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

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
struct Uniform
{
    foreground: Vector4,
    background: Vector4,
    h_size: Vector2,
    // _pad: [f32; 2],
    uv_trans: Vector2
}
unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}
impl Default for Uniform
{
    fn default() -> Self
    {
        Self {
            foreground: Vector4::zero(),
            background: Vector4::zero(),
            h_size: Vector2::zero(),
            // _pad: [0.0, 0.0],
            uv_trans: Vector2::zero()
        }
    }
}

struct PlotPipe<const ID: usize>
{
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    sample_texture: Texture,
    bind_group: BindGroup
}

impl<const ID: usize> PlotPipe<ID>
{
    pub fn new<D: TextureData>(
        device: &iced::widget::shader::wgpu::Device,
        format: iced::widget::shader::wgpu::TextureFormat) -> Self
    {
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("plot.uniform"),
            contents: bytemuck::cast_slice(&[Uniform::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let sample_texture = device.create_texture(&TextureDescriptor {
                size: Extent3d { width: SPECTRUM_SIZE, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D1,
                format: D::FORMAT,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label: Some("plot.samples"),
                view_formats: &[]
            }
        );
        
        let sample_texture_view = sample_texture.create_view(&TextureViewDescriptor::default());
        
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("plot.uniform.bind"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D1,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                }
            ]
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("plot.uniform.group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&sample_texture_view),
                }
            ]
        });
        
        let shader = device.create_shader_module(include_wgsl!("line_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("plot.rp.lay"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });
        
        let vertex_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("plot.verts"),
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
            label: Some("plot.pipe"),
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
            sample_texture,
            bind_group
        };
    }
}