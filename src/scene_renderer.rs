use std::fmt::Debug;

use backend::Colour;
use iced::widget::shader::wgpu::util::{BufferInitDescriptor, DeviceExt};
use iced::widget::shader::wgpu::*;
use iced::widget::shader::Primitive;
use iced::Rectangle;
use num::Zero;
use zene_structs::{Matrix4, Vector2, Vector4};

#[derive(Debug)]
pub struct SceneRender<const ID: usize>
{
    data: Vec<(Vector2<f32>, Colour)>,
    // background: Vector4<f32>,
    zoom: f32,
    pan: Vector2<f32>
}

impl<const ID: usize> SceneRender<ID>
{
    pub fn new(data: Vec<(Vector2<f32>, Colour)>, // background: Vector4<f32>,
        zoom: f32, pan: Vector2<f32>) -> Self
    {
        return Self {
            data,
            // background,
            zoom,
            pan
        };
    }
}

impl<const ID: usize> Primitive for SceneRender<ID>
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
        let pipe = storage.get_mut::<ScenePipe<ID>>();
        let pipe = match pipe
        {
            Some(lp) => lp,
            None =>
            {
                let lp = ScenePipe::<ID>::new(device, format);
                storage.store(lp);
                storage.get_mut::<ScenePipe<ID>>().unwrap()
            },
        };
        
        let size = self.data.len() as u32;
        
        let (width, height) = if bounds.width < bounds.height
        {
            (1.0, bounds.height / bounds.width)
        } else
        {
            (bounds.width / bounds.height, 1.0)
        };
        
        let uni_dat = Uniform {
            matrix: Matrix4::create_scale(self.zoom) *
                Matrix4::create_translation_2(self.pan) *
                Matrix4::create_orthographic(width, height, 0.0, 1.0),
            // background: self.background
        };
        queue.write_buffer(&pipe.uniform_buffer, 0,
            bytemuck::cast_slice(&[uni_dat]));
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &Rectangle<u32>)
    {
        let pipe = storage.get::<ScenePipe<ID>>();
        match pipe
        {
            Some(pipe) =>
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("scene.render"),
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
                
                render_pass.draw(0..(self.data.len() as u32), 0..1);
            },
            None => {},
        };
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
struct Uniform
{
    matrix: Matrix4<f32>,
    // background: Vector4<f32>
}
unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}
impl Default for Uniform
{
    fn default() -> Self
    {
        Self {
            matrix: Matrix4::identity(),
            // background: Vector4::zero()
        }
    }
}

struct ScenePipe<const ID: usize>
{
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    bind_group: BindGroup
}

impl<const ID: usize> ScenePipe<ID>
{
    pub fn new(
        device: &iced::widget::shader::wgpu::Device,
        format: iced::widget::shader::wgpu::TextureFormat) -> Self
    {
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("scene.uniform"),
            contents: bytemuck::cast_slice(&[Uniform::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("scene.uniform.bind"),
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
                }
            ]
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("scene.uniform.group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ]
        });
        
        let shader = device.create_shader_module(include_wgsl!("line_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("scene.rp.lay"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });
        
        let vertex_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("scene.verts"),
                contents: &[],
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
            }
        );
        
        let buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 4]>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2]
        };
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("scene.pipe"),
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
            bind_group
        };
    }
}